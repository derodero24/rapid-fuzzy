//! Typo-tolerant search (the `maxTypos` option): matches without typos keep
//! their scores, typo matches are added below them, and every entry point
//! (standalone search, `FuzzyIndex`, multi-key search and `KeyedFuzzyIndex`)
//! returns the same results.

use nucleo_matcher::pattern::CaseMatching;
use rapid_fuzzy_core::search::{
    FuzzyIndexCore, KeyMatchMode, KeySearchResult, KeyedFuzzyIndexCore, SearchKeysOptions,
    SearchResult, search_keys_impl, search_over_items, search_over_items_with_typos,
};

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

fn search(query: &str, items: &[String], max_typos: u16) -> Vec<SearchResult> {
    search_over_items_with_typos(
        query,
        items,
        None,
        None,
        true,
        CaseMatching::Smart,
        max_typos,
    )
}

fn found(results: &[SearchResult]) -> Vec<&str> {
    results.iter().map(|r| r.item.as_str()).collect()
}

/// Items mixing identifiers, paths, phrases and non-ASCII text.
fn corpus() -> Vec<String> {
    let mut items = strings(&[
        "TypeScript",
        "JavaScript",
        "CoffeeScript",
        "PureScript",
        "typescript-eslint",
        "src/components/Button.tsx",
        "src/components/ButtonGroup.tsx",
        "src/utils/format-date.ts",
        "README.md",
        "package.json",
        "HTMLInputElement",
        "HTMLTextAreaElement",
        "addEventListener",
        "removeEventListener",
        "Café crème",
        "Crème brûlée",
        "naïve résumé",
        "東京都 渋谷区",
        "Zürich Hauptbahnhof",
        "the quick brown fox",
    ]);
    items.extend((0..300).map(|i| format!("filler_entry_{i}")));
    items
}

#[test]
fn swapped_and_wrong_letters_match_with_typos() {
    let items = corpus();
    for query in ["tpyescript", "typozcript", "typescirpt", "tyepscript"] {
        assert!(
            search(query, &items, 0)
                .iter()
                .all(|r| r.item != "TypeScript"),
            "{query} should not find TypeScript without typos"
        );
        let results = search(query, &items, 2);
        assert!(
            found(&results).contains(&"TypeScript"),
            "{query}: {:?}",
            found(&results)
        );
    }
    assert_eq!(
        found(&search("buttn", &items, 1))[0],
        "src/components/Button.tsx"
    );
    assert_eq!(found(&search("evnet listener", &items, 1)).len(), 2);
}

#[test]
fn matches_without_typos_keep_their_scores() {
    let items = corpus();
    for query in [
        "ts",
        "button",
        "btn",
        "script",
        "event",
        "html elem",
        "crème",
        "cafe",
        "src tsx",
        "'Script",
        "^src",
        "json$",
        "!filler",
        "type !eslint",
    ] {
        let exact = search(query, &items, 0);
        let with_typos = search(query, &items, 2);
        for r in &exact {
            let same = with_typos
                .iter()
                .find(|t| t.index == r.index)
                .unwrap_or_else(|| panic!("{query}: {} lost with typos", r.item));
            assert_eq!(
                same.score.to_bits(),
                r.score.to_bits(),
                "{query}: {}",
                r.item
            );
            assert_eq!(same.positions, r.positions, "{query}: {}", r.item);
        }
    }
}

#[test]
fn typo_matches_rank_below_perfect_matches() {
    let items = strings(&["TypeScript", "TypeScropt"]);
    let results = search("typescript", &items, 1);
    assert_eq!(found(&results), ["TypeScript", "TypeScropt"]);
    assert!(results[1].score < results[0].score);
    assert!(results[1].score > 0.0);
}

#[test]
fn short_terms_and_syntax_terms_match_without_typos() {
    let items = strings(&["cat", "car", "card", "category"]);
    // Terms of three characters or fewer tolerate no typo.
    assert_eq!(found(&search("cqt", &items, 3)), Vec::<&str>::new());
    // Exact, prefix, suffix and negated terms are matched exactly.
    let items = corpus();
    assert!(search("'tpyescript", &items, 2).is_empty());
    assert!(search("^tpyescript", &items, 2).is_empty());
    assert!(search("tpyescript$", &items, 2).is_empty());
    // A negated term excludes only what it matches exactly.
    let kept = search("typescript !esltn", &items, 2);
    assert!(found(&kept).contains(&"typescript-eslint"));
}

#[test]
fn non_ascii_items_fold_like_nucleo() {
    let items = corpus();
    // "cafw": a typo for "cafe", which matches "Café" through diacritic and
    // case folding.
    let results = search("cafw", &items, 1);
    assert_eq!(found(&results).first(), Some(&"Café crème"));
    let positions = &results[0].positions;
    assert!(!positions.is_empty());
    assert!(positions.iter().all(|&p| p < 10), "{positions:?}");
    // "zurihc" (a swap) finds "Zürich".
    assert!(found(&search("zurihc", &items, 1)).contains(&"Zürich Hauptbahnhof"));
}

#[test]
fn positions_point_into_the_item() {
    let items = corpus();
    for query in [
        "tpyescript",
        "buttn",
        "evnet",
        "naive resume",
        "brulee creme",
    ] {
        for r in search(query, &items, 2) {
            let len = r.item.chars().count() as u32;
            assert!(
                r.positions.iter().all(|&p| p < len),
                "{query}: {} {:?}",
                r.item,
                r.positions
            );
            let mut sorted = r.positions.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted, r.positions, "{query}: {}", r.item);
        }
    }
}

#[test]
fn zero_typos_is_the_default_search() {
    let items = corpus();
    for query in ["tpyescript", "button", "cafe"] {
        let default = search_over_items(query, &items, None, None, true, CaseMatching::Smart);
        let zero = search(query, &items, 0);
        assert_eq!(found(&default), found(&zero));
    }
}

#[test]
fn the_index_returns_what_search_returns() {
    let items = corpus();
    let index = FuzzyIndexCore::new(items.clone());
    // Type-ahead sequences exercise the incremental cache.
    for query in [
        "t",
        "tp",
        "tpy",
        "tpye",
        "tpyes",
        "tpyescript",
        "buttn",
        "evnet listener",
        "cafw",
        "zurihc",
        "htmlinputelemnt",
        "filler_entyr",
        "typescript !esltn",
    ] {
        for max_typos in [0, 1, 2] {
            let expected = search(query, &items, max_typos);
            let actual =
                index.search_with_typos(query, None, None, true, CaseMatching::Smart, max_typos);
            let summary = |rs: &[SearchResult]| -> Vec<(u32, u64, Vec<u32>)> {
                rs.iter()
                    .map(|r| (r.index, r.score.to_bits(), r.positions.clone()))
                    .collect()
            };
            assert_eq!(
                summary(&actual),
                summary(&expected),
                "{query} ({max_typos})"
            );
        }
    }
}

fn keyed(
    query: &str,
    key_texts: &[Vec<String>],
    match_mode: KeyMatchMode,
    max_typos: u16,
) -> Vec<KeySearchResult> {
    let options = SearchKeysOptions {
        match_mode: Some(match_mode),
        max_typos: Some(max_typos),
        ..SearchKeysOptions::default()
    };
    let standalone = search_keys_impl(query, key_texts, &[2.0, 1.0], Some(options)).unwrap();
    let index = KeyedFuzzyIndexCore::new(key_texts.to_vec(), vec![2.0, 1.0]).unwrap();
    let indexed = index.search_with_options(query, options);
    let summary = |rs: &[KeySearchResult]| -> Vec<(u32, u64)> {
        rs.iter().map(|r| (r.index, r.score.to_bits())).collect()
    };
    assert_eq!(summary(&indexed), summary(&standalone), "{query}");
    standalone
}

#[test]
fn keyed_searches_tolerate_typos() {
    let names = strings(&["Alice Johnson", "Bob Smith", "Carol Williams", "Dave Brown"]);
    let cities = strings(&["Tokyo", "Zürich", "New York", "Paris"]);
    let key_texts = vec![names, cities];
    for mode in [KeyMatchMode::PerKey, KeyMatchMode::CrossKey] {
        let results = keyed("wilaims", &key_texts, mode, 1);
        assert_eq!(results.first().map(|r| r.index), Some(2), "{mode:?}");
        let results = keyed("zurihc", &key_texts, mode, 1);
        assert_eq!(results.first().map(|r| r.index), Some(1), "{mode:?}");
        assert!(keyed("wilaims", &key_texts, mode, 0).is_empty(), "{mode:?}");
    }
    // Cross-key: one term per key, one of them with a typo.
    let results = keyed("dave pairs", &key_texts, KeyMatchMode::CrossKey, 1);
    assert_eq!(results.iter().map(|r| r.index).collect::<Vec<_>>(), [3]);
    assert!(keyed("dave pairs", &key_texts, KeyMatchMode::CrossKey, 0).is_empty());
}
