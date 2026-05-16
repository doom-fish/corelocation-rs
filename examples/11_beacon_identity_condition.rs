use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match BeaconIdentityCondition::with_major_minor("AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE", 1, 2) {
        Ok(condition) => println!("beacon_identity_condition = {:?}", condition.snapshot()?),
        Err(error) => println!("beacon identity condition skipped: {error}"),
    }
    Ok(())
}
