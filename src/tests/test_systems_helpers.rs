use crate::systems::helpers::pairs;

#[test]
fn test_pairs_collects_str_slices() {
    let out = pairs(["BTCUSDT", "ETHUSDT"]);
    assert_eq!(out, vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]);
}

#[test]
fn test_pairs_collects_owned_strings() {
    let out = pairs(vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]);
    assert_eq!(out, vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()]);
}

#[test]
fn test_pairs_preserves_order() {
    let out = pairs(["SOLUSDT", "BTCUSDT", "ETHUSDT"]);
    assert_eq!(
        out,
        vec![
            "SOLUSDT".to_string(),
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string()
        ]
    );
}

#[test]
fn test_pairs_allows_empty_iterator() {
    let out = pairs(std::iter::empty::<&str>());
    assert!(out.is_empty());
}
