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

    assert_eq!(snapshot.identifier, "demo-beacon");
    assert_eq!(condition.major, Some(1));
    assert!(peripheral_data.is_object());
    Ok(())
}
