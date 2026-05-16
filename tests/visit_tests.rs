use corelocation::prelude::*;

#[test]
fn visit_round_trip_and_manager_methods_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let visit = Visit {
        arrival_date: 0.0,
        departure_date: 60.0,
        coordinate: Coordinate::new(37.3349, -122.0090),
        horizontal_accuracy: 25.0,
    };
    let json = serde_json::to_string(&visit)?;
    let decoded: Visit = serde_json::from_str(&json)?;
    assert_eq!(decoded, visit);

    let manager = LocationManager::new()?;
    manager.start_monitoring_visits();
    manager.stop_monitoring_visits();
    Ok(())
}
