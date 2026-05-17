use corelocation::prelude::*;

#[test]
fn geocoder_creation_and_extended_methods_smoke() -> Result<(), Box<dyn std::error::Error>> {
    let geocoder = Geocoder::new()?;
    assert!(!geocoder.is_geocoding());
    geocoder.cancel();

    let placemark = Placemark {
        name: Some("Apple Park".into()),
        thoroughfare: Some("Apple Park Way".into()),
        sub_thoroughfare: Some("1".into()),
        locality: Some("Cupertino".into()),
        sub_locality: None,
        administrative_area: Some("CA".into()),
        sub_administrative_area: Some("Santa Clara".into()),
        postal_code: Some("95014".into()),
        iso_country_code: Some("US".into()),
        country: Some("United States".into()),
        postal_address: Some(PostalAddress {
            street: Some("1 Apple Park Way".into()),
            city: Some("Cupertino".into()),
            state: Some("CA".into()),
            postal_code: Some("95014".into()),
            country: Some("United States".into()),
            iso_country_code: Some("US".into()),
            sub_administrative_area: Some("Santa Clara".into()),
            sub_locality: None,
        }),
        inland_water: None,
        ocean: None,
        areas_of_interest: vec![],
        time_zone_identifier: Some("America/Los_Angeles".into()),
        location: None,
        region: None,
    };
    let json = serde_json::to_string(&placemark)?;
    let decoded: Placemark = serde_json::from_str(&json)?;
    assert_eq!(decoded.postal_address, placemark.postal_address);

    let region = CircularRegion::new(
        Coordinate::new(37.3349, -122.0090),
        2_000.0,
        "apple-park-search",
    )?;
    let _ = geocoder.geocode_address_string_in_region("Apple Park", Some(&region));
    let _ = geocoder.geocode_postal_address(&PostalAddress::default());
    Ok(())
}
