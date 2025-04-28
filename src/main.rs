/// Here the marine vessel simulator is tested and implemented by examples.

use marine_vessel_simulator::*; // Import the add function from the marine_vessel_simulator crate

fn main() {
    // evaluate the data from shipping logs and calculate the mean speed of the vessels with standard deviation
    let file_path: &str = "mydata.csv"; // Path to the CSV file containing shipping logs
    let (speed_mean, speed_std, cargo_mean, cargo_std) = evaluate_cargo_shipping_logs(file_path);

    // Print the result to the console
    
    // PRINT SPEED AS A TEST. CAN BE DELETED
    let v_format = uom::si::f64::Velocity::format_args(uom::si::velocity::kilometer_per_hour, uom::fmt::DisplayStyle::Abbreviation);
    println!("{}", v_format.with(speed_mean));
    // println!("Speed mean: {}", speed_mean);
    println!("Speed std: {}", speed_std);
    println!("Cargo mean: {}", cargo_mean);
    println!("Cargo std: {}", cargo_std);


}