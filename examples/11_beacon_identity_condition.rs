use corelocation::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match BeaconIdentityCondition::with_major_minor("AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE", 1, 2) {
        Ok(condition) => println!("beacon_identity_condition = {:?}", condition.snapshot()?),
        Err(error) => println!("beacon identity condition skipped: {error}"),
    }

    let constraint = BeaconIdentityConstraint::with_major_minor(
        "AAAAAAAA-BBBB-CCCC-DDDD-EEEEEEEEEEEE",
        1,
        2,
    )?;
    println!("beacon_identity_constraint = {:?}", constraint.snapshot()?);
    let region = BeaconRegion::from_constraint(&constraint, "legacy-beacon")?;
    println!("beacon_region_from_constraint = {:?}", region.snapshot()?);
    Ok(())
}
