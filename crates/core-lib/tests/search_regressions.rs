//! Regression tests for search correctness bugs: each test pins one bug that
//! made `FuzzyIndex` disagree with `search()`, or both return wrong results.

use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Config, Matcher};
use rapid_fuzzy_core::search::{
    FuzzyIndexCore, MAX_TERM_CHARS, MatchType, QueryPlan, SearchResult, is_empty_query,
    normalize_query_whitespace, search_impl,
};

fn search(query: &str, items: &[&str]) -> Vec<SearchResult> {
    search_with(query, items, None, None)
}

fn search_with(
    query: &str,
    items: &[&str],
    max_results: Option<u32>,
    min_score: Option<f64>,
) -> Vec<SearchResult> {
    search_impl(
        query.to_owned(),
        items.iter().map(|s| s.to_string()).collect(),
        max_results,
        min_score,
        true,
        CaseMatching::Smart,
    )
}

fn index(items: &[&str]) -> FuzzyIndexCore {
    FuzzyIndexCore::new(items.iter().map(|s| s.to_string()).collect())
}

fn index_search(index: &FuzzyIndexCore, query: &str) -> Vec<SearchResult> {
    index.search_impl(query, None, None, true, CaseMatching::Smart)
}

fn found(results: &[SearchResult]) -> Vec<&str> {
    results.iter().map(|r| r.item.as_str()).collect()
}

/// 6000 items that do not match any of the queries below, so that the index
/// is above the size where it used to pre-filter with bigrams.
fn large(extra: &[&str]) -> Vec<String> {
    (0..6000)
        .map(|i| format!("filler_entry_{i}"))
        .chain(extra.iter().map(|s| s.to_string()))
        .collect()
}

fn assert_same(index: &FuzzyIndexCore, items: &[String], query: &str) -> Vec<SearchResult> {
    let expected = search_impl(
        query.to_owned(),
        items.to_vec(),
        None,
        None,
        true,
        CaseMatching::Smart,
    );
    let got = index_search(index, query);
    assert_eq!(
        format!("{got:?}"),
        format!("{expected:?}"),
        "FuzzyIndex differs from search() for {query:?}"
    );
    got
}

// ─── Bigram pre-filter (#746) ───────────────────────────────────────────────

#[test]
fn large_index_finds_gapped_and_unicode_matches() {
    let items = large(&[
        "handler",
        "my_handler_x",
        "Москва",
        "e\u{301}cole",
        "foo bar",
        "foo$",
        "src/components/index.ts",
    ]);
    let index = FuzzyIndexCore::new(items.clone());
    for (query, expected) in [
        ("hndlr", vec!["handler", "my_handler_x"]),
        ("москва", vec!["Москва"]),
        ("ecole", vec!["e\u{301}cole"]),
        ("foo\\ bar", vec!["foo bar"]),
        ("foo\\$", vec!["foo$"]),
        ("src/index.ts", vec!["src/components/index.ts"]),
    ] {
        let got = assert_same(&index, &items, query);
        assert_eq!(found(&got), expected, "{query:?}");
    }
}

// ─── Character mask case folding ────────────────────────────────────────────

#[test]
fn index_matches_uppercase_non_ascii_items() {
    let items = [
        "Łódź",
        "ŠKODA",
        "Ōsaka",
        "Đakovo",
        "\u{212A}elvin",
        "Ćevapi",
    ];
    let index = index(&items);
    for (query, expected) in [
        ("łódź", "Łódź"),
        ("škoda", "ŠKODA"),
        ("ōsaka", "Ōsaka"),
        ("đakovo", "Đakovo"),
        ("kelvin", "\u{212A}elvin"),
        ("ćevapi", "Ćevapi"),
    ] {
        assert_eq!(
            found(&search(query, &items)),
            [expected],
            "search {query:?}"
        );
        assert_eq!(
            found(&index_search(&index, query)),
            [expected],
            "FuzzyIndex {query:?}"
        );
    }
}

// ─── Haystack conversion (NFD text) ─────────────────────────────────────────

#[test]
fn nfd_items_use_grapheme_positions_everywhere() {
    let items = ["e\u{301}cole"];
    let expected = search("ecole", &items);
    assert_eq!(expected.len(), 1);
    // One position per grapheme, not per UTF-8 byte.
    assert_eq!(expected[0].positions, vec![0, 1, 2, 3, 4]);
    assert_eq!(expected[0].match_type, Some(MatchType::Exact));
    assert_eq!(expected[0].score, 1.0);
    let got = index_search(&index(&items), "ecole");
    assert_eq!(format!("{got:?}"), format!("{expected:?}"));
}

// ─── Max score with query syntax ────────────────────────────────────────────

#[test]
fn anchored_queries_are_normalized_against_their_own_terms() {
    let results = search("bar$", &["foobar", "xbar", "bar"]);
    assert_eq!(found(&results), ["bar", "xbar", "foobar"]);
    assert_eq!(results[0].score, 1.0);
    assert!(results[1].score < 1.0, "{results:?}");
    assert!(results[2].score < 1.0, "{results:?}");

    let results = search("^f ob", &["f_ob", "foo-ob"]);
    assert!(results.iter().all(|r| r.score <= 1.0));

    // A negative term does not lower the maximum score, so minScore filters.
    let strict = search_with("fob !zzz", &["fob", "foobar", "f_o_b"], None, Some(0.99));
    assert_eq!(found(&strict), ["fob"]);

    // Escaped spaces: the single term `foo bar` scores 1.0 on an exact match.
    let results = search("foo\\ bar", &["foo  bar", "foo bar"]);
    assert_eq!(found(&results), ["foo bar", "foo  bar"]);
    assert_eq!(results[0].score, 1.0);
    assert!(results[1].score < 1.0, "{results:?}");
}

#[test]
fn plain_query_scores_are_unchanged() {
    // The per-term maximum equals the old whole-query self-match for plain
    // queries, so their scores did not change (values from v2.1.1).
    let results = search("typscript", &["TypeScript", "JavaScript"]);
    assert_eq!(found(&results), ["TypeScript"]);
    let results = search("hlo", &["hello world"]);
    assert_eq!(results[0].positions, vec![0, 3, 4]);
    let results = search("john smith", &["Smith, John", "John Smith"]);
    assert_eq!(results[1].score, results[0].score.min(results[1].score));
    assert_eq!(search("apple", &["apple"])[0].score, 1.0);
    assert_eq!(search("foo bar", &["foo bar"])[0].score, 1.0);
}

// ─── Syntax-only queries ────────────────────────────────────────────────────

#[test]
fn syntax_only_queries_are_empty() {
    let items = ["a", "b", "^", "$"];
    let index = index(&items);
    for query in [
        "^", "'", "$", "!", "^$", "!^", "!'", "! ^ $", "   ", "\u{3000}",
    ] {
        assert!(is_empty_query(query), "{query:?}");
        assert!(search(query, &items).is_empty(), "search {query:?}");
        assert!(
            index_search(&index, query).is_empty(),
            "FuzzyIndex {query:?}"
        );
    }
    for query in ["a", "\\^", "\\$", "!a", "^a"] {
        assert!(!is_empty_query(query), "{query:?}");
    }
}

// ─── Incremental cache ──────────────────────────────────────────────────────

#[test]
fn cache_is_not_reused_when_syntax_changes_meaning() {
    let items = [
        "fo$x", "xfo", "foo bar", "foo\\x", "foo$", "a", "b", "c", "d",
    ];
    for (first, second) in [("fo$", "fo$x"), ("foo\\", "foo\\ bar"), ("foo\\", "foo\\$")] {
        let index = index(&items);
        let _ = index_search(&index, first);
        let got = index_search(&index, second);
        let expected = search(second, &items);
        assert!(!expected.is_empty(), "{second:?}");
        assert_eq!(
            format!("{got:?}"),
            format!("{expected:?}"),
            "{first:?} -> {second:?}"
        );
    }
}

#[test]
fn cache_is_not_reused_across_normalizing_characters() {
    // nucleo compares `ſ` (long s) with `s` only on some of its code paths
    // when normalization is off, which a term containing `é` turns off: `és`
    // does not match `éſxſ` but `ésx` does.
    let items = ["és", "éſxſ", "zzz", "yyy", "www"];
    let index = index(&items);
    assert_eq!(found(&index_search(&index, "és")), ["és"]);
    let expected = search("ésx", &items);
    assert_eq!(found(&expected), ["éſxſ"]);
    let got = index_search(&index, "ésx");
    assert_eq!(format!("{got:?}"), format!("{expected:?}"));
}

#[test]
fn cache_survives_typing_and_mutation() {
    let mut items: Vec<String> = ["handler", "hand", "handle_x", "other", "o", "p", "q", "r"]
        .map(String::from)
        .to_vec();
    let mut index = FuzzyIndexCore::new(items.clone());
    for q in ["h", "ha", "han", "hand", "handl", "handle"] {
        assert_same(&index, &items, q);
    }
    index.add("handler2".into());
    items.push("handler2".into());
    assert_same(&index, &items, "handler");
    assert!(index.remove(0));
    items.swap_remove(0);
    assert_same(&index, &items, "handlers");
    assert_same(&index, &items, "handler");
}

// ─── Score overflow ─────────────────────────────────────────────────────────

#[test]
fn longest_supported_term_scores_without_overflow() {
    let term = "a".repeat(MAX_TERM_CHARS);
    let mut matcher = Matcher::new(Config::DEFAULT);
    let plan = QueryPlan::new(&term, CaseMatching::Smart, &mut matcher).expect("supported");
    // 26 * m + 10 is the largest score nucleo can produce for m characters;
    // it must still fit in nucleo's u16 score.
    assert_eq!(plan.max_score, (26 * MAX_TERM_CHARS + 10) as f64);
    assert!(plan.max_score <= f64::from(u16::MAX));

    let items = [term.as_str(), "aaa"];
    let results = search(&term, &items);
    assert_eq!(found(&results), [term.as_str()]);
    assert_eq!(results[0].score, 1.0);
    assert_eq!(found(&index_search(&index(&items), &term)), [term.as_str()]);
}

#[test]
fn over_long_terms_match_nothing() {
    let mut matcher = Matcher::new(Config::DEFAULT);
    for len in [MAX_TERM_CHARS + 1, 10_000] {
        let term = "a".repeat(len);
        assert!(QueryPlan::new(&term, CaseMatching::Smart, &mut matcher).is_none());
        let items = [term.as_str(), "a"];
        assert!(search(&term, &items).is_empty(), "len {len}");
        assert!(index_search(&index(&items), &term).is_empty(), "len {len}");
        // Also as a negative term, and next to a valid term.
        assert!(
            search(&format!("a !{term}"), &items).is_empty(),
            "len {len}"
        );
    }
    // Long queries made of short terms are fine.
    let many_terms = vec!["a"; 5000].join(" ");
    assert_eq!(search(&many_terms, &["a"]).len(), 1);
}

// ─── Unicode whitespace ─────────────────────────────────────────────────────

#[test]
fn unicode_whitespace_separates_terms() {
    let items = ["東京都港区", "大阪府", "foo bar", "bar foo"];
    let index = index(&items);
    for (query, expected) in [
        ("東京\u{3000}港区", vec!["東京都港区"]),
        ("港区\u{3000}東京", vec!["東京都港区"]),
        ("foo\u{a0}bar", vec!["foo bar", "bar foo"]),
        ("foo\tbar", vec!["foo bar", "bar foo"]),
        ("foo\nbar", vec!["foo bar", "bar foo"]),
        ("foo\r\nbar", vec!["foo bar", "bar foo"]),
        ("\u{3000}foo\u{3000}", vec!["foo bar", "bar foo"]),
    ] {
        let mut expected = expected.clone();
        expected.sort_unstable();
        let results = search(query, &items);
        let mut got = found(&results);
        got.sort_unstable();
        assert_eq!(got, expected, "search {query:?}");
        let results = index_search(&index, query);
        let mut got = found(&results);
        got.sort_unstable();
        assert_eq!(got, expected, "FuzzyIndex {query:?}");
    }
    // A backslash still escapes the (normalized) space.
    assert_eq!(normalize_query_whitespace("foo\\\u{3000}bar"), "foo\\ bar");
    assert_eq!(found(&search("foo\\\u{3000}bar", &items)), ["foo bar"]);
    assert_eq!(normalize_query_whitespace("plain"), "plain");
}
