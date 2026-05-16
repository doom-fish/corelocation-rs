use corelocation::prelude::*;

#[test]
fn authorization_types_and_snapshot_smoke() -> Result<(), Box<dyn std::error::Error>> {
    assert!(AuthorizationStatus::AuthorizedAlways.is_authorized());
    assert_eq!(
        AccuracyAuthorization::from_raw(0),
        Some(AccuracyAuthorization::FullAccuracy)
    );

    let manager = LocationManager::new()?;
    let snapshot = manager.authorization()?;
    assert_eq!(snapshot.status, manager.authorization_status());
    Ok(())
}
