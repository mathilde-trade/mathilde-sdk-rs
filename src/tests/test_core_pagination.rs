use crate::core::error::SdkError;
use crate::core::pagination::{PaginationAdvance, PaginationState, require_explicit_close_end};

#[test]
fn test_pagination_state_detects_repeated_cursor() {
    let mut state = PaginationState::new();

    let first = state
        .advance(Some("cursor-1"), false)
        .expect("first page advances");
    assert_eq!(
        first,
        PaginationAdvance::Continue {
            cursor: "cursor-1".to_string(),
        }
    );

    let err = state
        .advance(Some("cursor-1"), false)
        .expect_err("repeated cursor must fail closed");

    match err {
        SdkError::ContractDrift { message } => {
            assert!(message.contains("cursor repeated without progress"));
            assert!(message.contains("cursor-1"));
        }
        other => panic!("expected ContractDrift, got {other:?}"),
    }
}

#[test]
fn test_pagination_state_stops_on_done() {
    let mut state = PaginationState::new();

    let out = state
        .advance(Some("cursor-ignored"), true)
        .expect("done page must finish traversal");

    assert_eq!(out, PaginationAdvance::Finished);
    assert_eq!(state.pages_fetched(), 1);
}

#[test]
fn test_pagination_state_stops_on_null_next_cursor() {
    let mut state = PaginationState::new();

    let out = state
        .advance(None, false)
        .expect("missing cursor without done should still finish");

    assert_eq!(out, PaginationAdvance::Finished);
    assert_eq!(state.pages_fetched(), 1);
}

#[test]
fn test_require_explicit_close_end_rejects_open_ended_search_like_traversal() {
    let err = require_explicit_close_end::<i64>(None, "search")
        .expect_err("open-ended traversal must fail closed");

    match err {
        SdkError::UnsupportedOrUnprovedUsage { message } => {
            assert_eq!(message, "search traversal requires explicit close_end");
        }
        other => panic!("expected UnsupportedOrUnprovedUsage, got {other:?}"),
    }
}
