use std::cell::RefCell;
use std::collections::HashMap;

use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32String};

/// Classification of how a query matched an item.
///
/// Derived from the matched character positions:
/// - **Exact**: all positions consecutive from index 0, covering every character in the item.
/// - **Prefix**: all positions consecutive from index 0, but the item is longer.
/// - **Contains**: all positions consecutive (a substring match), not starting at 0.
/// - **Fuzzy**: positions have gaps (character-level fuzzy match).
#[derive(Debug, Clone, PartialEq)]
pub enum MatchType {
    Exact,
    Prefix,
    Contains,
    Fuzzy,
}

/// A single fuzzy search result with the matched item and its score.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The original string that matched.
    pub item: String,
    /// The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
    pub score: f64,
    /// The index of the item in the original input array.
    pub index: u32,
    /// Indices of matched characters in the item string.
    /// Empty unless `include_positions` is set to true.
    pub positions: Vec<u32>,
    /// How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
    /// Only present when `include_positions` is set to true.
    pub match_type: Option<MatchType>,
}

/// A lightweight search result containing only index and score (no item string).
///
/// Use this when you maintain your own data array and only need the index
/// to look up the original item. Avoids String cloning overhead.
#[derive(Debug, Clone)]
pub struct IndexSearchResult {
    /// The index of the item in the original input array.
    pub index: u32,
    /// The match score normalized to 0.0-1.0 range (1.0 is a perfect match).
    pub score: f64,
    /// Indices of matched characters in the item string.
    /// Empty unless `include_positions` is set to true.
    pub positions: Vec<u32>,
    /// How the query matched this item (Exact, Prefix, Contains, or Fuzzy).
    /// Only present when `include_positions` is set to true.
    pub match_type: Option<MatchType>,
}

/// A single result from multi-key fuzzy search.
#[derive(Debug, Clone)]
pub struct KeySearchResult {
    /// The index of the item in the original input array.
    pub index: u32,
    /// The combined weighted score normalized to 0.0-1.0 range.
    pub score: f64,
    /// Per-key scores in the same order as the input keys.
    /// A score of 0.0 means the item did not match on that key.
    pub key_scores: Vec<f64>,
}

/// Classify the match type from the matched character positions.
pub fn classify_match(positions: &[u32], item_char_count: usize) -> MatchType {
    if positions.is_empty() {
        return MatchType::Fuzzy;
    }

    let is_consecutive = positions.len() == 1 || positions.windows(2).all(|w| w[1] == w[0] + 1);

    if is_consecutive {
        if positions[0] == 0 && positions.len() == item_char_count {
            MatchType::Exact
        } else if positions[0] == 0 {
            MatchType::Prefix
        } else {
            MatchType::Contains
        }
    } else {
        MatchType::Fuzzy
    }
}

/// Compute the maximum possible score for a given pattern by scoring
/// the query against itself (exact match = theoretical maximum).
pub fn compute_max_score(query: &str, pattern: &Pattern, matcher: &mut Matcher) -> f64 {
    let mut buf = Vec::new();
    let atoms = nucleo_matcher::Utf32Str::new(query, &mut buf);
    pattern.score(atoms, matcher).unwrap_or(1) as f64
}

/// Convert the `is_case_sensitive` flag into a `CaseMatching` variant.
pub fn resolve_case_matching(is_case_sensitive: Option<bool>) -> CaseMatching {
    match is_case_sensitive {
        Some(true) => CaseMatching::Respect,
        _ => CaseMatching::Smart,
    }
}

thread_local! {
    /// Reusable Matcher for standalone search/closest calls.
    /// Avoids allocating internal scoring matrices on every invocation.
    /// FuzzyIndex has its own Matcher, so this is only for the standalone path.
    static STANDALONE_MATCHER: RefCell<Matcher> = RefCell::new(Matcher::new(Config::DEFAULT));
}

/// Compute a character-presence bitmask for quick pre-filtering.
///
/// Bit layout: 0–25 = a–z (case-insensitive), 26–35 = 0–9.
/// Characters outside this range are ignored (conservative — no false negatives).
pub fn compute_char_mask(s: &str) -> u64 {
    let mut mask = 0u64;
    for c in s.chars() {
        let bit = match c {
            'a'..='z' => Some(c as u32 - 'a' as u32),
            'A'..='Z' => Some(c as u32 - 'A' as u32),
            '0'..='9' => Some(26 + c as u32 - '0' as u32),
            _ => None,
        };
        if let Some(b) = bit {
            mask |= 1u64 << b;
        }
    }
    mask
}

/// Extract a character-presence bitmask from a query string, considering
/// only positive (non-inverted) terms. Inverted terms (`!term`) and
/// syntax prefixes/suffixes (`^`, `'`, `$`) are stripped.
pub fn compute_query_mask(query: &str) -> u64 {
    let mut mask = 0u64;
    for term in query.split_whitespace() {
        if term.starts_with('!') {
            continue;
        }
        let term = term.trim_start_matches(['^', '\'']);
        let term = term.trim_end_matches('$');
        mask |= compute_char_mask(term);
    }
    mask
}

/// Encode a character into a u8 bucket for bigram key construction.
///
/// ASCII letters are case-folded to 0–25, digits to 26–35.
/// Non-ASCII characters are hashed into the 36–255 range.
pub(crate) fn char_bucket(c: char) -> u8 {
    match c {
        'a'..='z' => (c as u32 - 'a' as u32) as u8,
        'A'..='Z' => (c as u32 - 'A' as u32) as u8,
        '0'..='9' => (26 + c as u32 - '0' as u32) as u8,
        _ => (36 + (c as u32 % 220)) as u8,
    }
}

/// Encode a character pair into a u16 bigram key.
pub fn bigram_key(a: char, b: char) -> u16 {
    ((char_bucket(a) as u16) << 8) | char_bucket(b) as u16
}

/// Extract bigram keys from a string.
pub fn extract_bigrams(s: &str) -> Vec<u16> {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() < 2 {
        return Vec::new();
    }
    let mut bigrams = Vec::with_capacity(chars.len() - 1);
    for pair in chars.windows(2) {
        bigrams.push(bigram_key(pair[0], pair[1]));
    }
    bigrams.sort_unstable();
    bigrams.dedup();
    bigrams
}

/// Extract bigram keys from a query string, respecting nucleo syntax.
///
/// Skips inverted terms (`!term`) and strips syntax prefixes/suffixes (`^`, `'`, `$`).
pub fn extract_query_bigrams(query: &str) -> Vec<u16> {
    let mut all_bigrams = Vec::new();
    for term in query.split_whitespace() {
        if term.starts_with('!') {
            continue;
        }
        let term = term.trim_start_matches(['^', '\'']);
        let term = term.trim_end_matches('$');
        let chars: Vec<char> = term.chars().collect();
        if chars.len() < 2 {
            continue;
        }
        for pair in chars.windows(2) {
            all_bigrams.push(bigram_key(pair[0], pair[1]));
        }
    }
    all_bigrams.sort_unstable();
    all_bigrams.dedup();
    all_bigrams
}

/// Intersect two sorted slices of u32, returning a new sorted Vec.
pub fn intersect_sorted(a: &[u32], b: &[u32]) -> Vec<u32> {
    let mut result = Vec::with_capacity(a.len().min(b.len()));
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                result.push(a[i]);
                i += 1;
                j += 1;
            }
        }
    }
    result
}

/// Inverted bigram index for pre-filtering candidates in FuzzyIndex.
///
/// Maps each bigram (pair of adjacent characters) to a sorted list of item indices
/// that contain that bigram. At query time, posting lists are intersected to find
/// candidates that contain all query bigrams.
pub struct BigramIndex {
    /// Inverted index: bigram_key → sorted list of item indices containing that bigram.
    index: HashMap<u16, Vec<u32>>,
    /// Total number of items in the index.
    num_items: usize,
}

impl BigramIndex {
    /// Build a bigram index from a slice of items.
    pub fn new(items: &[String]) -> Self {
        let mut index: HashMap<u16, Vec<u32>> = HashMap::new();
        for (i, item) in items.iter().enumerate() {
            let bigrams = extract_bigrams(item);
            for bg in bigrams {
                index.entry(bg).or_default().push(i as u32);
            }
        }
        Self {
            index,
            num_items: items.len(),
        }
    }

    /// Get candidate indices that contain all query bigrams.
    ///
    /// Returns `None` if the query has no bigrams (single-char query)
    /// or if the intersection is not significantly smaller than the full set.
    pub fn candidates(&self, query_bigrams: &[u16]) -> Option<Vec<u32>> {
        if query_bigrams.is_empty() || self.num_items == 0 {
            return None;
        }

        // Find posting lists for each query bigram, sorted by length (shortest first).
        let mut lists: Vec<&Vec<u32>> = Vec::with_capacity(query_bigrams.len());
        for &bg in query_bigrams {
            match self.index.get(&bg) {
                Some(list) => lists.push(list),
                None => return Some(Vec::new()), // bigram not present → no candidates
            }
        }
        lists.sort_by_key(|l| l.len());

        // Intersect all posting lists starting from the shortest.
        let mut result = lists[0].clone();
        for list in &lists[1..] {
            result = intersect_sorted(&result, list);
            if result.is_empty() {
                return Some(Vec::new());
            }
        }

        // Skip if the intersection is > 80% of total items (overhead not worth it).
        if result.len() * 5 > self.num_items * 4 {
            return None;
        }

        Some(result)
    }

    /// Add a single item to the index.
    pub fn add_item(&mut self, index: u32, item: &str) {
        let bigrams = extract_bigrams(item);
        for bg in bigrams {
            self.index.entry(bg).or_default().push(index);
        }
        self.num_items += 1;
    }

    /// Handle swap_remove semantics: remove item at `removed_index`, and if the
    /// last item was swapped in, update its index from `last_index` to `removed_index`.
    pub fn remove_item(
        &mut self,
        removed_index: u32,
        last_index: u32,
        removed_item: &str,
        last_item: Option<&str>,
    ) {
        // Remove all bigrams of the removed item.
        let removed_bigrams = extract_bigrams(removed_item);
        for bg in removed_bigrams {
            if let Some(list) = self.index.get_mut(&bg) {
                list.retain(|&x| x != removed_index);
                if list.is_empty() {
                    self.index.remove(&bg);
                }
            }
        }

        // If a swap happened (removed_index != last_index), update the swapped item's index.
        if let Some(last_item_str) = last_item
            && removed_index != last_index
        {
            let last_bigrams = extract_bigrams(last_item_str);
            for bg in last_bigrams {
                if let Some(list) = self.index.get_mut(&bg) {
                    for val in list.iter_mut() {
                        if *val == last_index {
                            *val = removed_index;
                            break;
                        }
                    }
                    list.sort_unstable();
                }
            }
        }

        self.num_items = self.num_items.saturating_sub(1);
    }

    /// Clear the index.
    pub fn clear(&mut self) {
        self.index.clear();
        self.num_items = 0;
    }
}

/// Pre-computed search context for FuzzyIndex.
pub struct PrecomputedSearch<'a> {
    pub items: &'a [String],
    pub utf32_items: &'a [Utf32String],
    pub char_masks: &'a [u64],
    /// Optional subset of item indices to search (for incremental narrowing).
    /// When `None`, all items are searched.
    pub candidate_indices: Option<&'a [u32]>,
    pub matcher: &'a RefCell<Matcher>,
}

/// Result of `search_over_precomputed`: the top-k results plus all matching indices.
pub struct PrecomputedSearchResult {
    /// Top-k search results (sorted, truncated).
    pub results: Vec<SearchResult>,
    /// Indices of ALL items that matched (before truncation), for incremental cache.
    pub all_matching_indices: Vec<u32>,
}

/// Lightweight result of `search_over_precomputed_indices`: index-only results.
pub struct PrecomputedIndexSearchResult {
    /// Top-k index-only results (sorted, truncated).
    pub results: Vec<IndexSearchResult>,
    /// Indices of ALL items that matched (before truncation), for incremental cache.
    pub all_matching_indices: Vec<u32>,
}

/// Shared search logic over a borrowed slice of items.
///
/// Both the standalone `search_impl` and `FuzzyIndex::search_impl` delegate
/// to this function, which contains the core scoring/filtering/sorting logic.
pub fn search_over_items(
    query: &str,
    items: &[String],
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    if query.is_empty() || items.is_empty() {
        return Vec::new();
    }

    STANDALONE_MATCHER.with(|cell| {
        let mut matcher = cell.borrow_mut();
        let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
        let max_score = compute_max_score(query, &pattern, &mut matcher);
        let threshold = min_score.unwrap_or(0.0);

        let mut buf = Vec::new();

        // Pass 1: Score all items, collect (index, score) only — no String cloning.
        let mut scored: Vec<(u32, f64)> = items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| {
                buf.clear();
                let atoms = nucleo_matcher::Utf32Str::new(item, &mut buf);
                let raw_score = pattern.score(atoms, &mut matcher)?;
                let normalized = (raw_score as f64 / max_score).min(1.0);
                if normalized >= threshold {
                    Some((index as u32, normalized))
                } else {
                    None
                }
            })
            .collect();

        // Sort by score descending, with shorter items first as tiebreaker,
        // then by original index for fully deterministic ordering.
        let cmp = |a: &(u32, f64), b: &(u32, f64)| {
            let score_ord = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
            if score_ord != std::cmp::Ordering::Equal {
                return score_ord;
            }
            let len_ord = items[a.0 as usize].len().cmp(&items[b.0 as usize].len());
            if len_ord != std::cmp::Ordering::Equal {
                return len_ord;
            }
            a.0.cmp(&b.0)
        };

        // Top-k selection: use quickselect O(n) + sort O(k log k) instead of
        // full sort O(n log n) when maxResults is set.
        if let Some(max) = max_results {
            let k = max as usize;
            if scored.len() > k {
                scored.select_nth_unstable_by(k, cmp);
                scored.truncate(k);
            }
        }
        scored.sort_unstable_by(cmp);

        // Pass 2: Construct results only for the final top-k items.
        scored
            .into_iter()
            .map(|(index, score)| {
                let (positions, match_type) = if include_positions {
                    buf.clear();
                    let atoms = nucleo_matcher::Utf32Str::new(&items[index as usize], &mut buf);
                    let mut indices = Vec::new();
                    pattern.indices(atoms, &mut matcher, &mut indices);
                    indices.sort_unstable();
                    indices.dedup();
                    let item_char_count = items[index as usize].chars().count();
                    let mt = classify_match(&indices, item_char_count);
                    (indices, Some(mt))
                } else {
                    (Vec::new(), None)
                };

                SearchResult {
                    item: items[index as usize].clone(),
                    score,
                    index,
                    positions,
                    match_type,
                }
            })
            .collect()
    })
}

/// Search over pre-computed Utf32String items with a reusable Matcher.
///
/// Used by FuzzyIndex to avoid per-search string conversion and Matcher allocation.
/// When `ctx.candidate_indices` is set, only those items are scored (incremental search).
pub fn search_over_precomputed(
    query: &str,
    ctx: &PrecomputedSearch<'_>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> PrecomputedSearchResult {
    let items = ctx.items;
    let utf32_items = ctx.utf32_items;
    let matcher_cell = ctx.matcher;
    if query.is_empty() || items.is_empty() {
        return PrecomputedSearchResult {
            results: Vec::new(),
            all_matching_indices: Vec::new(),
        };
    }

    let mut matcher = matcher_cell.borrow_mut();
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let max_score = compute_max_score(query, &pattern, &mut matcher);
    let threshold = min_score.unwrap_or(0.0);
    let query_mask = compute_query_mask(query);
    let char_masks = ctx.char_masks;

    // Scoring closure shared by both full-scan and candidate paths.
    let mut score_item = |index: u32| -> Option<(u32, f64)> {
        let idx = index as usize;
        if query_mask != 0 && (char_masks[idx] & query_mask) != query_mask {
            return None;
        }
        let atoms = utf32_items[idx].slice(..);
        let raw_score = pattern.score(atoms, &mut matcher)?;
        let normalized = (raw_score as f64 / max_score).min(1.0);
        if normalized >= threshold {
            Some((index, normalized))
        } else {
            None
        }
    };

    // Pass 1: Score items. Use candidate_indices if provided (incremental search).
    let mut scored: Vec<(u32, f64)> = match ctx.candidate_indices {
        Some(candidates) => candidates.iter().filter_map(|&i| score_item(i)).collect(),
        None => (0..utf32_items.len() as u32)
            .filter_map(&mut score_item)
            .collect(),
    };

    // Collect matching indices for incremental cache, but only when the match set
    // is meaningfully smaller than the full dataset. When most items match (e.g.,
    // short queries), the cache overhead outweighs the narrowing benefit.
    let total = utf32_items.len();
    let all_matching_indices: Vec<u32> = if total > 0 && scored.len() < total / 2 {
        scored.iter().map(|&(index, _)| index).collect()
    } else {
        Vec::new()
    };

    // Sort by score descending, with shorter items first as tiebreaker,
    // then by original index for fully deterministic ordering.
    let cmp = |a: &(u32, f64), b: &(u32, f64)| {
        let score_ord = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
        if score_ord != std::cmp::Ordering::Equal {
            return score_ord;
        }
        let len_ord = items[a.0 as usize].len().cmp(&items[b.0 as usize].len());
        if len_ord != std::cmp::Ordering::Equal {
            return len_ord;
        }
        a.0.cmp(&b.0)
    };

    // Top-k selection: use quickselect O(n) + sort O(k log k) instead of
    // full sort O(n log n) when maxResults is set.
    if let Some(max) = max_results {
        let k = max as usize;
        if scored.len() > k {
            scored.select_nth_unstable_by(k, cmp);
            scored.truncate(k);
        }
    }
    scored.sort_unstable_by(cmp);

    // Pass 2: Construct results only for the final top-k items.
    let results = scored
        .into_iter()
        .map(|(index, score)| {
            let (positions, match_type) = if include_positions {
                let atoms = utf32_items[index as usize].slice(..);
                let mut indices = Vec::new();
                pattern.indices(atoms, &mut matcher, &mut indices);
                indices.sort_unstable();
                indices.dedup();
                let item_char_count = items[index as usize].chars().count();
                let mt = classify_match(&indices, item_char_count);
                (indices, Some(mt))
            } else {
                (Vec::new(), None)
            };

            SearchResult {
                item: items[index as usize].clone(),
                score,
                index,
                positions,
                match_type,
            }
        })
        .collect();

    PrecomputedSearchResult {
        results,
        all_matching_indices,
    }
}

/// Index-only variant of `search_over_precomputed`.
///
/// Reuses the same Pass 1 (scoring, filtering, top-k selection) but builds
/// lightweight `IndexSearchResult` objects in Pass 2 — no String cloning.
pub fn search_over_precomputed_indices(
    query: &str,
    ctx: &PrecomputedSearch<'_>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> PrecomputedIndexSearchResult {
    let items = ctx.items;
    let utf32_items = ctx.utf32_items;
    let matcher_cell = ctx.matcher;
    if query.is_empty() || items.is_empty() {
        return PrecomputedIndexSearchResult {
            results: Vec::new(),
            all_matching_indices: Vec::new(),
        };
    }

    let mut matcher = matcher_cell.borrow_mut();
    let pattern = Pattern::parse(query, case_matching, Normalization::Smart);
    let max_score = compute_max_score(query, &pattern, &mut matcher);
    let threshold = min_score.unwrap_or(0.0);
    let query_mask = compute_query_mask(query);
    let char_masks = ctx.char_masks;

    let mut score_item = |index: u32| -> Option<(u32, f64)> {
        let idx = index as usize;
        if query_mask != 0 && (char_masks[idx] & query_mask) != query_mask {
            return None;
        }
        let atoms = utf32_items[idx].slice(..);
        let raw_score = pattern.score(atoms, &mut matcher)?;
        let normalized = (raw_score as f64 / max_score).min(1.0);
        if normalized >= threshold {
            Some((index, normalized))
        } else {
            None
        }
    };

    let mut scored: Vec<(u32, f64)> = match ctx.candidate_indices {
        Some(candidates) => candidates.iter().filter_map(|&i| score_item(i)).collect(),
        None => (0..utf32_items.len() as u32)
            .filter_map(&mut score_item)
            .collect(),
    };

    let total = utf32_items.len();
    let all_matching_indices: Vec<u32> = if total > 0 && scored.len() < total / 2 {
        scored.iter().map(|&(index, _)| index).collect()
    } else {
        Vec::new()
    };

    let cmp = |a: &(u32, f64), b: &(u32, f64)| {
        let score_ord = b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal);
        if score_ord != std::cmp::Ordering::Equal {
            return score_ord;
        }
        let len_ord = items[a.0 as usize].len().cmp(&items[b.0 as usize].len());
        if len_ord != std::cmp::Ordering::Equal {
            return len_ord;
        }
        a.0.cmp(&b.0)
    };

    if let Some(max) = max_results {
        let k = max as usize;
        if scored.len() > k {
            scored.select_nth_unstable_by(k, cmp);
            scored.truncate(k);
        }
    }
    scored.sort_unstable_by(cmp);

    // Pass 2: Build IndexSearchResult without String cloning.
    let results = scored
        .into_iter()
        .map(|(index, score)| {
            let (positions, match_type) = if include_positions {
                let atoms = utf32_items[index as usize].slice(..);
                let mut indices = Vec::new();
                pattern.indices(atoms, &mut matcher, &mut indices);
                indices.sort_unstable();
                indices.dedup();
                let item_char_count = items[index as usize].chars().count();
                let mt = classify_match(&indices, item_char_count);
                (indices, Some(mt))
            } else {
                (Vec::new(), None)
            };

            IndexSearchResult {
                index,
                score,
                positions,
                match_type,
            }
        })
        .collect();

    PrecomputedIndexSearchResult {
        results,
        all_matching_indices,
    }
}

/// Internal search implementation used by both the napi export and tests.
pub fn search_impl(
    query: String,
    items: Vec<String>,
    max_results: Option<u32>,
    min_score: Option<f64>,
    include_positions: bool,
    case_matching: CaseMatching,
) -> Vec<SearchResult> {
    search_over_items(
        &query,
        &items,
        max_results,
        min_score,
        include_positions,
        case_matching,
    )
}
