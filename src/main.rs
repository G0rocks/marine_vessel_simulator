/// Here the marine vessel simulator is tested and implemented by examples.
// Author: G0rocks
// Date: 2025-04-30


use marine_vessel_simulator::*; // Import the add function from the marine_vessel_simulator crate

fn main() {
    // evaluate the data from shipping logs and calculate the mean speed of the vessels with standard deviation
    let file_path: &str = "mydata.csv"; // Path to the CSV file containing shipping logs
    let (speed_mean, speed_std, cargo_mean, cargo_std, num_trips) = evaluate_cargo_shipping_logs(file_path);

    let mut my_boat: Boat = Boat::new(); // Create a new boat instance
    println!("Boat name: {:?}", my_boat.name); // Print the name of the boat
    Boat::set_name(&mut my_boat, "The raisin");
    println!("Boat name: {:?}", my_boat.name.unwrap()); // Print the name of the boat

    // Print the result to the console
    println!("Speed mean: {:?}", speed_mean);
    println!("Speed std: {:?}", speed_std);
    println!("Cargo mean: {:?}", cargo_mean);
    println!("Cargo std: {:?}", cargo_std);
    println!("Num trips: {}", num_trips);
}