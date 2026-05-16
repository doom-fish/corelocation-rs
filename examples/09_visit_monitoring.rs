use corelocation::prelude::*;

fn main() {
    let manager = LocationManager::new().expect("failed to create location manager");
    manager.start_monitoring_visits();
    manager.stop_monitoring_visits();

    let visit = Visit {
        arrival_date: 0.0,
        departure_date: 60.0,
        coordinate: Coordinate::new(37.3349, -122.0090),
        horizontal_accuracy: 25.0,
    };
    println!("visit = {visit:?}");
}
