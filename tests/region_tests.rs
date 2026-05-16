use corelocation::prelude::*;

#[test]
fn circular_region_snapshot_and_contains() -> Result<(), Box<dyn std::error::Error>> {
    let region = CircularRegion::new(
        Coordinate::new(37.3349, -122.0090),
        250.0,
        "apple-park-circle",
    )?;
    region.set_notify_on_entry(false);

    let snapshot = region.snapshot()?;
    assert_eq!(snapshot.identifier, "apple-park-circle");
    assert!(region.contains_coordinate(Coordinate::new(37.3349, -122.0090)));
    Ok(())
}
