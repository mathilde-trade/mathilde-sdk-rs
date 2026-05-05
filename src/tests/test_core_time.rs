use crate::core::error::SdkError;
use crate::core::time::{TimeInput, parse_utc_string_to_ms, validate_utc_ms};

#[test]
fn test_time_input_ms_passes_through_unchanged() {
    let value = 1_775_646_896_123i64;
    let out = TimeInput::Ms(value).to_utc_ms().expect("valid ms");
    assert_eq!(out, value);
}

#[test]
fn test_invalid_ms_is_rejected() {
    let err = validate_utc_ms(i64::MAX).expect_err("expected invalid ms");
    match err {
        SdkError::InvalidTimeInput { .. } => {}
        other => panic!("expected InvalidTimeInput, got {other:?}"),
    }
}

#[test]
fn test_rfc3339_utc_parses_to_ms() {
    let out = parse_utc_string_to_ms("2026-04-08T12:34:56Z").expect("valid utc");
    assert_eq!(out, 1_775_651_696_000);
}

#[test]
fn test_rfc3339_utc_fractional_parses_to_ms() {
    let out = parse_utc_string_to_ms("2026-04-08T12:34:56.123Z").expect("valid utc");
    assert_eq!(out, 1_775_651_696_123);
}

#[test]
fn test_compact_utc_minute_form_parses_to_ms() {
    let out = parse_utc_string_to_ms("2026-04-08:12:34").expect("valid compact utc");
    assert_eq!(out, 1_775_651_640_000);
}

#[test]
fn test_compact_utc_second_form_parses_to_ms() {
    let out = parse_utc_string_to_ms("2026-04-08:12:34:56").expect("valid compact utc");
    assert_eq!(out, 1_775_651_696_000);
}

#[test]
fn test_invalid_string_is_rejected() {
    let err = TimeInput::Utc("2026-04-08 12:34".to_string())
        .to_utc_ms()
        .expect_err("expected invalid string");
    match err {
        SdkError::InvalidTimeInput { .. } => {}
        other => panic!("expected InvalidTimeInput, got {other:?}"),
    }
}

#[test]
fn test_rfc3339_offset_form_is_rejected() {
    let err = parse_utc_string_to_ms("2026-04-08T12:34:56+02:00")
        .expect_err("expected offset-form rejection");
    match err {
        SdkError::InvalidTimeInput { message } => {
            assert!(message.contains("unsupported utc time input"));
        }
        other => panic!("expected InvalidTimeInput, got {other:?}"),
    }
}

#[test]
fn test_rfc3339_zero_offset_form_is_rejected_without_z() {
    let err = parse_utc_string_to_ms("2026-04-08T12:34:56+00:00")
        .expect_err("expected zero-offset rejection");
    match err {
        SdkError::InvalidTimeInput { message } => {
            assert!(message.contains("unsupported utc time input"));
        }
        other => panic!("expected InvalidTimeInput, got {other:?}"),
    }
}
