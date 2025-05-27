/// Everything Simulator related for the Marine vessel simulator that simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-05-27

use crate::*;   // To use everything from the crate

/// Enum of simulation methods
pub enum SimMethod {
    /// Constant velocity, uses the mean velocity of the boat
    ConstVelocity,
    // Use the mean and std of the boat speed
    MeanAndSTDVelocity,
    // Use downloaded weather data from file
    // Weather_data_from_file,
    // Use the copernicus weather data from the past for the exact location of the boat to simulate the boat movements
    // Copernicus_Weather_Data,
    // Use the copernicus weather forecast data for the exact location of the boat to simulate the boat movements
    // Copernicus_Weather_Forecast,
}



/// Function that simulates more than one waypoint mission
/// Saves the results of each simulation in the boat.ship_log
pub fn sim_waypoint_missions(boat: &mut Boat, start_times: Vec<Timestamp>, time_step: f64, max_iterations: usize) -> Result<Vec<String>, io::Error> {
    // Init sim_msg:
    let mut sim_msg_vec: Vec<String> = Vec::new();
    // Runs sim_waypoint_mission for each start time in start_times
    for (i, start_time) in start_times.iter().enumerate() {
        match sim_waypoint_mission(boat, *start_time, time_step, max_iterations) {
            Ok(sim_msg) => {
                // Add sim_msg to sim_msg_vec
                sim_msg_vec.push(sim_msg);
            }
            Err(e) => {
                // Print the error message
                return Err(io::Error::new(io::ErrorKind::Other, format!("Error during simulation {}: {}", i.to_string(), e)));
            }
        }
    }

    // Run successful, return Ok(sim_msg_vec)
    return Ok(sim_msg_vec);
}

/// Function to simulate the boat following a waypoint mission
/// Is basically a simulation handler that pipes the boat to the correct simulation function
/// TODO: Add what to return, save csv file? Return travel time and more? Also improve documentation
pub fn sim_waypoint_mission(boat: &mut Boat, start_time: Timestamp, time_step: f64, max_iterations: usize) -> Result<String, io::Error> {
    // Check if the boat has a route plan, if no route plan
    if boat.route_plan.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Boat has no route plan"));
    }

    // match simulation method and run corresponding simulation function
    match boat.simulation_method {
        Some(SimMethod::ConstVelocity) => {
            // Simulate the boat using constant velocity
            match sim_waypoint_mission_constant_velocity(boat, start_time, time_step, max_iterations) {
                Ok(sim_msg) => {
                    return Ok(sim_msg);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Some(SimMethod::MeanAndSTDVelocity) => {
            // Simulate the boat using constant velocity
            match sim_waypoint_mission_mean_and_std_velocity(boat, start_time, time_step, max_iterations) {
                Ok(sim_msg) => {
                    return Ok(sim_msg);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        // Add other simulation methods here

        None => {
            // If no simulation method is set, return error
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "No simulation method set"));
        }
    } 
}


// Simulators
//----------------------------------------------------
/// Simulates the boat using constant velocity (uses boat.mean_velocity)
pub fn sim_waypoint_mission_constant_velocity(boat: &mut Boat, start_time: Timestamp, time_step: f64, max_iterations: usize) -> Result<String, io::Error> {
    // Verify that boat has mean velocity set
    if boat.velocity_mean.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing mean velocity"));
    }

    // Set boats current location to the first waypoint
    boat.location = Some(boat.route_plan.as_ref().expect("Route plan missing?")[0].p1);
    // Set current leg to 1
    boat.current_leg = Some(1);
    // Get total number of legs
    let total_legs: usize = boat.route_plan.as_ref().expect("Route plan missing?").len();

    // Init travel_dist
    let mut travel_dist: uom::si::f64::Length;

    // Init ship_log_entry
    // Get initial location
    let coordinates_initial = boat.location.unwrap();
    // Get final location to last waypoint
    let coordinates_final = boat.route_plan.as_ref().expect("Route plan missing?")[total_legs - 1].p2;                
    let new_log_entry: ShipLogEntry = ShipLogEntry {
        timestamp: Timestamp::new(start_time.year, start_time.month, start_time.day, start_time.hour, start_time.minute, start_time.second),
        coordinates_initial: coordinates_initial,
        coordinates_current: coordinates_initial,
        coordinates_final: coordinates_final,
        cargo_on_board: boat.cargo_current,
    };
    // Push first ship log entry
    boat.ship_log.push(new_log_entry);

    // Loop through each time step
    for i in 0..max_iterations {
        // Simulate the boat moving towards the next waypoint
        // Get distance traveled in time step
        // travel_dist = boat.velocity_mean.unwrap() * time_step;
        travel_dist = boat.velocity_mean.unwrap() * uom::si::f64::Time::new::<uom::si::time::day>(time_step); // travel_dist in meters, https://docs.rs/uom/latest/uom/si/f64/struct.Velocity.html#method.times

        // While still have some distance left to travel during time step
        while travel_dist.get::<uom::si::length::meter>() > 0.0 {

            // Get next waypoint
            let next_waypoint: geo::Point = boat.route_plan.as_ref().expect("Route plan missing?")[(boat.current_leg.unwrap()-1) as usize].p2;
            // Get distance to next waypoint from current location
            let dist_to_next_waypoint: uom::si::f64::Length = haversine_distance_uom_units(boat.location.unwrap(), next_waypoint);

            // if distance traveled is greater than the distance to the next waypoint move to next waypoint, update current leg number and go to next while loop iteration
            if travel_dist > dist_to_next_waypoint {
                // Move to next waypoint
                boat.location = Some(next_waypoint);

                // If the boat has reached the last waypoint, stop the simulation
                if boat.location.unwrap() == coordinates_final {
                    // Update ship logs with last point
                    let new_log_entry: ShipLogEntry = ShipLogEntry {
                        // Set timestamp to last shiplogentry + time step
                        timestamp: boat.ship_log.last().unwrap().timestamp.add_days(time_step),
                        // timestamp: boat.ship_log.last().unwrap().timestamp.add_days(time_step),
                        //timestamp: start_time + uom::si::f64::Time::new::<uom::si::time::second>(((i + 1) as f64)*time_step.get::<uom::si::time::second>()),
                        coordinates_initial: coordinates_initial,
                        coordinates_current: boat.location.unwrap(),
                        coordinates_final: coordinates_final,
                        cargo_on_board: boat.cargo_current,
                    };

                    // Push the new log entry to the ship log
                    boat.ship_log.push(new_log_entry);

                    // Stop the simulation
                    return Ok("Simulation completed".to_string());
                }

                // Update current leg number
                boat.current_leg = Some(boat.current_leg.unwrap() + 1);
                // Reduce travel distance by distance to next waypoint
                travel_dist = travel_dist - dist_to_next_waypoint;
            }
            // Otherwise, move boat towards next waypoint and log to ship_log
            else {
                // Get bearing to next waypoint
                let bearing = Haversine.bearing(boat.location.unwrap(), next_waypoint);

                // Get the new location of the boat with distance left to travel during timestep and bearing to next waypoint
                let new_location: geo::Point = Haversine.destination(boat.location.unwrap(), bearing, travel_dist.get::<uom::si::length::meter>()); // travel_dist in meters, https://docs.rs/geo/0.30.0/geo/algorithm/line_measures/metric_spaces/struct.HaversineMeasure.html#method.destination

                // Update the location of the boat
                boat.location = Some(new_location);

                // Log the new location to the ship log
                let new_log_entry: ShipLogEntry = ShipLogEntry {
                    timestamp: start_time.add_days(((i + 1) as f64)*time_step),
                    // timestamp: start_time + ((i + 1) as f64)*time_step,
                    // timestamp: start_time + uom::si::f64::Time::new::<uom::si::time::second>(((i + 1) as f64)*time_step.get::<uom::si::time::second>()),
                    coordinates_initial: coordinates_initial,
                    coordinates_current: boat.location.unwrap(),
                    coordinates_final: coordinates_final,
                    cargo_on_board: boat.cargo_current,
                    };

                // Push the new log entry to the ship log
                boat.ship_log.push(new_log_entry);

                // Set travel distance to zero for next loop
                travel_dist = travel_dist - travel_dist;
            }
        } // End while loop
    } // End for loop

    // Simulation ran through all the iterations, return ship log and error that the simulation did not finish
    // Return the ship log TODO: Move inside for loop
    return Ok("Maximized number of iterations. Stopping simulation".to_string());
}


/// Simulates the boat using mean and standard deviation velocity (uses boat.mean_velocity and boat.std_velocity)
pub fn sim_waypoint_mission_mean_and_std_velocity(boat: &mut Boat, start_time: Timestamp, time_step: f64, max_iterations: usize) -> Result<String, io::Error> {
    // Verify that boat has mean and std velocity set
    if boat.velocity_mean.is_none() || boat.velocity_std.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing mean or standard deviation velocity"));
    }

    // Set boats current location to the first waypoint
    boat.location = Some(boat.route_plan.as_ref().expect("Route plan missing?")[0].p1);
    // Set current leg to 1
    boat.current_leg = Some(1);
    // Get total number of legs
    let total_legs: usize = boat.route_plan.as_ref().expect("Route plan missing?").len();

    // Init travel_dist
    let mut travel_dist: uom::si::f64::Length;
    // init working velocity
    let mut working_velocity: uom::si::f64::Velocity;

    // Init ship_log_entry
    // Get initial location
    let coordinates_initial = boat.location.unwrap();
    // Get final location to last waypoint
    let coordinates_final = boat.route_plan.as_ref().expect("Route plan missing?")[total_legs - 1].p2;                
    let new_log_entry: ShipLogEntry = ShipLogEntry {
        timestamp: Timestamp::new(start_time.year, start_time.month, start_time.day, start_time.hour, start_time.minute, start_time.second),
        coordinates_initial: coordinates_initial,
        coordinates_current: coordinates_initial,
        coordinates_final: coordinates_final,
        cargo_on_board: boat.cargo_current,
    };
    // Push first ship log entry
    boat.ship_log.push(new_log_entry);


    // Loop through each time step
    for i in 0..max_iterations {
        // Simulate the boat moving towards the next waypoint
        // Working velocity is mean velocity plus a random standard deviation from the mean
        working_velocity = boat.velocity_mean.unwrap() + uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>(rand::random_range(-1.0..=1.0) * boat.velocity_std.unwrap().get::<uom::si::velocity::meter_per_second>());

        // Get distance traveled in time step
        travel_dist = working_velocity * uom::si::f64::Time::new::<uom::si::time::day>(time_step); // travel_dist in meters, https://docs.rs/uom/latest/uom/si/f64/struct.Velocity.html#method.times

        // While still have some distance left to travel during time step
        while travel_dist.get::<uom::si::length::meter>() > 0.0 {

            // Get next waypoint
            let next_waypoint: geo::Point = boat.route_plan.as_ref().expect("Route plan missing?")[(boat.current_leg.unwrap()-1) as usize].p2;
            // Get distance to next waypoint from current location
            let dist_to_next_waypoint: uom::si::f64::Length = haversine_distance_uom_units(boat.location.unwrap(), next_waypoint);

            // if distance traveled is greater than the distance to the next waypoint move to next waypoint, update current leg number and go to next while loop iteration
            if travel_dist > dist_to_next_waypoint {
                // Move to next waypoint
                boat.location = Some(next_waypoint);

                // If the boat has reached the last waypoint, stop the simulation
                if boat.location.unwrap() == coordinates_final {
                    // Update ship logs with last point
                    let new_log_entry: ShipLogEntry = ShipLogEntry {
                        // Set timestamp to last shiplogentry + time step
                        timestamp: boat.ship_log.last().unwrap().timestamp.add_days(time_step),
                        // timestamp: boat.ship_log.last().unwrap().timestamp.add_days(time_step),
                        //timestamp: start_time + uom::si::f64::Time::new::<uom::si::time::second>(((i + 1) as f64)*time_step.get::<uom::si::time::second>()),
                        coordinates_initial: coordinates_initial,
                        coordinates_current: boat.location.unwrap(),
                        coordinates_final: coordinates_final,
                        cargo_on_board: boat.cargo_current,
                    };

                    // Push the new log entry to the ship log
                    boat.ship_log.push(new_log_entry);

                    // Stop the simulation
                    return Ok("Simulation completed".to_string());
                }

                // Update current leg number
                boat.current_leg = Some(boat.current_leg.unwrap() + 1);
                // Reduce travel distance by distance to next waypoint
                travel_dist = travel_dist - dist_to_next_waypoint;
            }
            // Otherwise, move boat towards next waypoint and log to ship_log
            else {
                // Get bearing to next waypoint
                let bearing = Haversine.bearing(boat.location.unwrap(), next_waypoint);

                // Get the new location of the boat with distance left to travel during timestep and bearing to next waypoint
                let new_location: geo::Point = Haversine.destination(boat.location.unwrap(), bearing, travel_dist.get::<uom::si::length::meter>()); // travel_dist in meters, https://docs.rs/geo/0.30.0/geo/algorithm/line_measures/metric_spaces/struct.HaversineMeasure.html#method.destination

                // Update the location of the boat
                boat.location = Some(new_location);

                // Log the new location to the ship log
                let new_log_entry: ShipLogEntry = ShipLogEntry {
                    timestamp: start_time.add_days(((i + 1) as f64)*time_step),
                    // timestamp: start_time + ((i + 1) as f64)*time_step,
                    // timestamp: start_time + uom::si::f64::Time::new::<uom::si::time::second>(((i + 1) as f64)*time_step.get::<uom::si::time::second>()),
                    coordinates_initial: coordinates_initial,
                    coordinates_current: boat.location.unwrap(),
                    coordinates_final: coordinates_final,
                    cargo_on_board: boat.cargo_current,
                    };

                // Push the new log entry to the ship log
                boat.ship_log.push(new_log_entry);

                // Set travel distance to zero for next loop
                travel_dist = travel_dist - travel_dist;
            }
        } // End while loop
    } // End for loop

    // Simulation ran through all the iterations, return ship log and error that the simulation did not finish
    // Return the ship log TODO: Move inside for loop
    return Ok("Maximized number of iterations. Stopping simulation".to_string());
}
