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

#[test]
fn delegate_callbacks_arrive_without_a_run_loop_on_the_creating_thread() {
    use std::sync::mpsc::{self, TryRecvError};
    use std::thread;
    use std::time::Duration;

    let (sender, receiver) = mpsc::channel();
    let creator = thread::current().id();
    let manager = LocationManager::with_callbacks(
        LocationManagerCallbacks::new().on_authorization_details(move |snapshot| {
            let _ = sender.send((snapshot, thread::current().id()));
        }),
    )
    .expect("LocationManager::with_callbacks");

    let (snapshot, delivery_thread) = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("the initial authorization callback must arrive");
    assert_ne!(delivery_thread, creator);
    assert_eq!(snapshot.status, manager.authorization_status());

    drop(manager);
    let _earlier: Vec<_> = receiver.try_iter().collect();
    assert!(matches!(
        receiver.try_recv(),
        Err(TryRecvError::Disconnected)
    ));
}
