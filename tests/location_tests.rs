use corelocation::prelude::*;

#[test]
fn coordinate_validation_and_distance_work() {
    let apple_park = Location::from_coordinate(Coordinate::new(37.3349, -122.0090));
    let ferry_building = Location::from_coordinate(Coordinate::new(37.7955, -122.3937));

    assert!(apple_park.coordinate.is_valid());
    assert!(apple_park.distance_to(&ferry_building) > 1_000.0);
}

#[test]
fn location_details_json_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let details = LocationDetails {
        location: Location::from_coordinate(Coordinate::new(37.3349, -122.0090)),
        ellipsoidal_altitude: Some(12.0),
        course_accuracy: Some(0.5),
        speed_accuracy: Some(0.25),
        floor: Some(Floor { level: 3 }),
        source_information: Some(LocationSourceInformation {
            is_simulated_by_software: false,
            is_produced_by_accessory: true,
        }),
    };
    let json = serde_json::to_string(&details)?;
    let decoded: LocationDetails = serde_json::from_str(&json)?;
    assert_eq!(decoded, details);
    Ok(())
}
