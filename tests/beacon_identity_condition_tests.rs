use corelocation::prelude::*;

#[test]
fn beacon_identity_condition_snapshot_smoke() -> Result<(), Box<dyn std::error::Error>> {
    match BeaconIdentityCondition::with_major_minor("AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE", 1, 2) {
        Ok(condition) => {
            let snapshot = condition.snapshot()?;
            assert_eq!(snapshot.major, Some(1));
            assert_eq!(snapshot.minor, Some(2));
        }
        Err(error) => {
            println!("beacon identity condition unavailable: {error}");
        }
    }
    Ok(())
}
