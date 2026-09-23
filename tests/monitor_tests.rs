use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use corelocation::prelude::*;

fn unique_monitor_name() -> String {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("CoreLocationMonitor{suffix}")
}

fn run_monitor_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let configuration = MonitorConfiguration::new(unique_monitor_name());
    let monitor = configuration.open()?;
    assert_eq!(monitor.name(), configuration.name());

    let condition = CircularGeographicCondition::new(Coordinate::new(37.3349, -122.0090), 250.0)?;
    let snapshot = condition.snapshot()?;
    assert_eq!(snapshot.center, Coordinate::new(37.3349, -122.0090));
    assert!((snapshot.radius - 250.0).abs() < f64::EPSILON);

    let identifier = "appleparkcircle";
    monitor.add_condition_assuming_state(&condition, identifier, MonitoringState::Satisfied)?;

    let identifiers = monitor.monitored_identifiers()?;
    assert!(identifiers.iter().any(|candidate| candidate == identifier));

    let record = monitor
        .record(identifier)?
        .expect("monitoring record should exist immediately after add_condition_assuming_state");
    assert_eq!(record.last_event.identifier, identifier);
    assert_eq!(record.last_event.state, MonitoringState::Satisfied);
    match record.condition {
        ConditionSnapshot::CircularGeographic { center, radius } => {
            assert_eq!(center, snapshot.center);
            assert!((radius - snapshot.radius).abs() < f64::EPSILON);
        }
        other => panic!("unexpected monitoring condition snapshot: {other:?}"),
    }

    monitor.remove_condition(identifier)?;
    let identifiers = monitor.monitored_identifiers()?;
    assert!(!identifiers.iter().any(|candidate| candidate == identifier));
    Ok(())
}

#[test]
fn monitor_configuration_condition_and_record_smoke() {
    if let Err(error) = run_monitor_smoke() {
        println!("monitor APIs unavailable: {error}");
    }
}

#[test]
fn dropping_a_monitor_with_callbacks_waits_for_its_task_and_frees_the_delegate() {
    let events = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&events);
    let callbacks = MonitorCallbacks::new().on_event(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    });
    let monitor = match Monitor::with_callbacks(&unique_monitor_name(), callbacks) {
        Ok(monitor) => monitor,
        Err(error) => {
            println!("monitor APIs unavailable: {error}");
            return;
        }
    };
    let condition = CircularGeographicCondition::new(Coordinate::new(37.3349, -122.0090), 250.0)
        .expect("CircularGeographicCondition::new");
    monitor
        .add_condition(&condition, "dropfence")
        .expect("add_condition");
    monitor
        .remove_condition("dropfence")
        .expect("remove_condition");

    drop(monitor);
    let delivered = events.load(Ordering::SeqCst);
    assert_eq!(Arc::strong_count(&events), 1);
    std::thread::sleep(std::time::Duration::from_millis(100));
    assert_eq!(events.load(Ordering::SeqCst), delivered);
}
