use corelocation::prelude::*;

#[test]
fn beacon_region_snapshot_condition_and_peripheral_data() -> Result<(), Box<dyn std::error::Error>>
{
    let region = BeaconRegion::with_major_minor(
        "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE",
        1,
        2,
        "demo-beacon",
    )?;
    let snapshot = region.snapshot()?;
    let condition = region.beacon_identity_condition()?;
    let peripheral_data = region.peripheral_data(Some(-59))?;

    let constraint = BeaconIdentityConstraint::with_major_minor(
        "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE",
        1,
        2,
    )?;
    let legacy_region = BeaconRegion::from_constraint(&constraint, "legacy-beacon")?;
    let legacy_snapshot = legacy_region.snapshot()?;

    assert_eq!(snapshot.identifier, "demo-beacon");
    assert_eq!(condition.major, Some(1));
    assert_eq!(legacy_snapshot.identifier, "legacy-beacon");
    assert!(peripheral_data.is_object());
    Ok(())
}
