//! Multi-key search: `KeyedFuzzyIndexCore::search` must return exactly what
//! the standalone `search_keys_impl` returns (indices, scores, key scores and
//! order) for any key texts, weights, query, options and score mode, and both
//! must agree with a naive reference implementation of the documented
//! semantics:
//!
//! * every key is scored like `search()` scores an item (`keyScores`, also
//!   for keys whose weight is zero);
//! * the combined score depends on the score mode, over the keys with a
//!   positive weight (`w`) and their scores (`s`):
//!   - `Weighted` (the default): `sum(w * s) / sum(w)`;
//!   - `Matched`: `sum(w * s) / sum(w of the keys with s > 0)`;
//!   - `Max`: `max(s)`;
//! * items whose combined score is 0 (no match on a key with a positive
//!   weight) are excluded, and so are items scoring below `minScore`;
//! * results are ordered by score (descending), then by the byte length of
//!   the best-matching key's text (shorter first, like `search()`), then by
//!   index. The best-matching key is the one contributing the most to the
//!   combined score (`w * s`, or `s` in `Max` mode; the first one on a tie).
//!
//! Random inputs come from a seeded generator so failures are reproducible.

use nucleo_matcher::pattern::CaseMatching;
use nucleo_matcher::{Config, Matcher};
use rapid_fuzzy_core::search::{
    KeyScoreMode, KeySearchResult, KeyedFuzzyIndexCore, QueryPlan, SearchKeysOptions,
    is_empty_query, search_impl, search_keys_impl, utf32_haystack,
};

const MODES: [KeyScoreMode; 3] = [
    KeyScoreMode::Weighted,
    KeyScoreMode::Matched,
    KeyScoreMode::Max,
];

// ─── Helpers ────────────────────────────────────────────────────────────────

fn strings(xs: &[&str]) -> Vec<String> {
    xs.iter().map(|s| s.to_string()).collect()
}

fn columns(cols: &[&[&str]]) -> Vec<Vec<String>> {
    cols.iter().map(|c| strings(c)).collect()
}

#[derive(Clone, Copy, Debug, Default)]
struct Opts {
    max_results: Option<u32>,
    min_score: Option<f64>,
    case_sensitive: bool,
    return_all_on_empty: bool,
    score_mode: KeyScoreMode,
}

impl Opts {
    fn case_matching(self) -> CaseMatching {
        if self.case_sensitive {
            CaseMatching::Respect
        } else {
            CaseMatching::Smart
        }
    }

    fn to_keys_options(self) -> SearchKeysOptions {
        SearchKeysOptions {
            max_results: self.max_results,
            min_score: self.min_score,
            is_case_sensitive: Some(self.case_sensitive),
            return_all_on_empty: Some(self.return_all_on_empty),
            score_mode: Some(self.score_mode),
        }
    }
}

fn via_index(
    query: &str,
    key_texts: &[Vec<String>],
    weights: &[f64],
    o: Opts,
) -> Vec<KeySearchResult> {
    let index =
        KeyedFuzzyIndexCore::new(key_texts.to_vec(), weights.to_vec()).expect("valid index input");
    index.search(
        query,
        o.max_results,
        o.min_score,
        o.case_matching(),
        o.return_all_on_empty,
        o.score_mode,
    )
}

fn via_search_keys(
    query: &str,
    key_texts: &[Vec<String>],
    weights: &[f64],
    o: Opts,
) -> Vec<KeySearchResult> {
    search_keys_impl(query, key_texts, weights, Some(o.to_keys_options()))
        .expect("valid searchKeys input")
}

/// Naive reference implementation of the documented semantics.
fn reference(
    query: &str,
    key_texts: &[Vec<String>],
    weights: &[f64],
    o: Opts,
) -> Vec<KeySearchResult> {
    let num_keys = key_texts.len();
    let num_items = key_texts.first().map_or(0, Vec::len);
    if num_keys == 0 || num_items == 0 {
        return Vec::new();
    }
    if o.return_all_on_empty && is_empty_query(query) {
        let limit = o.max_results.map_or(usize::MAX, |m| m as usize);
        return (0..num_items)
            .take(limit)
            .map(|i| KeySearchResult {
                index: i as u32,
                score: 1.0,
                key_scores: vec![1.0; num_keys],
            })
            .collect();
    }
    let mut matcher = Matcher::new(Config::DEFAULT);
    let Some(plan) = QueryPlan::new(query, o.case_matching(), &mut matcher) else {
        return Vec::new();
    };
    let total: f64 = weights.iter().sum();
    let threshold = o.min_score.unwrap_or(0.0);
    let mut buf = Vec::new();
    // (index, score, tie length, key scores)
    let mut rows: Vec<(u32, f64, usize, Vec<f64>)> = Vec::new();
    for i in 0..num_items {
        let key_scores: Vec<f64> = key_texts
            .iter()
            .map(|col| {
                let haystack = utf32_haystack(&col[i], &mut buf);
                plan.pattern
                    .score(haystack, &mut matcher)
                    .map_or(0.0, |raw| plan.normalize(raw))
            })
            .collect();
        let weighted: f64 = key_scores
            .iter()
            .zip(weights)
            .map(|(s, w)| s * w)
            .fold(0.0, |acc, x| acc + x);
        let combined = match o.score_mode {
            KeyScoreMode::Weighted => weighted / total,
            KeyScoreMode::Matched => {
                let matched: f64 = key_scores
                    .iter()
                    .zip(weights)
                    .filter(|&(&s, &w)| s > 0.0 && w > 0.0)
                    .map(|(_, w)| w)
                    .fold(0.0, |acc, x| acc + x);
                if matched > 0.0 {
                    weighted / matched
                } else {
                    0.0
                }
            }
            KeyScoreMode::Max => key_scores
                .iter()
                .zip(weights)
                .filter(|&(_, &w)| w > 0.0)
                .map(|(&s, _)| s)
                .fold(0.0, f64::max),
        };
        if combined > 0.0 && combined >= threshold {
            // What a key contributes to the combined score.
            let contribution = |k: usize| match o.score_mode {
                KeyScoreMode::Max if weights[k] > 0.0 => key_scores[k],
                KeyScoreMode::Max => 0.0,
                _ => key_scores[k] * weights[k],
            };
            let mut best = 0;
            for k in 1..num_keys {
                if contribution(k) > contribution(best) {
                    best = k;
                }
            }
            rows.push((i as u32, combined, key_texts[best][i].len(), key_scores));
        }
    }
    rows.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap()
            .then(a.2.cmp(&b.2))
            .then(a.0.cmp(&b.0))
    });
    if let Some(max) = o.max_results {
        rows.truncate(max as usize);
    }
    rows.into_iter()
        .map(|(index, score, _, key_scores)| KeySearchResult {
            index,
            score,
            key_scores,
        })
        .collect()
}

fn summary(results: &[KeySearchResult]) -> Vec<(u32, u64, Vec<u64>)> {
    results
        .iter()
        .map(|r| {
            (
                r.index,
                r.score.to_bits(),
                r.key_scores.iter().map(|s| s.to_bits()).collect(),
            )
        })
        .collect()
}

#[track_caller]
fn assert_all_agree(query: &str, key_texts: &[Vec<String>], weights: &[f64], o: Opts) {
    let expected = summary(&reference(query, key_texts, weights, o));
    let keys = summary(&via_search_keys(query, key_texts, weights, o));
    let index = summary(&via_index(query, key_texts, weights, o));
    let context = format!("query={query:?} key_texts={key_texts:?} weights={weights:?} opts={o:?}");
    assert_eq!(keys, expected, "searchKeys != reference for {context}");
    assert_eq!(
        index, expected,
        "KeyedFuzzyIndex != reference for {context}"
    );
}

// ─── Regressions ────────────────────────────────────────────────────────────

#[test]
fn min_score_equal_to_the_combined_score_keeps_the_item() {
    // The index's early exit subtracted weights from a running total and
    // rejected item 0, whose combined score is exactly 1.
    let weights = [0.494, 0.953, 0.137, 0.447, 0.673];
    let key_texts = vec![strings(&["foo", "bar"]); 5];
    let o = Opts {
        min_score: Some(1.0),
        ..Opts::default()
    };
    let index = via_index("foo", &key_texts, &weights, o);
    let keys = via_search_keys("foo", &key_texts, &weights, o);
    assert_eq!(index.len(), 1);
    assert_eq!(keys.len(), 1);
    assert_eq!(index[0].index, 0);
    assert_eq!(index[0].score, 1.0);
    assert_all_agree("foo", &key_texts, &weights, o);
}

#[test]
fn early_exit_never_rejects_a_qualifying_item() {
    let mut rng = Rng(7);
    for _ in 0..300 {
        let num_keys = 1 + rng.below(6);
        let weights: Vec<f64> = (0..num_keys).map(|_| rng.weight()).collect();
        if weights.iter().sum::<f64>() <= 0.0 {
            continue;
        }
        let key_texts: Vec<Vec<String>> = (0..num_keys)
            .map(|_| {
                (0..3)
                    .map(|_| rng.pick(&["foo", "fo", "bar", "f_o_o", "xfoo"]).to_string())
                    .collect()
            })
            .collect();
        for score_mode in MODES {
            assert_boundary_thresholds_agree("foo", &key_texts, &weights, score_mode);
        }
    }
}

/// Use every achievable combined score as `minScore`, and the doubles just
/// below and above it, so that the early exit is exercised at its boundary.
#[track_caller]
fn assert_boundary_thresholds_agree(
    query: &str,
    key_texts: &[Vec<String>],
    weights: &[f64],
    score_mode: KeyScoreMode,
) {
    let base = with_mode(score_mode);
    for r in &reference(query, key_texts, weights, base) {
        for min_score in [r.score, r.score.next_down(), r.score.next_up()] {
            let o = Opts {
                min_score: Some(min_score),
                ..base
            };
            assert_all_agree(query, key_texts, weights, o);
        }
    }
}

#[test]
fn extreme_weights_keep_both_paths_in_agreement() {
    // Tiny, subnormal and huge weights stress the early exit's slack, which
    // is relative to the weights.
    const EXTREME: [f64; 8] = [0.0, 5e-324, 1e-310, 1e-300, 1e-9, 1.0, 1e300, 3.0];
    let mut rng = Rng(31);
    for _ in 0..400 {
        let num_keys = 1 + rng.below(4);
        let weights: Vec<f64> = (0..num_keys).map(|_| *rng.pick(&EXTREME)).collect();
        let total: f64 = weights.iter().sum();
        if total <= 0.0 || !total.is_finite() {
            continue;
        }
        let key_texts: Vec<Vec<String>> = (0..num_keys)
            .map(|_| {
                (0..4)
                    .map(|_| {
                        rng.pick(&["foo", "fo", "bar", "f_o_o", "xfoo", "foo bar"])
                            .to_string()
                    })
                    .collect()
            })
            .collect();
        for score_mode in MODES {
            assert_all_agree("foo", &key_texts, &weights, with_mode(score_mode));
            assert_boundary_thresholds_agree("foo", &key_texts, &weights, score_mode);
        }
    }
}

// ─── Score modes ────────────────────────────────────────────────────────────

fn with_mode(score_mode: KeyScoreMode) -> Opts {
    Opts {
        score_mode,
        ..Opts::default()
    }
}

fn scores(results: &[KeySearchResult]) -> Vec<(u32, f64)> {
    results.iter().map(|r| (r.index, r.score)).collect()
}

fn find(results: &[KeySearchResult], index: u32) -> &KeySearchResult {
    results
        .iter()
        .find(|r| r.index == index)
        .unwrap_or_else(|| panic!("item {index} is returned"))
}

#[test]
fn score_modes_combine_key_scores_as_documented() {
    // name, email, bio: item 0 matches only its name, exactly; item 1
    // matches its name and its email partially.
    let key_texts = columns(&[
        &["John Smith", "S. Mitchell"],
        &["john@example.com", "smitchell@example.com"],
        &["Engineer", "Designer"],
    ]);
    let weights = [2.0, 1.0, 1.0];
    let weighted = via_search_keys("smith", &key_texts, &weights, Opts::default());
    let exact = find(&weighted, 0);
    let partial = find(&weighted, 1);
    let [name, email, bio] = partial.key_scores[..] else {
        panic!("three key scores");
    };
    assert_eq!(exact.key_scores, [1.0, 0.0, 0.0]);
    assert!(name > 0.0 && name < 1.0, "{name}");
    assert!(email > 0.0 && email < 1.0, "{email}");
    assert_eq!(bio, 0.0);
    // Weighted: the exact name match only gets its share of the total weight.
    assert_eq!(exact.score, 2.0 / 4.0);
    assert_eq!(partial.score, (name * 2.0 + email) / 4.0);

    let matched = via_search_keys(
        "smith",
        &key_texts,
        &weights,
        with_mode(KeyScoreMode::Matched),
    );
    assert_eq!(find(&matched, 0).score, 1.0);
    assert_eq!(find(&matched, 1).score, (name * 2.0 + email) / 3.0);
    assert_eq!(matched[0].index, 0, "the exact match ranks first");

    let max = via_search_keys("smith", &key_texts, &weights, with_mode(KeyScoreMode::Max));
    assert_eq!(find(&max, 0).score, 1.0);
    assert_eq!(find(&max, 1).score, name.max(email));
    assert_eq!(max[0].index, 0, "the exact match ranks first");

    // keyScores do not depend on the mode.
    for results in [&matched, &max] {
        for r in results.iter() {
            assert_eq!(r.key_scores, find(&weighted, r.index).key_scores);
        }
    }
    for score_mode in MODES {
        assert_all_agree("smith", &key_texts, &weights, with_mode(score_mode));
    }
}

#[test]
fn the_default_score_mode_is_weighted() {
    assert_eq!(KeyScoreMode::default(), KeyScoreMode::Weighted);
    let key_texts = columns(&[
        &["apple pie", "apple", "pear"],
        &["fruit", "apple tart", "x"],
    ]);
    let weights = [0.494, 0.953];
    let explicit = summary(&via_search_keys(
        "apple",
        &key_texts,
        &weights,
        Opts::default(),
    ));
    assert_eq!(explicit.len(), 2);
    for options in [
        None,
        Some(SearchKeysOptions {
            max_results: None,
            min_score: None,
            is_case_sensitive: None,
            return_all_on_empty: None,
            score_mode: None,
        }),
    ] {
        let unset = search_keys_impl("apple", &key_texts, &weights, options).unwrap();
        assert_eq!(summary(&unset), explicit);
    }
}

#[test]
fn matched_mode_scores_an_item_whose_only_match_is_exact_one() {
    let key_texts = columns(&[&["zzz", "ada"], &["ada", "zzz"], &["zzz", "zzz"]]);
    let o = with_mode(KeyScoreMode::Matched);
    for weights in [[0.3, 0.7, 5.0], [1e-9, 3.0, 0.1], [0.494, 0.953, 0.137]] {
        for results in [
            via_index("ada", &key_texts, &weights, o),
            via_search_keys("ada", &key_texts, &weights, o),
        ] {
            assert_eq!(scores(&results), [(0, 1.0), (1, 1.0)], "{weights:?}");
        }
        let exact_only = Opts {
            min_score: Some(1.0),
            ..o
        };
        assert_eq!(via_index("ada", &key_texts, &weights, exact_only).len(), 2);
        assert_all_agree("ada", &key_texts, &weights, exact_only);
    }
}

#[test]
fn max_mode_uses_weights_only_to_select_keys() {
    let key_texts = columns(&[
        &["foobar", "x", "foo", "f_o_o"],
        &["x", "foo bar", "xfoo", "foo"],
    ]);
    let o = with_mode(KeyScoreMode::Max);
    let baseline = summary(&via_search_keys("foo", &key_texts, &[1.0, 1.0], o));
    assert_eq!(baseline.len(), 4);
    for weights in [[3.0, 0.1], [1e-9, 1e9], [0.494, 0.953]] {
        assert_eq!(
            summary(&via_search_keys("foo", &key_texts, &weights, o)),
            baseline,
            "{weights:?}"
        );
        assert_all_agree("foo", &key_texts, &weights, o);
    }
}

#[test]
fn zero_weight_keys_never_count_in_any_mode() {
    // Key 1 has weight 0: it is scored, but it never selects an item, never
    // counts as matched and never provides the maximum.
    let key_texts = columns(&[&["apple", "zzz", "pineapple"], &["zzz", "apple", "apple"]]);
    let weights = [1.0, 0.0];
    let partial = find(
        &via_search_keys("apple", &key_texts[..1], &[1.0], Opts::default()),
        2,
    )
    .score;
    assert!(partial > 0.0 && partial < 1.0, "{partial}");
    for score_mode in MODES {
        for results in [
            via_index("apple", &key_texts, &weights, with_mode(score_mode)),
            via_search_keys("apple", &key_texts, &weights, with_mode(score_mode)),
        ] {
            assert_eq!(scores(&results), [(0, 1.0), (2, partial)], "{score_mode:?}");
            assert_eq!(results[1].key_scores, [partial, 1.0], "{score_mode:?}");
        }
        assert_all_agree("apple", &key_texts, &weights, with_mode(score_mode));
    }
}

#[test]
fn max_mode_breaks_ties_by_the_first_key_reaching_the_maximum() {
    // Both items score 1. The tie is broken by the length of the text of the
    // first key that scores 1, then by index.
    let o = with_mode(KeyScoreMode::Max);
    let key_texts = columns(&[&["foo", "foo bar baz"], &["foo bar baz", "foo"]]);
    let results = via_search_keys("foo", &key_texts, &[1.0, 1.0], o);
    assert_eq!(scores(&results), [(0, 1.0), (1, 1.0)]);
    let swapped = columns(&[&["foo bar baz", "foo"], &["foo", "foo bar baz"]]);
    let results = via_search_keys("foo", &swapped, &[1.0, 1.0], o);
    assert_eq!(scores(&results), [(1, 1.0), (0, 1.0)]);
    assert_all_agree("foo", &key_texts, &[1.0, 1.0], o);
    assert_all_agree("foo", &swapped, &[1.0, 1.0], o);
}

#[test]
fn return_all_on_empty_scores_one_in_every_mode() {
    let key_texts = columns(&[&["a", "b"], &["c", "d"]]);
    for score_mode in MODES {
        let o = Opts {
            return_all_on_empty: true,
            ..with_mode(score_mode)
        };
        let results = via_search_keys("", &key_texts, &[1.0, 0.0], o);
        assert_eq!(scores(&results), [(0, 1.0), (1, 1.0)]);
        assert!(results.iter().all(|r| r.key_scores == [1.0, 1.0]));
        assert_all_agree("", &key_texts, &[1.0, 0.0], o);
    }
}

#[test]
fn score_mode_names() {
    for (name, mode) in [
        ("weighted", KeyScoreMode::Weighted),
        ("matched", KeyScoreMode::Matched),
        ("max", KeyScoreMode::Max),
    ] {
        assert_eq!(KeyScoreMode::from_name(name), Ok(mode));
        assert_eq!(mode.name(), name);
    }
    for name in ["", "Weighted", "MAX", "mean", " max", "matched ", "sum"] {
        assert_eq!(
            KeyScoreMode::from_name(name),
            Err(format!(
                "scoreMode must be \"weighted\", \"matched\" or \"max\", got {name:?}"
            )),
        );
    }
}

#[test]
fn zero_weight_keys_are_scored_but_do_not_select_items() {
    let key_texts = columns(&[&["apple", "zzz", "apple"], &["zzz", "apple", "apple"]]);
    let weights = [1.0, 0.0];
    for (name, results) in [
        (
            "index",
            via_index("apple", &key_texts, &weights, Opts::default()),
        ),
        (
            "searchKeys",
            via_search_keys("apple", &key_texts, &weights, Opts::default()),
        ),
    ] {
        let indices: Vec<u32> = results.iter().map(|r| r.index).collect();
        assert_eq!(
            indices,
            [0, 2],
            "{name}: item 1 only matches a zero-weight key"
        );
        assert_eq!(results[0].key_scores, [1.0, 0.0], "{name}");
        assert_eq!(
            results[1].key_scores,
            [1.0, 1.0],
            "{name}: zero-weight key score"
        );
        assert!(results.iter().all(|r| r.score > 0.0), "{name}");
    }
    assert_all_agree("apple", &key_texts, &weights, Opts::default());
}

#[test]
fn case_folded_and_normalized_characters_match_in_both_paths() {
    let key_texts = columns(&[&["Łódź", "\u{212A}elvin", "ŠKODA", "other"]]);
    let weights = [1.0];
    for query in ["łódź", "kelvin", "škoda"] {
        let expected: Vec<u32> = search_impl(
            query.to_string(),
            key_texts[0].clone(),
            None,
            None,
            false,
            CaseMatching::Smart,
        )
        .iter()
        .map(|r| r.index)
        .collect();
        assert!(!expected.is_empty(), "search() matches {query:?}");
        let index: Vec<u32> = via_index(query, &key_texts, &weights, Opts::default())
            .iter()
            .map(|r| r.index)
            .collect();
        assert_eq!(index, expected, "index for {query:?}");
        assert_all_agree(query, &key_texts, &weights, Opts::default());
    }
}

#[test]
fn ties_prefer_the_shorter_best_matching_key_like_search() {
    let items = strings(&["foobar", "foo", "foo bar baz", "xfoo"]);
    let expected: Vec<(u32, u64)> = search_impl(
        "foo".to_string(),
        items.clone(),
        None,
        None,
        false,
        CaseMatching::Smart,
    )
    .iter()
    .map(|r| (r.index, r.score.to_bits()))
    .collect();
    assert_eq!(expected[0].0, 1, "search() puts the shorter item first");
    let key_texts = vec![items];
    for results in [
        via_index("foo", &key_texts, &[1.0], Opts::default()),
        via_search_keys("foo", &key_texts, &[1.0], Opts::default()),
    ] {
        let got: Vec<(u32, u64)> = results
            .iter()
            .map(|r| (r.index, r.score.to_bits()))
            .collect();
        assert_eq!(got, expected);
    }

    // With several keys, the key contributing most decides the length.
    let key_texts = columns(&[&["zzzz", "zzzz"], &["foo long text", "foo"]]);
    let results = via_search_keys("foo", &key_texts, &[1.0, 1.0], Opts::default());
    assert_eq!(results.iter().map(|r| r.index).collect::<Vec<_>>(), [1, 0]);
    assert_all_agree("foo", &key_texts, &[1.0, 1.0], Opts::default());
}

#[test]
fn single_key_results_equal_search_results() {
    let mut rng = Rng(11);
    for _ in 0..200 {
        let items: Vec<String> = (0..1 + rng.below(12)).map(|_| rng.item()).collect();
        let query = rng.query(&items);
        let o = rng.opts(items.len());
        let weight = *rng.pick(&[1.0, 2.0, 0.5, 8.0]);
        let expected: Vec<(u32, u64)> = search_impl(
            query.clone(),
            items.clone(),
            o.max_results,
            o.min_score,
            false,
            o.case_matching(),
        )
        .iter()
        // Multi-key search excludes items with a combined score of 0, which
        // search() returns for queries made only of exclusions (`!foo`).
        .filter(|r| r.score > 0.0)
        .map(|r| (r.index, r.score.to_bits()))
        .collect();
        let key_texts = vec![items];
        for results in [
            via_index(
                &query,
                &key_texts,
                &[weight],
                Opts {
                    return_all_on_empty: false,
                    ..o
                },
            ),
            via_search_keys(
                &query,
                &key_texts,
                &[weight],
                Opts {
                    return_all_on_empty: false,
                    ..o
                },
            ),
        ] {
            let got: Vec<(u32, u64)> = results
                .iter()
                .map(|r| (r.index, r.score.to_bits()))
                .collect();
            assert_eq!(
                got, expected,
                "query={query:?} items={key_texts:?} opts={o:?}"
            );
        }
    }
}

#[test]
fn unicode_whitespace_separates_query_terms() {
    let key_texts = columns(&[&["foo bar", "foo", "bar"], &["x", "y", "z"]]);
    let weights = [1.0, 1.0];
    let plain = summary(&via_search_keys(
        "foo bar",
        &key_texts,
        &weights,
        Opts::default(),
    ));
    assert_eq!(plain.len(), 1);
    for query in ["foo\u{3000}bar", "foo\u{a0}bar", "foo\tbar", "foo\nbar"] {
        assert_eq!(
            summary(&via_search_keys(
                query,
                &key_texts,
                &weights,
                Opts::default()
            )),
            plain,
            "{query:?}"
        );
        assert_all_agree(query, &key_texts, &weights, Opts::default());
    }
}

#[test]
fn syntax_only_queries_are_empty() {
    let key_texts = columns(&[&["a^b", "c"], &["!", "$"]]);
    let weights = [1.0, 1.0];
    for query in ["^", "!", "$", "'", "^$", " \u{3000} "] {
        assert!(via_search_keys(query, &key_texts, &weights, Opts::default()).is_empty());
        assert!(via_index(query, &key_texts, &weights, Opts::default()).is_empty());
        let all = Opts {
            return_all_on_empty: true,
            ..Opts::default()
        };
        assert_eq!(
            via_search_keys(query, &key_texts, &weights, all).len(),
            2,
            "{query:?}"
        );
        assert_eq!(
            via_index(query, &key_texts, &weights, all).len(),
            2,
            "{query:?}"
        );
    }
}

#[test]
fn bar_dollar_scores_one_on_an_exact_match() {
    // The maximum score is computed per term, so anchored terms can reach 1.
    let key_texts = columns(&[&["bar", "foobar"]]);
    let results = via_search_keys("bar$", &key_texts, &[1.0], Opts::default());
    assert_eq!(results[0].index, 0);
    assert_eq!(results[0].score, 1.0);
    assert_all_agree("bar$", &key_texts, &[1.0], Opts::default());
}

#[test]
fn overlong_terms_match_nothing() {
    let long = "a".repeat(3000);
    let key_texts = vec![vec![long.clone()]];
    assert!(via_search_keys(&long, &key_texts, &[1.0], Opts::default()).is_empty());
    assert!(via_index(&long, &key_texts, &[1.0], Opts::default()).is_empty());
}

#[test]
fn invalid_input_is_rejected_with_the_constructor_messages() {
    let cases: Vec<(Vec<Vec<String>>, Vec<f64>)> = vec![
        (columns(&[&["a", "b"], &["c"]]), vec![1.0, 1.0]),
        (columns(&[&["a"]]), vec![1.0, 2.0]),
        (columns(&[&["a"], &["b"]]), vec![1.0]),
        (columns(&[&["a"]]), vec![-1.0]),
        (columns(&[&["a"]]), vec![f64::NAN]),
        (columns(&[&["a"]]), vec![f64::INFINITY]),
        (columns(&[&["a"], &["b"]]), vec![0.0, 0.0]),
        (columns(&[&["a"], &["b"]]), vec![f64::MAX, f64::MAX]),
        (Vec::new(), Vec::new()),
    ];
    for (key_texts, weights) in cases {
        let constructor = KeyedFuzzyIndexCore::new(key_texts.clone(), weights.clone())
            .err()
            .unwrap_or_else(|| panic!("constructor accepts {key_texts:?} {weights:?}"));
        for query in ["a", ""] {
            let keys = search_keys_impl(query, &key_texts, &weights, None)
                .err()
                .unwrap_or_else(|| panic!("searchKeys accepts {key_texts:?} {weights:?}"));
            assert_eq!(keys, constructor, "{key_texts:?} {weights:?}");
        }
    }
}

#[test]
fn weights_summing_to_infinity_are_rejected() {
    let err = KeyedFuzzyIndexCore::new(columns(&[&["a"], &["a"]]), vec![f64::MAX, f64::MAX])
        .err()
        .expect("overflowing weights are rejected");
    assert!(err.contains("finite"), "{err}");
}

#[test]
fn empty_items_return_nothing() {
    let key_texts: Vec<Vec<String>> = vec![Vec::new(), Vec::new()];
    assert!(via_search_keys("a", &key_texts, &[1.0, 1.0], Opts::default()).is_empty());
    assert!(via_index("a", &key_texts, &[1.0, 1.0], Opts::default()).is_empty());
}

#[test]
fn index_mutations_keep_parity() {
    let mut rng = Rng(23);
    for _ in 0..60 {
        let num_keys = 1 + rng.below(3);
        let weights: Vec<f64> = loop {
            let w: Vec<f64> = (0..num_keys).map(|_| rng.weight()).collect();
            if w.iter().sum::<f64>() > 0.0 {
                break w;
            }
        };
        let mut key_texts: Vec<Vec<String>> = (0..num_keys)
            .map(|_| (0..rng.below(8)).map(|_| rng.item()).collect())
            .collect();
        let n = key_texts.iter().map(Vec::len).min().unwrap_or(0);
        key_texts.iter_mut().for_each(|c| c.truncate(n));
        let mut index = KeyedFuzzyIndexCore::new(key_texts.clone(), weights.clone()).unwrap();
        for _ in 0..6 {
            if rng.chance(50) || key_texts[0].is_empty() {
                let row: Vec<String> = (0..num_keys).map(|_| rng.item()).collect();
                for (col, value) in key_texts.iter_mut().zip(&row) {
                    col.push(value.clone());
                }
                index.add(row).unwrap();
            } else {
                let i = rng.below(key_texts[0].len());
                for col in &mut key_texts {
                    col.swap_remove(i);
                }
                assert!(index.remove(i as u32));
            }
            let flat: Vec<String> = key_texts.iter().flatten().cloned().collect();
            let query = rng.query(&flat);
            let o = rng.opts(key_texts[0].len());
            let got = summary(&index.search(
                &query,
                o.max_results,
                o.min_score,
                o.case_matching(),
                o.return_all_on_empty,
                o.score_mode,
            ));
            assert_eq!(
                got,
                summary(&reference(&query, &key_texts, &weights, o)),
                "query={query:?} key_texts={key_texts:?} weights={weights:?} opts={o:?}"
            );
        }
    }
}

// ─── Differential test ──────────────────────────────────────────────────────

#[test]
fn index_search_keys_and_reference_agree_on_random_input() {
    for seed in 0..1500u64 {
        let mut rng = Rng(seed);
        let num_keys = 1 + rng.below(4);
        let num_items = rng.below(14);
        let key_texts: Vec<Vec<String>> = (0..num_keys)
            .map(|_| (0..num_items).map(|_| rng.item()).collect())
            .collect();
        let weights: Vec<f64> = (0..num_keys).map(|_| rng.weight()).collect();
        if weights.iter().sum::<f64>() <= 0.0 {
            continue;
        }
        let flat: Vec<String> = key_texts.iter().flatten().cloned().collect();
        for _ in 0..4 {
            let query = rng.query(&flat);
            let base = rng.opts(num_items);
            let boundary = rng.chance(30);
            let nudge = rng.below(3);
            for score_mode in MODES {
                let mut o = Opts { score_mode, ..base };
                if boundary {
                    // Thresholds equal to (or one ULP away from) an
                    // achievable score exercise the early exit at its
                    // boundary.
                    let all = reference(&query, &key_texts, &weights, with_mode(score_mode));
                    if !all.is_empty() {
                        let score = all[rng.below(all.len())].score;
                        o.min_score = Some(match nudge {
                            0 => score,
                            1 => score.next_down(),
                            _ => score.next_up(),
                        });
                    }
                }
                assert_all_agree(&query, &key_texts, &weights, o);
            }
        }
    }
}

/// SplitMix64: tiny, deterministic, good enough for test-case generation.
struct Rng(u64);

const ITEM_PIECES: &[&str] = &[
    "a",
    "b",
    "c",
    "e",
    "f",
    "o",
    "r",
    "s",
    "x",
    "A",
    "B",
    "F",
    "O",
    "S",
    "_",
    "/",
    ".",
    "-",
    " ",
    " ",
    "$",
    "^",
    "!",
    "'",
    "\\",
    "0",
    "1",
    "é",
    "É",
    "e\u{301}",
    "ł",
    "Ł",
    "ó",
    "Ó",
    "ź",
    "Ź",
    "д",
    "Д",
    "м",
    "М",
    "\u{212A}",
    "k",
    "K",
    "ſ",
    "ß",
    "İ",
    "ı",
    "Σ",
    "σ",
    "ς",
    "Ａ",
    "東",
    "京",
    "\u{3000}",
    "\u{a0}",
    "\t",
    "👍🏽",
    "foo",
    "bar",
    "foobar",
    "Łódź",
    "Москва",
];

const QUERY_PIECES: &[&str] = &[
    " ", "\u{3000}", "\u{a0}", "\t", "\n", "^", "$", "!", "'", "\\", "\\ ",
];

impl Rng {
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }

    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }

    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }

    fn weight(&mut self) -> f64 {
        match self.below(6) {
            0 => 0.0,
            1 => 1.0,
            2 => (self.below(1000) as f64) / 1000.0,
            3 => *self.pick(&[0.1, 0.2, 0.3, 0.7, 1e-9, 3.0]),
            _ => (self.below(10_000) as f64 + 1.0) / 997.0,
        }
    }

    fn item(&mut self) -> String {
        (0..self.below(6))
            .map(|_| *self.pick(ITEM_PIECES))
            .collect()
    }

    /// A query: usually a (mangled) slice of an existing text, sometimes
    /// random pieces, with syntax and whitespace sprinkled in.
    fn query(&mut self, texts: &[String]) -> String {
        let mut q = String::new();
        if !texts.is_empty() && self.chance(75) {
            let chars: Vec<char> = self.pick(texts).chars().collect();
            if !chars.is_empty() {
                let start = self.below(chars.len());
                let len = 1 + self.below(chars.len() - start);
                for &c in &chars[start..start + len] {
                    if !self.chance(15) {
                        q.push(c);
                    }
                }
            }
        } else {
            for _ in 0..self.below(4) {
                q.push_str(self.pick::<&str>(ITEM_PIECES));
            }
        }
        if self.chance(25) {
            let piece = *self.pick(QUERY_PIECES);
            let at = self.below(q.chars().count() + 1);
            let byte = q.char_indices().nth(at).map_or(q.len(), |(b, _)| b);
            q.insert_str(byte, piece);
        }
        if self.chance(10) {
            q = q.to_uppercase();
        }
        q
    }

    fn opts(&mut self, num_items: usize) -> Opts {
        Opts {
            max_results: self.chance(40).then(|| self.below(num_items + 2) as u32),
            min_score: self
                .chance(30)
                .then(|| *self.pick(&[0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0])),
            case_sensitive: self.chance(20),
            return_all_on_empty: self.chance(20),
            score_mode: *self.pick(&MODES),
        }
    }
}
