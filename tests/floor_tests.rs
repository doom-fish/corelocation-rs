use corelocation::prelude::*;

#[test]
fn floor_and_source_information_round_trip() -> Result<(), Box<dyn std::error::Error>> {
    let floor = Floor { level: -1 };
    let source_information = LocationSourceInformation {
        is_simulated_by_software: true,
        is_produced_by_accessory: false,
    };
    let json = serde_json::to_string(&(floor.clone(), source_information.clone()))?;
    let decoded: (Floor, LocationSourceInformation) = serde_json::from_str(&json)?;
    assert_eq!(decoded.0, floor);
    assert_eq!(decoded.1, source_information);
    Ok(())
}
