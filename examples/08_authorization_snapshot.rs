use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manager = LocationManager::new()?;
    println!(
        "global_authorization_status = {:?}",
        LocationManager::global_authorization_status()
    );
    println!("manager_authorization = {:?}", manager.authorization()?);
    println!(
        "ranging_available = {}",
        LocationManager::ranging_available()
    );
    Ok(())
}
