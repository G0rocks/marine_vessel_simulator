/// Everything Simulator related for the Marine vessel simulator that simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-05-27

use crate::*;   // To use everything from the crate

/// Enum of simulation methods
#[derive(Debug)]
pub enum SimMethod {
    /// Constant velocity, uses the mean velocity of the boat
    ConstVelocity,
    /// Use the mean and std of the boat speed
    MeanAndSTDVelocity,
    /// Use downloaded weather data from file
    WeatherDataFromFile,
    // Use the copernicus weather data from the past for the exact location of the boat to simulate the boat movements
    // Copernicus_Weather_Data,
    // Use the copernicus weather forecast data for the exact location of the boat to simulate the boat movements
    // Copernicus_Weather_Forecast,
}


/// Struct for simulation
#[derive(Debug)]
pub struct Simulation {
    /// The simulation method to use
    pub simulation_method: SimMethod,
    /// Start times for the simulation
    pub start_times: Vec<Timestamp>,
    /// The time step for the simulation in days
    pub time_step: f64, // Time step for the simulation in days
    /// The maximum number of iterations for the simulation
    pub max_iterations: usize, // Maximum number of iterations for the simulation
    /// Weather data file for the simulation
    pub weather_data_file: Option<String>, // Weather data file for the simulation
    // TODO: Add Copernicus information
}

impl Simulation {
    /// Creates a new simulation with the given parameters
    pub fn new(simulation_method: SimMethod, start_times: Vec<Timestamp>, time_step: f64, max_iterations: usize, weather_data_file: Option<String>) -> Self {
        Simulation {
            simulation_method,
            start_times,
            time_step,
            max_iterations,
            weather_data_file,
        }
    }
}


/// Function that simulates more than one waypoint mission
/// Saves the results of each simulation in the boat.ship_log
pub fn sim_waypoint_missions(boat: &mut Boat, simulation: &Simulation) -> Result<Vec<String>, io::Error> {
    // Init sim_msg:
    let mut sim_msg_vec: Vec<String> = Vec::new();
    // Runs sim_waypoint_mission for each start time in start_times
    for (i, start_time) in simulation.start_times.iter().enumerate() {
        match sim_waypoint_mission(boat, *start_time, simulation) {
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
pub fn sim_waypoint_mission(boat: &mut Boat, start_time: Timestamp, simulation: &Simulation) -> Result<String, io::Error> {
    // Check if the boat has a route plan, if no route plan
    if boat.route_plan.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Boat has no route plan"));
    }

    // match simulation method and run corresponding simulation function
    match simulation.simulation_method {
        SimMethod::ConstVelocity => {
            // Simulate the boat using constant velocity
            match sim_waypoint_mission_constant_velocity(boat, start_time, simulation) {
                Ok(sim_msg) => {
                    return Ok(sim_msg);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        SimMethod::MeanAndSTDVelocity => {
            // Simulate the boat using constant velocity
            match sim_waypoint_mission_mean_and_std_velocity(boat, start_time, simulation) {
                Ok(sim_msg) => {
                    return Ok(sim_msg);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        SimMethod::WeatherDataFromFile => {
            // Simulate the boat using weather data from file
            match sim_waypoint_mission_weather_data_from_file(boat, start_time, simulation) {
                Ok(sim_msg) => {
                    return Ok(sim_msg);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        // Add other simulation methods here
    } 
}


// Simulators
//----------------------------------------------------
/// Simulates the boat using constant velocity (uses boat.mean_velocity)
pub fn sim_waypoint_mission_constant_velocity(boat: &mut Boat, start_time: Timestamp, simulation: &Simulation) -> Result<String, io::Error> {
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
    for i in 0..simulation.max_iterations {
        // Simulate the boat moving towards the next waypoint
        // Get distance traveled in time step
        // travel_dist = boat.velocity_mean.unwrap() * time_step;
        travel_dist = boat.velocity_mean.unwrap() * uom::si::f64::Time::new::<uom::si::time::day>(simulation.time_step); // travel_dist in meters, https://docs.rs/uom/latest/uom/si/f64/struct.Velocity.html#method.times

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
                        timestamp: boat.ship_log.last().unwrap().timestamp.add_days(simulation.time_step),
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
                    timestamp: start_time.add_days(((i + 1) as f64)*simulation.time_step),
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
pub fn sim_waypoint_mission_mean_and_std_velocity(boat: &mut Boat, start_time: Timestamp, simulation: &Simulation) -> Result<String, io::Error> {
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
    for i in 0..simulation.max_iterations {
        // Simulate the boat moving towards the next waypoint
        // Working velocity is mean velocity plus a random standard deviation from the mean
        working_velocity = boat.velocity_mean.unwrap() + uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>(rand::random_range(-1.0..=1.0) * boat.velocity_std.unwrap().get::<uom::si::velocity::meter_per_second>());

        // Get distance traveled in time step
        travel_dist = working_velocity * uom::si::f64::Time::new::<uom::si::time::day>(simulation.time_step); // travel_dist in meters, https://docs.rs/uom/latest/uom/si/f64/struct.Velocity.html#method.times

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
                        timestamp: boat.ship_log.last().unwrap().timestamp.add_days(simulation.time_step),
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
                    timestamp: start_time.add_days(((i + 1) as f64)*simulation.time_step),
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

/// Simulates the boat using weather data from file
/// NOTE: Currently uses 5 m/s blowing in from the north as a placeholder for the weather data
/// Note: Tacking width is the total width around the center of leg line for each leg.
pub fn sim_waypoint_mission_weather_data_from_file(boat: &mut Boat, start_time: Timestamp, simulation: &Simulation) -> Result<String, io::Error> {
    // Verify that necessary fields are set
    if simulation.weather_data_file.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing weather data file name from simulation"));
    }
    if boat.mass.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing mass from boat"));
    }
    if boat.sail.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing sail from boat"));
    }
    if boat.min_angle_of_attack.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing minimum angle of attack from boat"));
    }
    // if boat.hull_drag_coefficient.is_none() {
    //     return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing drag coefficient from boat"));
    // }
    if boat.route_plan.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing route plan from boat"));
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

    // Init wind vector
    let wind: Wind = Wind::new(uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>(5.0), 0.0); // Placeholder for wind speed, should be replaced with actual weather data from file
    // Init next waypoint
    let mut next_waypoint: geo::Point;
    let mut bearing_to_next_waypoint: f64;
    let mut iteration_counter: usize = 0; // Counter for iterations to tack back into tacking width
    // Todo: Add number of tacks?

    // Loop through each time step
    for i in 0..simulation.max_iterations {
        // Simulate the boat moving towards the next waypoint
        // Get next waypoint
        next_waypoint = boat.route_plan.as_ref().expect("Route plan missing?")[(boat.current_leg.unwrap()-1) as usize].p2;

        // Get wind speed and direction for current location from weather data file
        // ToDO
        
        // Check if boat is out of the tacking width of the route plan, tack by setting preferred wind side of the boat
        // Get tacking width from route plan
        let tacking_width = boat.route_plan.as_ref().expect("Route plan missing?")[(boat.current_leg.unwrap()-1) as usize].tacking_width;
        // Get shortest distance to leg line from current location
        // If distance to leg line is bigger than tacking width, tack. Give boat 10 iterations to make it back inside allowed area
        if (iteration_counter + 10) < i && tacking_width < min_haversine_distance(boat.route_plan.as_ref().expect("Route plan missing?")[(boat.current_leg.unwrap()-1) as usize].p1, next_waypoint, boat.location.unwrap()) {
            // Tack, flipping preferred side of the boat for wind
            boat.wind_preferred_side.switch();
            // Set iteration counter to i
            iteration_counter = i;
        }
        
        // Compute heading
        // Compute angle of wind relative to line between current location and next waypoint. North: 0°, East: 90°, South: 180°, West: 270°
        bearing_to_next_waypoint = Haversine.bearing(boat.location.unwrap(), next_waypoint);
        // Compute angle of wind relative to boat heading
        let relative_wind_angle = wind.angle - bearing_to_next_waypoint;
        // Relative wind angle must be in the range of -180° to 180°
        let relative_wind_angle = if relative_wind_angle < -180.0 {
            relative_wind_angle + 360.0
        } else if relative_wind_angle > 180.0 {
            relative_wind_angle - 360.0
        } else {
            relative_wind_angle
        };

        // If absolute relative wind angle is smaller than minimum angle of attack, then use tacking method
        if relative_wind_angle.abs() < boat.min_angle_of_attack.unwrap() {
            // If wind is on port side, keep wind on port side and opposite for starboard side
            // Set heading to the minimum angle of attack with respect to the wind angle 
            if boat.wind_preferred_side == VesselSide::Port {
                // Wind on port side
                boat.heading = Some(wind.angle + boat.min_angle_of_attack.unwrap());
            } else if boat.wind_preferred_side == VesselSide::Starboard {
                // Wind on starboard side, keep on starboard side
                boat.heading = Some(wind.angle - boat.min_angle_of_attack.unwrap());
            }   // If boat has no preferred wind side set, catch and set to starboard
            else {
                boat.wind_preferred_side = VesselSide::Starboard; // Default to starboard since then we have the right of way in most cases
                boat.heading = Some(wind.angle - boat.min_angle_of_attack.unwrap());
            }
        } // Otherwise relative wind angle is bigger than minimum angle of attack, then go straight towards next waypoint
        else {
            // Set heading to the bearing to next waypoint
            boat.heading = Some(bearing_to_next_waypoint);
        }





        
        // Find total force on boat
        // force on boat from wind
        // let wind_force: uom::si::f64::Force = uom::si::f64::Force::new::<uom::si::force::newton>(5.0);

        // Force from velocity of ocean current

        // Add forces together
        // let force_total: uom::si::f64::Force = wind_force; // + ocean_current_force; // Add other forces here

        // Find total mass
        // let total_mass: uom::si::f64::Mass = boat.mass.unwrap() + boat.cargo_current; // Add cargo mass to boat mass

        // Find acceleration of boat from forces
        // let a: uom::si::f64::Acceleration = force_total / total_mass; // a = F/m, where F is the total force on the boat and m is the mass of the boat

        // Find final velocity of boat from acceleration
        // let final_velocity: uom::si::f64::Velocity = a * uom::si::f64::Time::new::<uom::si::time::day>(simulation.time_step); // final_velocity in meters per second

        // Working velocity is initial velocity plus final velocity divided by 2
        working_velocity = boat.velocity_mean.unwrap(); // (boat.velocity_current.unwrap() + final_velocity) / 2.0; // working_velocity in meters per second

        // Update the current velocity of the boat
        // boat.velocity_current = Some(working_velocity);

        // Calculate drag on hull from working velocity
        //Todo make sure all forces are correct, for now we just want to see the boat tack on the visualization


        // Get distance traveled in time step
        travel_dist = working_velocity * uom::si::f64::Time::new::<uom::si::time::day>(simulation.time_step); // travel_dist in meters, https://docs.rs/uom/latest/uom/si/f64/struct.Velocity.html#method.times
        
        

        // While still have some distance left to travel during time step
        while travel_dist.get::<uom::si::length::meter>() > 0.0 {
            // Get next waypoint
            next_waypoint = boat.route_plan.as_ref().expect("Route plan missing?")[(boat.current_leg.unwrap()-1) as usize].p2;

            // Get distance to next waypoint from current location
            let dist_to_next_waypoint: uom::si::f64::Length = haversine_distance_uom_units(boat.location.unwrap(), next_waypoint);

            // if distance traveled is greater than the distance to the next waypoint and the heading is the bearing to the next waypoint move to next waypoint, update current leg number and go to next while loop iteration
            if travel_dist > dist_to_next_waypoint && boat.heading.unwrap() == bearing_to_next_waypoint {
                // Move to next waypoint
                boat.location = Some(next_waypoint);

                // If the boat has reached the last waypoint, stop the simulation
                if boat.location.unwrap() == coordinates_final {
                    // Update ship logs with last point
                    let new_log_entry: ShipLogEntry = ShipLogEntry {
                        // Set timestamp to last shiplogentry + time step
                        timestamp: boat.ship_log.last().unwrap().timestamp.add_days(simulation.time_step),
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
            // Otherwise, move boat forwards along heading and log to ship_log
            else {
                // Get the new location of the boat with distance left to travel during timestep and bearing to next waypoint
                let new_location: geo::Point = Haversine.destination(boat.location.unwrap(), boat.heading.unwrap(), travel_dist.get::<uom::si::length::meter>()); // travel_dist in meters, https://docs.rs/geo/0.30.0/geo/algorithm/line_measures/metric_spaces/struct.HaversineMeasure.html#method.destination

                // Update the location of the boat
                boat.location = Some(new_location);

                // Log the new location to the ship log
                let new_log_entry: ShipLogEntry = ShipLogEntry {
                    timestamp: start_time.add_days(((i + 1) as f64)*simulation.time_step),
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