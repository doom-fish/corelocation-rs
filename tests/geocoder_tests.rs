use corelocation::prelude::*;

#[test]
fn geocoder_creation_and_extended_methods_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let geocoder = Geocoder::new()?;
    assert!(!geocoder.is_geocoding());
    geocoder.cancel();

    let region = CircularRegion::new(
        Coordinate::new(37.3349, -122.0090),
        2_000.0,
        "apple-park-search",
    )?;
    let _ = geocoder.geocode_address_string_in_region("Apple Park", Some(&region));
    let _ = geocoder.geocode_postal_address(&PostalAddress::default());
    Ok(())
}
