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
    // Use downloaded weather data from file
    // WeatherDataFromFile,
    /// Use the copernicus weather data from the past for the exact location of the boat to simulate the boat movements
    WeatherDataFromCopernicus,
    // Use the copernicus weather forecast data for the exact location of the boat to simulate the boat movements
    // Copernicus_Weather_Forecast,
}


/// Struct for simulation
#[derive(Debug)]
pub struct Simulation {
    /// The simulation method to use
    pub simulation_method: SimMethod,
    /// Start times for the simulation
    pub start_times: Vec<time::UtcDateTime>,
    /// The time step for the simulation in days
    pub time_step: time::Duration, // Time step for the simulation in seconds
    /// The maximum number of iterations for the simulation
    pub max_iterations: usize, // Maximum number of iterations for the simulation
    /// Weather data file for the simulation
    pub weather_data_file: Option<String>, // Weather data file for the simulation
    /// Copernicus information
    pub copernicus: Option<copernicusmarine_rs::Copernicus>,
    /// Progress bar, set to none if not needed, if you use, set the length to the total number of legs in all simulations
    pub progress_bar: Option<indicatif::ProgressBar>,
}

impl Simulation {
    /// Creates a new simulation with the given parameters
    /// # Example - Adding a progress bar
    /// // Init progress bar for simulation
    /// let mut progress_bar = indicatif::ProgressBar::new((num_simulations*100) as u64);
    /// // Set progress bar style
    /// progress_bar.set_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar} {pos:>3}/{len:3} ETA:{eta:>1}").unwrap()); //.progress_chars("##-"));
    /// // Add progress bar to simulation
    /// my_sim.progress_bar = Some(progress_bar); // Set the progress bar for the simulation
    pub fn new(simulation_method: SimMethod, start_times: Vec<UtcDateTime>, time_step: time::Duration, max_iterations: usize, weather_data_file: Option<String>, copernicus: Option<copernicusmarine_rs::Copernicus>) -> Self {
        Simulation {
            simulation_method,
            start_times,
            time_step,
            max_iterations,
            weather_data_file,
            copernicus,
            progress_bar: None
        }
    }
}


/// Function that simulates more than one waypoint mission
/// Saves the results of each simulation in the boat.ship_log
pub fn sim_waypoint_missions(boat: &mut Boat, simulation: &Simulation) -> Result<Vec<String>, io::Error> {
    // Init sim_msg:
    let mut sim_msg_vec: Vec<String> = Vec::new();

    // Check for interactive terminal for progress bar
    let is_interactive_terminal = atty::is(atty::Stream::Stdout);
    // If simulation has progress bar, set it up and use it
    if !(simulation.progress_bar.is_none()) {
        // If terminal is interactive, use live redraw, otherwise use static redraw
        if is_interactive_terminal {
            // Normal terminal behavior (live redraw)
            simulation.progress_bar.as_ref().unwrap().set_draw_target(indicatif::ProgressDrawTarget::stdout());
            simulation.progress_bar.as_ref().unwrap().enable_steady_tick(std::time::Duration::from_millis(500));
        } else {
            // Force static redraw every step to stdout (or to log)
            // bar.set_draw_target(indicatif::ProgressDrawTarget::stdout_with_hz(1)); // Or `.stdout_with_hz(1)` for slow redraw
            let eta = time::UtcDateTime::now().saturating_add(time::Duration::new(simulation.progress_bar.as_ref().unwrap().eta().as_secs() as i64, 0)); // What time the simulations will end
            println!("Elapsed: {:?}, Steps {}/{}, ETA: {}-{}-{} {}:{}:{}", simulation.progress_bar.as_ref().unwrap().elapsed(), simulation.progress_bar.as_ref().unwrap().position(), simulation.progress_bar.as_ref().unwrap().length().unwrap(), eta.year(), eta.month() as u8, eta.day(), eta.hour()+1, eta.minute(), eta.second());
        }
        simulation.progress_bar.as_ref().unwrap().inc(0);
    }
    
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
    // Finish progress bar
    simulation.progress_bar.as_ref().unwrap().finish();

    // Run successful, return Ok(sim_msg_vec)
    return Ok(sim_msg_vec);
}

/// Function to simulate the boat following a waypoint mission
/// Is basically a simulation handler that pipes the boat to the correct simulation function
pub fn sim_waypoint_mission(boat: &mut Boat, start_time: time::UtcDateTime, simulation: &Simulation) -> Result<String, io::Error> {
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
        // SimMethod::WeatherDataFromFile => {
        //     // Simulate the boat using weather data from file
        //     match sim_waypoint_mission_weather_data_from_file(boat, start_time, simulation) {
        //         Ok(sim_msg) => {
        //             return Ok(sim_msg);
        //         }
        //         Err(e) => {
        //             return Err(e);
        //         }
        //     }
        // }
        SimMethod::WeatherDataFromCopernicus => {
            // Simulate the boat using weather data from Copernicus
            match sim_waypoint_mission_weather_data_from_copernicus(boat, start_time, simulation) {
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
pub fn sim_waypoint_mission_constant_velocity(boat: &mut Boat, start_time: time::UtcDateTime, simulation: &Simulation) -> Result<String, io::Error> {
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
        timestamp: time::UtcDateTime::new(time::Date::from_calendar_date(start_time.year(), start_time.month(), start_time.day()).expect("Couldn't make time::Date"), time::Time::from_hms(start_time.hour(), start_time.minute(), start_time.second()).expect("Couldn't make time::Time")),
        coordinates_initial: coordinates_initial,
        coordinates_current: coordinates_initial,
        coordinates_final: coordinates_final,
        cargo_on_board: Some(boat.cargo_current),
        velocity: Some(PhysVec::new(boat.velocity_mean.unwrap().get::<uom::si::velocity::meter_per_second>(), 0.0)),  // Initial velocity is defaulted to direction zero degrees
        course: None,
        heading: None,
        track_angle: Some(Rhumb.bearing(boat.ship_log.last().unwrap().coordinates_current, boat.location.unwrap())),
        true_bearing: None,
        draft: None,
        navigation_status: None,
    };
    // Push first ship log entry
    boat.ship_log.push(new_log_entry);

    // Loop through each time step
    for i in 0..simulation.max_iterations {
        // Simulate the boat moving towards the next waypoint
        // Get distance traveled in time step
        // travel_dist = boat.velocity_mean.unwrap() * time_step;
        travel_dist = boat.velocity_mean.unwrap() * uom::si::f64::Time::new::<uom::si::time::day>(simulation.time_step.as_seconds_f64()); // travel_dist in meters, https://docs.rs/uom/latest/uom/si/f64/struct.Velocity.html#method.times

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
                        timestamp: boat.ship_log.last().unwrap().timestamp.checked_add(simulation.time_step).expect("Couldn't add seconds, probably an overflow occured"),
                        coordinates_initial: coordinates_initial,
                        coordinates_current: boat.location.unwrap(),
                        coordinates_final: coordinates_final,
                        cargo_on_board: Some(boat.cargo_current),
                        velocity: Some(PhysVec::new(boat.velocity_mean.unwrap().get::<uom::si::velocity::meter_per_second>(), boat.heading.unwrap())),
                        course: None,
                        heading: boat.heading,
                        track_angle: Some(Rhumb.bearing(boat.ship_log.last().unwrap().coordinates_current, boat.location.unwrap())),
                        true_bearing: None,
                        draft: None,
                        navigation_status: None,
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
                    timestamp: start_time.checked_add(simulation.time_step.checked_mul((i + 1) as i32).expect("Could not multiply, an overflow error probably occurred")).expect("Could not add timestep, an overflow probably occurred"),
                    coordinates_initial: coordinates_initial,
                    coordinates_current: boat.location.unwrap(),
                    coordinates_final: coordinates_final,
                    cargo_on_board: Some(boat.cargo_current),
                    velocity: Some(PhysVec::new(boat.velocity_mean.unwrap().get::<uom::si::velocity::meter_per_second>(), boat.heading.unwrap())),
                    course: None,
                    heading: boat.heading,
                    track_angle: Some(Rhumb.bearing(boat.ship_log.last().unwrap().coordinates_current, boat.location.unwrap())),
                    true_bearing: None,
                    draft: None,
                    navigation_status: None,
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
pub fn sim_waypoint_mission_mean_and_std_velocity(boat: &mut Boat, start_time: time::UtcDateTime, simulation: &Simulation) -> Result<String, io::Error> {
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

    // Init travel_dist, unit [m]
    let mut travel_dist: f64;
    // init working velocity, unit [m/s]
    let mut working_velocity: PhysVec;

    // Init ship_log_entry
    // Get initial location
    let coordinates_initial = boat.location.unwrap();
    // Get final location to last waypoint
    let coordinates_final = boat.route_plan.as_ref().expect("Route plan missing?")[total_legs - 1].p2;                
    let new_log_entry: ShipLogEntry = ShipLogEntry {
        timestamp: time::UtcDateTime::new(time::Date::from_calendar_date(start_time.year(), start_time.month(), start_time.day()).expect("Could not make time::Date from values"), time::Time::from_hms(start_time.hour(), start_time.minute(), start_time.second()).expect("Could not make time::Time from values")),
        coordinates_initial: coordinates_initial,
        coordinates_current: coordinates_initial,
        coordinates_final: coordinates_final,
        cargo_on_board: Some(boat.cargo_current),
        velocity: None,
        course: None,
        heading: None,
        track_angle: None,
        true_bearing: None,
        draft: None,
        navigation_status: None,
    };
    // Push first ship log entry
    boat.ship_log.push(new_log_entry);


    // Loop through each time step
    for i in 0..simulation.max_iterations {
        // Simulate the boat moving towards the next waypoint
        // Working velocity is mean velocity plus a random standard deviation from the mean
        working_velocity = PhysVec::new(boat.velocity_mean.unwrap().get::<uom::si::velocity::meter_per_second>() + rand::random_range(-1.0..=1.0) * boat.velocity_std.unwrap().get::<uom::si::velocity::meter_per_second>(), boat.heading.unwrap());

        // Get distance traveled in time step, unit [m]
        travel_dist = working_velocity.magnitude * simulation.time_step.as_seconds_f64();

        // While still have some distance left to travel during time step
        while travel_dist > 0.0 {
            // Get next waypoint
            let next_waypoint: geo::Point = boat.route_plan.as_ref().expect("Route plan missing?")[(boat.current_leg.unwrap()-1) as usize].p2;
            // Get distance to next waypoint from current location
            let dist_to_next_waypoint: f64 = Haversine.distance(boat.location.unwrap(), next_waypoint);

            // if distance traveled is greater than the distance to the next waypoint move to next waypoint, update current leg number and go to next while loop iteration
            if travel_dist > dist_to_next_waypoint {
                // Move to next waypoint
                boat.location = Some(next_waypoint);

                // If the boat has reached the last waypoint, stop the simulation
                if boat.location.unwrap() == coordinates_final {
                    // Update ship logs with last point
                    let new_log_entry: ShipLogEntry = ShipLogEntry {
                        // Set timestamp to last shiplogentry + time step
                        timestamp: boat.ship_log.last().unwrap().timestamp.checked_add(simulation.time_step).expect("Could not add time::Duration to time::UtcDateTime. Maybe an overflow happened?"),
                        coordinates_initial: coordinates_initial,
                        coordinates_current: boat.location.unwrap(),
                        coordinates_final: coordinates_final,
                        cargo_on_board: Some(boat.cargo_current),
                        velocity: Some(working_velocity),
                        course: None,
                        heading: boat.heading,
                        track_angle: Some(Rhumb.bearing(boat.ship_log.last().unwrap().coordinates_current, boat.location.unwrap())),
                        true_bearing: None,
                        draft: None,
                        navigation_status: None,
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

                // Get the new location of the boat with distance left to travel during timestep and bearing to next waypoint, important to use meters for travel_dist
                let new_location: geo::Point = Haversine.destination(boat.location.unwrap(), bearing, travel_dist);

                // Update the location of the boat
                boat.location = Some(new_location);

                // Log the new location to the ship log
                let new_log_entry: ShipLogEntry = ShipLogEntry {
                    timestamp: start_time.checked_add(simulation.time_step.checked_mul((i + 1) as i32).expect("Could not multiply time::Duration with value. Maybe an overflow occurred?")).expect("Could not add time::Duration to time::UtcDateTime. Maybe an overflow occurred?"),
                    coordinates_initial: coordinates_initial,
                    coordinates_current: boat.location.unwrap(),
                    coordinates_final: coordinates_final,
                    cargo_on_board: Some(boat.cargo_current),
                    velocity: Some(working_velocity),
                    course: None,
                    heading: boat.heading,
                    track_angle: Some(Rhumb.bearing(boat.ship_log.last().unwrap().coordinates_current, boat.location.unwrap())),
                    true_bearing: None,
                    draft: None,
                    navigation_status: None,
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
pub fn sim_waypoint_mission_weather_data_from_copernicus(boat: &mut Boat, start_time: time::UtcDateTime, simulation: &Simulation) -> Result<String, io::Error> {
    // Verify that necessary fields are set
    if simulation.weather_data_file.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing weather data file name from simulation"));
    }
    if simulation.copernicus.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing copernicus info from simulation"))
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
    if boat.route_plan.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing route plan from boat"));
    }
    // TODO: Add drag
    // if boat.hull_drag_coefficient.is_none() {
    //     return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing drag coefficient from boat"));
    // }


    // Check for interactive terminal for progress bar
    let is_interactive_terminal = atty::is(atty::Stream::Stdout);

    // Set boats current location to the first waypoint
    boat.location = Some(boat.route_plan.as_ref().expect("Route plan missing?")[0].p1);
    // Set current leg to 1
    boat.current_leg = Some(1);
    // Get total number of legs
    let total_legs: usize = boat.route_plan.as_ref().expect("Route plan missing?").len();

    // Init travel_dist, unit [m]
    let mut travel_dist: f64;
    // init working velocity, unit [m/s]
    let mut working_velocity: PhysVec;

    // Get initial location
    let coordinates_initial = boat.location.unwrap();
    // Get final location to last waypoint
    let coordinates_final = boat.route_plan.as_ref().unwrap()[total_legs - 1].p2;
    // Init ship_log_entry
    let new_log_entry: ShipLogEntry = ShipLogEntry {
        timestamp: time::UtcDateTime::new(time::Date::from_calendar_date(start_time.year(), start_time.month(), start_time.day()).expect("Could not make time::Date from values"), time::Time::from_hms(start_time.hour(), start_time.minute(), start_time.second()).expect("Could not make time::Time from values")),
        coordinates_initial: coordinates_initial,
        coordinates_current: coordinates_initial,
        coordinates_final: coordinates_final,
        cargo_on_board: Some(boat.cargo_current),
        velocity: Some(PhysVec::new(0.0, 0.0)), // Start at 0 m/s with heading 0°
        course: None,
        heading: None,  // Note perhaps we can change this to be better, in the future
        track_angle: None,  // First point, can't get the angle from the last point since there is no last point
        true_bearing: None,
        draft: None,
        navigation_status: Some(NavigationStatus::UnderwaySailing),
    };
    // Push first ship log entry
    boat.ship_log.push(new_log_entry);

    // Init wind vector, unit [m/s]
    let mut wind: PhysVec;
    // Init ocean current vector, unit [m/s]
    let mut ocean_current: PhysVec;
    // Init waypoints
    let mut last_waypoint: geo::Point;
    let mut next_waypoint: geo::Point;
    let mut dist_to_next_waypoint: f64;
    // The angle (from north) from last to next waypoint
    let mut course: f64;
    // Init heading_adjustment to account for ocean_current
    let mut heading_adjustment: f64 = 0.0;
    // The minimum proximity to the next waypoint to consider the boat "at the waypotin"
    let mut min_proximity: f64;
    // Init bearing and other variables used in loop
    let mut bearing_to_next_waypoint: f64;
    let mut new_location: geo::Point;   // Init
    let mut temp_time_step: Option<f64> = None; // Temporary time step, used if the time step is longer than needed to reach a waypoint in seconds
    // TODO: Add number of tacks?

    // Loop through each time step
    let mut iteration: usize = 0;
    while iteration <= simulation.max_iterations {
        // Increment number of iterations
        iteration += 1;
        // Simulate the boat moving towards the next waypoint
        // Get working time step
        let working_time_step = match temp_time_step {
            // If temp_time_step is set, use it
            Some(t) => t,
            // If no temp_time_step, use the simulation time step
            None => simulation.time_step.as_seconds_f64(),
        };
        // Reset temp_time_step
        temp_time_step = None;

        // Get last and next waypoint from routeplan
        last_waypoint = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].p1;
        next_waypoint = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].p2;
        // Get minimum proximity [m] to next waypoint from route plan
        min_proximity = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].min_proximity;

        // Print for debug
        // println!("Distance to last waypoint: {:.3} km", Haversine.distance(boat.location.unwrap(), last_waypoint)/1000.0);
        // println!("Distance to next waypoint: {:.3} km", Haversine.distance(boat.location.unwrap(), next_waypoint)/1000.0);

        // Get boat current time and location
        let boat_time_now: UtcDateTime = boat.ship_log.last().unwrap().timestamp;
        let longitude: f64 = boat.location.expect("Boat has no location").x();
        let latitude: f64 = boat.location.expect("Boat has no location").y();

        // Pick next waypoint
        // Get next waypoint
        next_waypoint = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].p2;

        // Get distance to next waypoint from current location
        dist_to_next_waypoint = Haversine.distance(boat.location.unwrap(), next_waypoint);

        // if distance to the next waypoint is shorter than the simulation minimum proximity (or we are at the next waypoint)
        // Then we are at the next waypoint. Check if this is the final waypoint (if so, finish simulation) or go to next leg and continue simulation
        if (dist_to_next_waypoint < min_proximity) || (boat.location.unwrap() == next_waypoint) {
            // If the boat has reached the last waypoint, stop the simulation
            if next_waypoint == coordinates_final {
                // Stop the simulation
                return Ok("Simulation completed".to_string());
            }

            // Update current leg number
            boat.current_leg = Some(boat.current_leg.unwrap() + 1);
        
            // Since leg number increased, update progress bar if a progress bar is in use
            if !(simulation.progress_bar.is_none()) {
                // If leg number increased, update progress bar
                simulation.progress_bar.as_ref().unwrap().inc(1);
                // If not interactive terminal, print progressbar manually
                if is_interactive_terminal == false {
                    let eta = time::UtcDateTime::now().saturating_add(time::Duration::new(simulation.progress_bar.as_ref().unwrap().eta().as_secs() as i64, 0)); // What time the simulations will end
                println!("Elapsed: {} secs, Steps {}/{}, ETA: {}-{}-{} {}:{}:{}", simulation.progress_bar.as_ref().unwrap().elapsed().as_secs(), simulation.progress_bar.as_ref().unwrap().position(), simulation.progress_bar.as_ref().unwrap().length().unwrap(), eta.year(), eta.month() as u8, eta.day(), eta.hour(), eta.minute(), eta.second());
                }
            }   // End if
        }   // End if

        // Get last and next waypoint from routeplan
        last_waypoint = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].p1;
        next_waypoint = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].p2;
        course = Rhumb.bearing(last_waypoint, next_waypoint);
        // Recalculate distance to next waypoint from current location in case we just reached a waypoint and are going to the next one
        dist_to_next_waypoint = Haversine.distance(boat.location.unwrap(), next_waypoint);

        // Get tacking width from route plan
        let tacking_width: f64 = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].tacking_width;

        // Get wind data from Copernicus
        let wind_data = match simulation.copernicus.as_ref().unwrap().get_f64_values("cmems_obs-wind_glo_phy_nrt_l4_0.125deg_PT1H".to_string(), vec!["eastward_wind".to_string(), "northward_wind".to_string()], boat_time_now, boat_time_now, longitude, longitude, latitude, latitude, None, None) {
            Ok(w) => w,
            Err(e) => panic!("Error getting wind data from copernicusmarine: {}", e),
        };
        let wind_east_data = &wind_data[0];
        let wind_north_data = &wind_data[1];

        // Wind speed and direction
        let wind_east: f64 = wind_east_data[0];
        let wind_north: f64 = wind_north_data[0];
        let wind_angle: f64 = get_north_angle_from_northward_and_eastward_property(wind_east, wind_north);   // Angle in degrees
        let wind_speed = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>((wind_east*wind_east + wind_north*wind_north).sqrt().into());
        wind = PhysVec::new(wind_speed.get::<uom::si::velocity::meter_per_second>(), wind_angle);    // unit [m/s]

        // Get ocean current data from Copernicus
        // "uo" is the eastward sea water velocity and "vo" is the northward sea water velocity
        let ocean_current_data = match simulation.copernicus.as_ref().unwrap().get_f64_values("cmems_mod_glo_phy-cur_anfc_0.083deg_PT6H-i".to_string(), vec!["uo".to_string(), "vo".to_string()], boat_time_now, boat_time_now, longitude, longitude, latitude, latitude, Some(1.0), Some(1.0)){
            Ok(o) => o,
            Err(e) => panic!("Error getting ocean current data from copernicusmarine: {}", e),
        };
        let ocean_current_east_data = &ocean_current_data[0];
        let ocean_current_north_data = &ocean_current_data[1];

        // Ocean current speed and direction
        let ocean_current_east: f64 = ocean_current_east_data[0];
        let ocean_current_north: f64 = ocean_current_north_data[0];
        let ocean_current_angle: f64 = get_north_angle_from_northward_and_eastward_property(ocean_current_east, ocean_current_north);   // Angle in degrees
        let ocean_current_speed = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>((ocean_current_east*ocean_current_east + ocean_current_north*ocean_current_north).sqrt().into());
        ocean_current = PhysVec::new(ocean_current_speed.get::<uom::si::velocity::meter_per_second>(), ocean_current_angle);    // unit [m/s]

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

        // TODO: if we have the data in the ship logs, adjust heading based on last track_angle and heading difference
        // if boat.ship_log.last().is_some() {
        //     if boat.ship_log.last().unwrap().track_angle.is_some() && boat.ship_log.last().unwrap().heading.is_some() {
        //         heading_adjustment = boat.ship_log.last().unwrap().track_angle.unwrap() - boat.ship_log.last().unwrap().heading.unwrap();
        //     }
        // }
        // else {
        //     heading_adjustment = 0.0;
        // }

        // println!("Heading adjustment: {:.4}", heading_adjustment);

        // If absolute relative wind angle is smaller than minimum angle of attack, then use tacking method
        if relative_wind_angle.abs() < boat.min_angle_of_attack.unwrap() {
            boat.hold_tack(wind.angle);
        } // Otherwise relative wind angle is bigger than minimum angle of attack, then go straight towards next waypoint
        else {
            // Set heading to the bearing to next waypoint
            boat.heading = Some(bearing_to_next_waypoint);
            // boat.heading = Some(bearing_to_next_waypoint + heading_adjustment);
        }

        // TODO: use weather data to compute boats actual velocity
        // Find total force on boat
        // force on boat from wind
        // let wind_force: uom::si::f64::Force = uom::si::f64::Force::new::<uom::si::force::newton>(5.0);

        // Velocity of ocean current. Assume that boat is moving with the current

        // Add forces together
        // let force_total: uom::si::f64::Force = wind_force; // + ocean_current_force; // Add other forces here

        // Find total mass
        // let total_mass: uom::si::f64::Mass = boat.mass.unwrap() + boat.cargo_current; // Add cargo mass to boat mass

        // Find acceleration of boat from forces
        // let a: uom::si::f64::Acceleration = force_total / total_mass; // a = F/m, where F is the total force on the boat and m is the mass of the boat

        // Find final velocity of boat from acceleration
        // let final_velocity: uom::si::f64::Velocity = a * uom::si::f64::Time::new::<uom::si::time::day>(simulation.time_step); // final_velocity in meters per second

        // Working velocity is initial velocity plus final velocity divided by 2
        // TODO: implement properly
        // working_velocity = PhysVec::new(wind.magnitude*1.5, boat.heading.unwrap()) + ocean_current;
        working_velocity = PhysVec::new(wind.magnitude*1.5, boat.heading.unwrap());
        // working_velocity = boat.velocity_mean.unwrap(); // (boat.velocity_current.unwrap() + final_velocity) / 2.0; // working_velocity in meters per second

        // Update the current velocity of the boat
        boat.velocity_current = Some(working_velocity);

        // Calculate drag on hull from working velocity
        //TODO make sure all forces are correct


        // Get distance traveled [m] in time step [s]
        travel_dist = working_velocity.magnitude * working_time_step;

        // Move boat forwards along actual direction and log to ship_log
        // If distance traveled is greater than the distance to the next waypoint, set travel_dist to dist_to_next_waypoint and change temp_time_step
        if travel_dist > dist_to_next_waypoint {
            // Set travel distance [m] as distance to next waypoint
            travel_dist = dist_to_next_waypoint;

            // Set temp_time_step [s] to time left in simulation time_step after moving to (now current) waypoint
            let time_passed = dist_to_next_waypoint / working_velocity.magnitude;
            temp_time_step = Some(working_time_step - time_passed);
        }

        // Get the new location of the boat with distance left to travel during timestep and bearing to next waypoint, important to use unit [meter] for travel_dist
        new_location = Haversine.destination(boat.location.unwrap(), working_velocity.angle, travel_dist);
        // If new location is further away from leg line than half of tacking width, tack before moving
        let current_loc_min_dist_to_leg_line = get_min_point_to_great_circle_dist(last_waypoint, next_waypoint, boat.location.unwrap());
        let new_loc_min_dist_to_leg_line = get_min_point_to_great_circle_dist(last_waypoint, next_waypoint, new_location);

        // If currently inside or on boundary but heading out of boundary, tack
        if ((tacking_width/2.0) <  new_loc_min_dist_to_leg_line) && (current_loc_min_dist_to_leg_line <= tacking_width/2.0) {
            // Move to edge of tacking width, tack and go to next iteration of while loop
            // Minimum distance to tacking edge from current location
            let dist_to_tacking_edge = (tacking_width/2.0) - current_loc_min_dist_to_leg_line;
            // Minimum distance from current location to new location
            let dist_to_new_location = new_loc_min_dist_to_leg_line - current_loc_min_dist_to_leg_line;

            // Distance to tacking edge along current heading, see issue #21 for details https://github.com/G0rocks/marine_vessel_simulator/issues/21
            travel_dist = travel_dist * (dist_to_tacking_edge / dist_to_new_location);

            // Update location
            new_location = Haversine.destination(boat.location.unwrap(), boat.heading.unwrap(), travel_dist);

            // Tack
            boat.tack(wind.angle);

            // Set temp_time_step [s] to time left in simulation time_step after moving to tacking edge
            let time_passed = travel_dist / working_velocity.magnitude;
            temp_time_step = Some(working_time_step - time_passed);
        }

        // Update the location of the boat
        boat.location = Some(new_location);

        // Log the new location to the ship log
        let new_log_entry: ShipLogEntry = ShipLogEntry {
            timestamp: boat.ship_log.last().unwrap().timestamp.checked_add(time::Duration::seconds_f64(working_time_step)).expect("Could not add time::Duration to time::UtcDateTime. Maybe an overflow occurred?"),
            coordinates_initial: coordinates_initial,
            coordinates_current: boat.location.unwrap(),
            coordinates_final: coordinates_final,
            cargo_on_board: Some(boat.cargo_current),
            velocity: Some(working_velocity),
            course: Some(course),
            track_angle: Some(Rhumb.bearing(boat.ship_log.last().unwrap().coordinates_current, boat.location.unwrap())),
            heading: boat.heading,
            true_bearing: None,
            draft: None,
            navigation_status: Some(NavigationStatus::UnderwaySailing),
            };

        // Push the new log entry to the ship log
        boat.ship_log.push(new_log_entry);
    } // End while loop

    // Simulation ran through all the iterations, return ship log and error that the simulation did not finish
    // Return the ship log TODO: Move inside for loop
    return Ok("Maximized number of iterations. Stopping simulation".to_string());
}