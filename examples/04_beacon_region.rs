use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let region = BeaconRegion::with_major_minor(
        "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE",
        1,
        2,
        "demo-beacon",
    )?;
    region.set_notify_entry_state_on_display(true);

    println!(
        "beacon_region_monitoring_available = {}",
        LocationManager::beacon_region_monitoring_available()
    );
    println!("beacon_region = {:?}", region.snapshot()?);
    println!(
        "beacon_condition = {:?}",
        region.beacon_identity_condition()?
    );
    println!("peripheral_data = {:?}", region.peripheral_data(Some(-59))?);
    Ok(())
}
