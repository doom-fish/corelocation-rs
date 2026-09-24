use corelocation::prelude::*;

#[test]
fn authorization_types_and_snapshot_smoke() -> Result<(), Box<dyn std::error::Error>> {
    assert!(AuthorizationStatus::AuthorizedAlways.is_authorized());
    assert_eq!(
        AccuracyAuthorization::from_raw(0),
        Some(AccuracyAuthorization::FullAccuracy)
    );

    let manager = LocationManager::new()?;
    let (status, snapshot) = loop {
        let status = manager.authorization_status();
        let snapshot = manager.authorization()?;
        if manager.authorization_status() == status {
            break (status, snapshot);
        }
    };
    assert_eq!(snapshot.status, status);
    Ok(())
}
