use std::cell::RefCell;

use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Utf32Str, chars};

use super::{
    Corpus, IndexSearchResult, QueryPlan, RankedMatch, SearchParams, SearchResult,
    haystack_char_mask, index_haystack, normalize_query_whitespace, search_core, with_matcher,
};

/// Matches of the last unfiltered search, reused to narrow the next one.
struct SearchCache {
    /// The query (after whitespace normalization) the matches belong to.
    query: String,
    /// Case mode the matches were computed with.
    case_matching: CaseMatching,
    /// Ascending indices of every item matching `query`.
    matching: Vec<u32>,
}

/// Whether every item matching `new` provably also matches `old`, so that
/// `old`'s matches are a complete candidate set for `new`.
///
/// This holds when `new` extends `old` with more characters and neither
/// uses any query syntax: each term of `old` is then a term of `new` or a
/// prefix of one, and a fuzzy term only matches items that also match every
/// prefix of it. Typing an uppercase letter can switch a term from
/// case-insensitive to case-sensitive matching, which only removes matches.
///
/// Refused conservatively:
/// - syntax characters (`!`, `^`, `$`, `'`, `\`): anchors, negation and
///   escapes change the meaning of earlier characters (`fo$` -> `fo$x`,
///   `foo\` -> `foo\ bar`);
/// - non-ASCII characters that nucleo would normalize (like `é`): they turn
///   off normalization for their whole term, which can make the extended
///   term match items the shorter one did not.
fn refines(old: &str, new: &str) -> bool {
    new.len() > old.len()
        && new.starts_with(old)
        && !new.contains(['!', '^', '$', '\'', '\\'])
        && new
            .chars()
            .all(|c| c.is_ascii() || chars::normalize(c) == c)
}

/// Heap bytes owned by one item: its string and, for non-ASCII items, its
/// haystack.
fn item_heap_bytes(item: &String, haystack: Option<&[char]>) -> usize {
    item.capacity() + haystack.map_or(0, size_of_val)
}

/// Core state and logic for a persistent fuzzy search index.
///
/// This struct contains all platform-independent state and methods.
/// Binding crates (napi, wasm) wrap this with their own FFI layer.
///
/// Searches return exactly what the standalone `search` returns for the same
/// items; the index only avoids work (pre-converted haystacks, a character
/// mask pre-filter and an incremental cache for type-ahead queries).
///
/// The index owns no nucleo matcher (searches use the shared per-thread one,
/// see [`with_matcher`]), so [`destroy`](Self::destroy) frees all of its heap
/// memory.
pub struct FuzzyIndexCore {
    items: Vec<String>,
    /// Haystack of every non-ASCII item; `None` for ASCII items, which are
    /// matched as their own bytes instead of being stored twice.
    haystacks: Vec<Option<Box<[char]>>>,
    char_masks: Vec<u64>,
    /// Union of the character masks of every item ever added since the last
    /// `destroy` (removals do not clear bits, so it may over-approximate).
    /// A query needing a character no item has cannot match anything.
    union_mask: u64,
    /// Sum of [`item_heap_bytes`] over all items, for [`Self::heap_size`].
    item_bytes: usize,
    cache: RefCell<Option<SearchCache>>,
}

impl FuzzyIndexCore {
    /// Create a new FuzzyIndexCore from a list of items.
    pub fn new(items: Vec<String>) -> Self {
        let mut index = Self {
            items: Vec::new(),
            haystacks: Vec::new(),
            char_masks: Vec::new(),
            union_mask: 0,
            item_bytes: 0,
            cache: RefCell::new(None),
        };
        index.extend(items);
        index
    }

    /// Append items, computing their haystacks and character masks.
    fn extend(&mut self, items: Vec<String>) {
        // A new index gets exactly the capacity it needs; additions grow
        // geometrically so that repeated `add` calls stay amortized O(1).
        if self.items.is_empty() {
            self.haystacks.reserve_exact(items.len());
            self.char_masks.reserve_exact(items.len());
        } else {
            self.haystacks.reserve(items.len());
            self.char_masks.reserve(items.len());
        }
        for item in &items {
            let haystack = index_haystack(item);
            let mask = haystack_char_mask(match &haystack {
                Some(chars) => Utf32Str::Unicode(chars),
                None => Utf32Str::Ascii(item.as_bytes()),
            });
            self.item_bytes += item_heap_bytes(item, haystack.as_deref());
            self.union_mask |= mask;
            self.char_masks.push(mask);
            self.haystacks.push(haystack);
        }
        if self.items.is_empty() {
            self.items = items;
        } else {
            self.items.extend(items);
        }
    }

    /// Return the number of items in the index.
    pub fn size(&self) -> u32 {
        self.items.len() as u32
    }

    /// Access the items slice.
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Approximate number of heap bytes the index owns: its items, their
    /// pre-computed search data and the vectors holding them. The transient
    /// incremental-search cache is not included.
    ///
    /// Bindings report this to their JavaScript engine so that its garbage
    /// collector accounts for the index's native memory.
    pub fn heap_size(&self) -> usize {
        self.item_bytes
            + self.items.capacity() * size_of::<String>()
            + self.haystacks.capacity() * size_of::<Option<Box<[char]>>>()
            + self.char_masks.capacity() * size_of::<u64>()
    }

    /// Run a search, narrowing it with the incremental cache when sound.
    fn search_ranked(
        &self,
        query: &str,
        params: SearchParams,
        case_matching: CaseMatching,
    ) -> Vec<RankedMatch> {
        let query = normalize_query_whitespace(query);
        let Some(plan) = with_matcher(|matcher| QueryPlan::new(&query, case_matching, matcher))
        else {
            return Vec::new();
        };
        if self.union_mask & plan.char_mask != plan.char_mask {
            // Some character of the query occurs in no item.
            return Vec::new();
        }

        // A `min_score` threshold drops items that a longer query could still
        // match, so only unfiltered searches (including `closest` without a
        // threshold) may refresh the cache.
        let unfiltered = params.min_score.is_none_or(|score| score <= 0.0);
        let corpus = Corpus::Indexed {
            items: &self.items,
            haystacks: &self.haystacks,
            char_masks: &self.char_masks,
        };
        let outcome = {
            let cache = self.cache.borrow();
            let candidates = cache
                .as_ref()
                .filter(|cache| {
                    cache.case_matching == case_matching
                        && !cache.matching.is_empty()
                        && refines(&cache.query, &query)
                })
                .map(|cache| cache.matching.as_slice());
            with_matcher(|matcher| {
                search_core(&plan, matcher, corpus, candidates, params, unfiltered)
            })
        };

        if unfiltered {
            *self.cache.borrow_mut() = outcome.all_matching.map(|matching| SearchCache {
                query: query.into_owned(),
                case_matching,
                matching,
            });
        }

        outcome.matches
    }

    /// Search the index for items matching the query.
    pub fn search_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<SearchResult> {
        let params = SearchParams {
            max_results,
            min_score,
            include_positions,
        };
        self.search_ranked(query, params, case_matching)
            .into_iter()
            .map(|m| SearchResult {
                item: self.items[m.index as usize].clone(),
                score: m.score,
                index: m.index,
                positions: m.positions,
                match_type: m.match_type,
            })
            .collect()
    }

    /// Search the index, returning only indices and scores (no item strings).
    pub fn search_indices_impl(
        &self,
        query: &str,
        max_results: Option<u32>,
        min_score: Option<f64>,
        include_positions: bool,
        case_matching: CaseMatching,
    ) -> Vec<IndexSearchResult> {
        let params = SearchParams {
            max_results,
            min_score,
            include_positions,
        };
        self.search_ranked(query, params, case_matching)
            .into_iter()
            .map(|m| IndexSearchResult {
                index: m.index,
                score: m.score,
                positions: m.positions,
                match_type: m.match_type,
            })
            .collect()
    }

    /// Add a single item to the index.
    pub fn add(&mut self, item: String) {
        self.extend(vec![item]);
        self.invalidate_cache();
    }

    /// Add multiple items to the index at once.
    pub fn add_many(&mut self, items: Vec<String>) {
        self.extend(items);
        self.invalidate_cache();
    }

    /// Remove the item at the given index.
    ///
    /// Uses swap-remove for O(1) performance. Returns false if out of bounds.
    pub fn remove(&mut self, index: u32) -> bool {
        let idx = index as usize;
        if idx < self.items.len() {
            let item = self.items.swap_remove(idx);
            let haystack = self.haystacks.swap_remove(idx);
            self.char_masks.swap_remove(idx);
            self.item_bytes -= item_heap_bytes(&item, haystack.as_deref());
            self.invalidate_cache();
            true
        } else {
            false
        }
    }

    /// Free the internal data. After calling this, the index is empty and
    /// owns no heap memory.
    pub fn destroy(&mut self) {
        self.items = Vec::new();
        self.haystacks = Vec::new();
        self.char_masks = Vec::new();
        self.union_mask = 0;
        self.item_bytes = 0;
        self.invalidate_cache();
    }

    fn invalidate_cache(&self) {
        *self.cache.borrow_mut() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::refines;

    #[test]
    fn refines_plain_extensions() {
        assert!(refines("fo", "foo"));
        assert!(refines("foo", "foo bar"));
        assert!(refines("foo", "fooB"));
        assert!(refines("東", "東京"));
        assert!(!refines("foo", "foo"));
        assert!(!refines("foo", "fo"));
        assert!(!refines("foo", "bar foo"));
    }

    #[test]
    fn refines_refuses_syntax_and_normalizing_chars() {
        assert!(!refines("fo$", "fo$x"));
        assert!(!refines("foo\\", "foo\\ bar"));
        assert!(!refines("foo\\", "foo\\$"));
        assert!(!refines("foo", "foo !bar"));
        assert!(!refines("foo", "foo ^bar"));
        assert!(!refines("foo", "foo 'bar"));
        assert!(!refines("caf", "café"));
    }
}
