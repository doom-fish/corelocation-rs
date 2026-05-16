use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = LocationManager::new()?;
    println!(
        "authorization_status = {:?}",
        manager.authorization_status()
    );
    println!("authorization_snapshot = {:?}", manager.authorization()?);
    println!(
        "location_services_enabled = {}",
        LocationManager::location_services_enabled()
    );

    let geocoder = Geocoder::new()?;
    match geocoder.geocode_address_string("Apple Park, Cupertino") {
        Ok(placemarks) => {
            if let Some(first) = placemarks.first() {
                println!("geocode locality = {:?}", first.locality);
                println!("geocode country = {:?}", first.country);
            } else {
                println!("geocode returned no placemarks");
            }
        }
        Err(error) => {
            println!("geocode skipped: {error}");
        }
    }

    println!("✅ corelocation smoke OK");
    Ok(())
}
