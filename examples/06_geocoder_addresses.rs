use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let geocoder = Geocoder::new()?;
    let region = CircularRegion::new(
        Coordinate::new(37.3349, -122.0090),
        2_000.0,
        "apple-park-search",
    )?;
    let address = PostalAddress {
        street: Some("1 Apple Park Way".into()),
        city: Some("Cupertino".into()),
        state: Some("CA".into()),
        postal_code: Some("95014".into()),
        country: Some("United States".into()),
        iso_country_code: Some("US".into()),
        sub_administrative_area: Some("Santa Clara".into()),
        sub_locality: None,
    };

    match geocoder.geocode_address_string_in_region("Apple Park", Some(&region)) {
        Ok(placemarks) => println!("regional_geocode_count = {}", placemarks.len()),
        Err(error) => println!("regional geocode skipped: {error}"),
    }

    match geocoder.geocode_postal_address(&address) {
        Ok(placemarks) => println!("postal_geocode_count = {}", placemarks.len()),
        Err(error) => println!("postal geocode skipped: {error}"),
    }

    Ok(())
}
