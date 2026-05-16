use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let region = CircularRegion::new(
        Coordinate::new(37.3349, -122.0090),
        250.0,
        "apple-park-circle",
    )?;
    region.set_notify_on_entry(false);

    let snapshot = region.snapshot()?;
    let contains = region.contains_coordinate(Coordinate::new(37.3349, -122.0090));

    println!(
        "circular_region_monitoring_available = {}",
        LocationManager::circular_region_monitoring_available()
    );
    println!("region_snapshot = {snapshot:?}");
    println!("contains_center = {contains}");
    Ok(())
}
