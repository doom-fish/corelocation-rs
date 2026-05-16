use corelocation::prelude::*;

#[test]
fn location_manager_smoke_and_configuration_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let manager = LocationManager::new()?;
    manager.set_desired_accuracy(corelocation::manager::LOCATION_ACCURACY_HUNDRED_METERS);
    manager.set_distance_filter(corelocation::manager::DISTANCE_FILTER_NONE);
    manager.set_activity_type(ActivityType::Fitness);
    manager.set_heading_filter(corelocation::manager::HEADING_FILTER_NONE);
    manager.set_heading_orientation(DeviceOrientation::Portrait);
    manager.set_pauses_location_updates_automatically(false);
    manager.set_allows_background_location_updates(false);

    assert_eq!(manager.activity_type(), ActivityType::Fitness);
    assert_eq!(manager.heading_orientation(), DeviceOrientation::Portrait);
    assert!(
        (manager.distance_filter() - corelocation::manager::DISTANCE_FILTER_NONE).abs()
            < f64::EPSILON
    );
    assert!(!manager.pauses_location_updates_automatically());
    assert!(!manager.allows_background_location_updates());
    assert_eq!(
        manager.authorization()?.status,
        manager.authorization_status()
    );
    Ok(())
}
