use corelocation::prelude::*;

fn main() {
    let floor = Floor { level: 4 };
    let source_information = LocationSourceInformation {
        is_simulated_by_software: true,
        is_produced_by_accessory: false,
    };
    let details = LocationDetails {
        location: Location::from_coordinate(Coordinate::new(37.3349, -122.0090)),
        ellipsoidal_altitude: Some(12.0),
        course_accuracy: Some(1.0),
        speed_accuracy: Some(0.5),
        floor: Some(floor.clone()),
        source_information: Some(source_information.clone()),
    };

    println!("floor = {floor:?}");
    println!("source_information = {source_information:?}");
    println!("location_details = {details:?}");
}
