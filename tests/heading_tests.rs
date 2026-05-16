use corelocation::prelude::*;

#[test]
fn heading_configuration_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let manager = LocationManager::new()?;
    manager.set_heading_filter(corelocation::manager::HEADING_FILTER_NONE);
    manager.set_heading_orientation(DeviceOrientation::Portrait);

    assert!(
        (manager.heading_filter() - corelocation::manager::HEADING_FILTER_NONE).abs()
            < f64::EPSILON
    );
    assert_eq!(manager.heading_orientation(), DeviceOrientation::Portrait);
    Ok(())
}
