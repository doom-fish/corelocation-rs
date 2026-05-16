use corelocation::manager::HEADING_FILTER_NONE;
use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = LocationManager::new()?;
    manager.set_heading_filter(HEADING_FILTER_NONE);
    manager.set_heading_orientation(DeviceOrientation::Portrait);

    println!(
        "heading_available = {}",
        LocationManager::heading_available()
    );
    println!("heading_filter = {}", manager.heading_filter());
    println!("heading_orientation = {:?}", manager.heading_orientation());
    Ok(())
}
