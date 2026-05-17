use corelocation::error::{alternate_region_key, error_domain, CLErrorCode};
use corelocation::LocationManagerErrorInfo;

#[test]
fn corelocation_error_codes_and_constants_round_trip() {
    assert_eq!(CLErrorCode::from_raw(0), Some(CLErrorCode::LocationUnknown));
    assert_eq!(CLErrorCode::from_raw(1), Some(CLErrorCode::Denied));
    assert_eq!(CLErrorCode::from_raw(19), Some(CLErrorCode::HistoricalLocationError));
    assert_eq!(CLErrorCode::from_raw(999), None);

    assert!(!error_domain().is_empty());
    assert!(!alternate_region_key().is_empty());

    let info = LocationManagerErrorInfo {
        domain: error_domain().into(),
        code: i32::from(CLErrorCode::Denied),
        message: "denied".into(),
    };
    assert_eq!(info.error_code(), Some(CLErrorCode::Denied));
}
