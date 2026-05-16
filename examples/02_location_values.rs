use corelocation::prelude::*;

fn main() {
    let apple_park = Location::from_coordinate(Coordinate::new(37.3349, -122.0090));
    let ferry_building = Location::from_coordinate(Coordinate::new(37.7955, -122.3937));
    let distance_m = apple_park.distance_to(&ferry_building);

    let detailed = LocationDetails {
        location: apple_park.clone(),
        ellipsoidal_altitude: Some(18.0),
        course_accuracy: Some(0.5),
        speed_accuracy: Some(0.25),
        floor: Some(Floor { level: 1 }),
        source_information: Some(LocationSourceInformation {
            is_simulated_by_software: false,
            is_produced_by_accessory: false,
        }),
    };

    println!("apple_park_valid = {}", apple_park.coordinate.is_valid());
    println!("distance_m = {distance_m:.0}");
    println!("detailed_location = {detailed:?}");
}
