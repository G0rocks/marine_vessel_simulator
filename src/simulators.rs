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
}

impl Simulation {
    /// Creates a new simulation with the given parameters
    pub fn new(simulation_method: SimMethod, start_times: Vec<UtcDateTime>, time_step: time::Duration, max_iterations: usize, weather_data_file: Option<String>, copernicus: Option<copernicusmarine_rs::Copernicus>) -> Self {
        Simulation {
            simulation_method,
            start_times,
            time_step,
            max_iterations,
            weather_data_file,
            copernicus
        }
    }
}


/// Function that simulates more than one waypoint mission
/// Saves the results of each simulation in the boat.ship_log
pub fn sim_waypoint_missions(boat: &mut Boat, simulation: &Simulation) -> Result<Vec<String>, io::Error> {
    // Init sim_msg:
    let mut sim_msg_vec: Vec<String> = Vec::new();

    // Init progress bar with ETA and elapsed time
    let num_sims = simulation.start_times.len();
    let bar = indicatif::ProgressBar::new(num_sims as u64);
    // Set progress bar style
    bar.set_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar} {pos:>3}/{len:3} ETA:{eta:>1}").unwrap()); //.progress_chars("##-"));
    // If terminal is interactive, use live redraw, otherwise use static redraw
    let is_interactive_terminal = atty::is(atty::Stream::Stdout);
    if is_interactive_terminal {
        // Normal terminal behavior (live redraw)
        bar.set_draw_target(indicatif::ProgressDrawTarget::stdout());
        bar.enable_steady_tick(std::time::Duration::from_millis(500));
    } else {
        // Force static redraw every step to stdout (or to log)
        bar.set_draw_target(indicatif::ProgressDrawTarget::stdout_with_hz(1)); // Or `.stdout_with_hz(1)` for slow redraw
    }
    bar.inc(0);











    
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
        // Update progress bar
        bar.inc(1);
        // If not interactive terminal, print progressbar manually
        if !is_interactive_terminal {
            println!("Elapsed: {:?}, {:?}/{}, ETA: {:?}", bar.elapsed(), bar.position(), num_sims, bar.eta());
        }
    }
    // Finish progress bar
    bar.finish();

    // Run successful, return Ok(sim_msg_vec)
    return Ok(sim_msg_vec);
}

/// Function to simulate the boat following a waypoint mission
/// Is basically a simulation handler that pipes the boat to the correct simulation function
/// TODO: Add what to return, save csv file? Return travel time and more? Also improve documentation
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
        cargo_on_board: boat.cargo_current,
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
                    timestamp: start_time.checked_add(simulation.time_step.checked_mul((i + 1) as i32).expect("Could not multiply, an overflow error probably occurred")).expect("Could not add timestep, an overflow probably occurred"),
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
        timestamp: time::UtcDateTime::new(time::Date::from_calendar_date(start_time.year(), start_time.month(), start_time.day()).expect("Could not make time::Date from values"), time::Time::from_hms(start_time.hour(), start_time.minute(), start_time.second()).expect("Could not make time::Time from values")),
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
        travel_dist = working_velocity * uom::si::f64::Time::new::<uom::si::time::second>(simulation.time_step.whole_seconds() as f64); // travel_dist in meters, https://docs.rs/uom/latest/uom/si/f64/struct.Velocity.html#method.times

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
                        timestamp: boat.ship_log.last().unwrap().timestamp.checked_add(simulation.time_step).expect("Could not add time::Duration to time::UtcDateTime. Maybe an overflow happened?"),
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
                    timestamp: start_time.checked_add(simulation.time_step.checked_mul((i + 1) as i32).expect("Could not multiply time::Duration with value. Maybe an overflow occurred?")).expect("Could not add time::Duration to time::UtcDateTime. Maybe an overflow occurred?"),
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
    // Todo: Add drag
    // if boat.hull_drag_coefficient.is_none() {
    //     return Err(io::Error::new(io::ErrorKind::InvalidInput, "Missing drag coefficient from boat"));
    // }

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
    let coordinates_final = boat.route_plan.as_ref().unwrap()[total_legs - 1].p2;                
    let new_log_entry: ShipLogEntry = ShipLogEntry {
        timestamp: time::UtcDateTime::new(time::Date::from_calendar_date(start_time.year(), start_time.month(), start_time.day()).expect("Could not make time::Date from values"), time::Time::from_hms(start_time.hour(), start_time.minute(), start_time.second()).expect("Could not make time::Time from values")),
        coordinates_initial: coordinates_initial,
        coordinates_current: coordinates_initial,
        coordinates_final: coordinates_final,
        cargo_on_board: boat.cargo_current,
    };
    // Push first ship log entry
    boat.ship_log.push(new_log_entry);

    // Init wind vector
    let mut wind: Wind; // = Wind::new(uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>(5.0), 0.0); // Placeholder for wind speed, should be replaced with actual weather data from file
    // Init next waypoint
    let mut next_waypoint: geo::Point;
    let mut bearing_to_next_waypoint: f64;
    let mut new_location: geo::Point;   // Init
    // Todo: Add number of tacks?

    // Loop through each time step
    for i in 0..simulation.max_iterations {
        // Simulate the boat moving towards the next waypoint
        // Get next waypoint
        next_waypoint = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].p2;
        // Get tacking width from route plan
        let tacking_width = boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].tacking_width;

        // Get weather data for current location from weather data file
        // Wind speed and direction
        // ToDO
        let boat_time_now = boat.ship_log.last().unwrap().timestamp;
        let longitude: f64 = boat.location.expect("Boat has no location").x();
        let latitude: f64 = boat.location.expect("Boat has no location").y();
        // let wind_netcdf_file = simulation.copernicus.as_ref().unwrap().subset("cmems_obs-wind_glo_phy_nrt_l4_0.125deg_PT1H".to_string(), vec!["eastward_wind".to_string(),"northward_wind".to_string()], boat_time_now, boat_time_now, -7.1, 7.2, -7.3, 7.4);
        
        // Get wind and oc (ocean current) data from Copernicus
        let wind_netcdf_file = simulation.copernicus.as_ref().unwrap().subset("cmems_obs-wind_glo_phy_nrt_l4_0.125deg_PT1H".to_string(), vec!["eastward_wind".to_string(),"northward_wind".to_string()], boat_time_now, boat_time_now, longitude, longitude, latitude, latitude);
        // let oc_netcdf_file = simulation.copernicus.as_ref().unwrap().subset("cmems_mod_glo_phy-cur_anfc_0.083deg_PT6H-i".to_string(), vec!["uo".to_string(),"vo".to_string()], boat_time_now, boat_time_now, longitude, longitude, latitude, latitude);    // "uo" is the eastward sea water velocity and "vo" is the northward sea water velocity

        let wind_netcdf_root =  wind_netcdf_file.root().expect("Could not get netcdf root from netcdf file");
        // let oc_netcdf_root =  oc_netcdf_file.root().expect("Could not get netcdf root from netcdf file");

        // Get variables from netcdf file
        // let time_stamp = wind_netcdf_root.variable("time").expect("No variable: time");
        // let lat = wind_netcdf_root.variable("latitude").expect("No variable: latitude");
        // let lon = wind_netcdf_root.variable("longitude").expect("No variable: longitude");
        let wind_east = wind_netcdf_root.variable("eastward_wind").expect("No variable: eastward_wind");
        let wind_north = wind_netcdf_root.variable("northward_wind").expect("No northward_wind var");
        // let oc_east = oc_netcdf_root.variable("uo").expect("No variable: eastward_wind");
        // let oc_north = oc_netcdf_root.variable("vo").expect("No northward_wind var");

        // let wind_east_scale_factor_attr_val = wind_east.attribute("scale_factor").expect("No scale factor found").value().expect("Could not get scale factor value");
        // let wind_east_scale_factor = match wind_east_scale_factor_attr_val {
        //     netcdf::AttributeValue::Double(v) => v as f32,
        //     _ => panic!("scale_factor was not a Double"),
        // };
        // println!("Scale factor for wind_east: {:?}", wind_east_scale_factor);
        
        // let wind_east_add_offset_attr_val = wind_east.attribute("add_offset").expect("No scale factor found").value().expect("Could not get scale factor value");
        // let wind_east_add_offset = match wind_east_add_offset_attr_val {
        //     netcdf::AttributeValue::Double(v) => v as f32,
        //     _ => panic!("scale_factor was not a Double"),
        // };
        // println!("Add offset for wind_east: {:?}", wind_east_add_offset);

        // let wind_east_fill_value_attr_val = wind_east.attribute("fill_value").expect("No fill value found").value().expect("Could not get fill value");
        // let wind_east_fill_value = match wind_east_fill_value_attr_val {
        //     netcdf::AttributeValue::Double(v) => v as f32,
        //     _ => panic!("fill_value was not a Double"),
        // };
        // println!("Fill value for wind_east: {:?}", wind_east_fill_value);

        // Get data vectors from variables
        // let time_data: Vec<i64> = time_stamp.get_values(netcdf::Extents::All).expect("Failed to read time stamps");
        // let lat_data: Vec<f64> = lat.get_values(netcdf::Extents::All).expect("Failed to read latitude");
        // let lon_data: Vec<f64> = lon.get_values(netcdf::Extents::All).expect("Failed to read latitude");
        let wind_east_data: Vec<f32> = wind_east.get_values(netcdf::Extents::All).expect("Failed to read eastward wind");    // Scale factor is 0.01 according to page 21 of https://documentation.marine.copernicus.eu/PUM/CMEMS-WIND-PUM-012-004-006.pdf
        let wind_north_data: Vec<f32> = wind_north.get_values(netcdf::Extents::All).expect("Failed to read northward wind");    // Scale factor is 0.01 according to page 21 of https://documentation.marine.copernicus.eu/PUM/CMEMS-WIND-PUM-012-004-006.pdf
        // let oc_east_data: Vec<f32> = oc_east.get_values(netcdf::Extents::All).expect("Failed to read eastward ocean current");    // Scale factor is 0.01 according to page 21 of https://documentation.marine.copernicus.eu/PUM/CMEMS-WIND-PUM-012-004-006.pdf
        // let oc_north_data: Vec<f32> = oc_north.get_values(netcdf::Extents::All).expect("Failed to read northward ocean current");    // Scale factor is 0.01 according to page 21 of https://documentation.marine.copernicus.eu/PUM/CMEMS-WIND-PUM-012-004-006.pdf
        // println!("Timestamp: {:?}", copernicusmarine_rs::secs_since_1990_01_01_0_to_utcdatetime(time_data[0]));
        // println!("Latitude: {:?}", lat_data[0]);
        // println!("Longitude: {:?}", lon_data[0]);
        // println!("east wind 2: {:.02}", wind_east_data[1]);
        // println!("north wind 2: {:.02}", wind_north_data[1]);
        // println!("Wind east: {:.02}", wind_east_data[0]*wind_east_scale_factor + wind_east_add_offset);
        // println!("Wind north: {:.02}", wind_north_data[0]*0.01);
        // println!("Ocean current east: {:.02}", oc_east_data[0]);
        // println!("Ocean current north: {:.02}", oc_north_data[0]);

        // Todo: Try to delete downloaded file before leaving directory to conserve available storage space on computer
        // Copy netcdf_file name
        let wind_filename = wind_netcdf_file.path().expect("Could not get netcdf file path").clone();
        // Stop using netcdf_file so it can be deleted
        wind_netcdf_file.close().expect("Could not close netcdf file");
        // Move into output path directory
        let start_dir = std::env::current_dir().expect("Could not get current directory");
        // Change directory
        std::env::set_current_dir(std::path::Path::new(&simulation.copernicus.clone().unwrap().output_path)).expect("Error changing directories");
        // Try to delete the file
        match std::fs::remove_file(&wind_filename) {
            Ok(_) => {}
            Err(e) => {
                println!("Could not delete file {:?}: {}", &wind_filename, e);
                    let f = std::fs::File::open(wind_filename)?;
                    let metadata = f.metadata().expect("Oh no, NO METADATA FOUND!");
                    let permissions = metadata.permissions();
                println!("Permissions: {:?}", permissions);
            }
        }

        // Move back into directory
        std::env::set_current_dir(start_dir).expect("Error changing directories");


        // panic!("Stop run for debugging");


        let wind_east: f64 = wind_east_data[0].into();
        let wind_east = wind_east*0.01;
        let wind_north: f64 = wind_north_data[0].into();
        let wind_north = wind_north * 0.01;
        let angle: f64 = north_angle_from_north_and_eastward_wind(wind_east, wind_north);   // Angle in degrees
        
        // println!("Wind north: {}\nWind east: {}", wind_north, wind_east);
        let wind_speed = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>((wind_east*wind_east + wind_north*wind_north).sqrt().into());
        wind = Wind::new(wind_speed, angle);
        // println!("WIND TEST: {:?}", wind_test);
        // println!("WIND TEST: {:?}", wind);


        // todo!("Fix angle calculations!");    // Todo verify angle calculation

        //Todo: Let's only run once while debugging
        // panic!("Stop run");




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





        // Todo: use weather data to compute boats actual velocity
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
        // todo: implement properly
        working_velocity = wind.speed*1.5;
        // working_velocity = boat.velocity_mean.unwrap(); // (boat.velocity_current.unwrap() + final_velocity) / 2.0; // working_velocity in meters per second

        // Update the current velocity of the boat
        // boat.velocity_current = Some(working_velocity);

        // Calculate drag on hull from working velocity
        //Todo make sure all forces are correct, for now we just want to see the boat tack on the visualization


        // Get distance traveled in time step
        travel_dist = working_velocity * uom::si::f64::Time::new::<uom::si::time::second>(simulation.time_step.whole_seconds() as f64); // travel_dist in meters, https://docs.rs/uom/latest/uom/si/f64/struct.Velocity.html#method.times
        
        

        // While still have some distance left to travel during time step
        while travel_dist.get::<uom::si::length::meter>() > 0.0 {
            // Get next waypoint
            next_waypoint = boat.route_plan.as_ref().expect("Route plan missing?")[(boat.current_leg.unwrap()-1) as usize].p2;

            // Get distance to next waypoint from current location
            let dist_to_next_waypoint: uom::si::f64::Length = haversine_distance_uom_units(boat.location.unwrap(), next_waypoint);

            // if distance traveled is greater than the distance to the next waypoint and the heading is the bearing to the next waypoint, move to next waypoint, update current leg number and go to next while loop iteration
            if travel_dist > dist_to_next_waypoint && boat.heading.unwrap() == bearing_to_next_waypoint {
                // Move to next waypoint
                boat.location = Some(next_waypoint);

                // If the boat has reached the last waypoint, stop the simulation
                if boat.location.unwrap() == coordinates_final {
                    // Update ship logs with last point
                    let new_log_entry: ShipLogEntry = ShipLogEntry {
                        // Set timestamp to last shiplogentry + time step
                        timestamp: boat.ship_log.last().unwrap().timestamp.checked_add(simulation.time_step).expect("Could not add time::Duration to time::UtcDateTime. Maybe an overflow occurred?"),
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
                new_location = Haversine.destination(boat.location.unwrap(), boat.heading.unwrap(), travel_dist.get::<uom::si::length::meter>()); // travel_dist in meters, https://docs.rs/geo/0.30.0/geo/algorithm/line_measures/metric_spaces/struct.HaversineMeasure.html#method.destination
                // If new location is further away from leg line than half of tacking width, tack before moving
                let min_dist_to_leg_line = min_haversine_distance(boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].p1, next_waypoint, new_location);

                if tacking_width <  min_dist_to_leg_line {
                    boat.tack(wind.angle);
                    // print debug message current distance to leg line
                    // println!("Current distance to leg line: {:.1} km", min_haversine_distance(boat.route_plan.as_ref().unwrap()[(boat.current_leg.unwrap()-1) as usize].p1, next_waypoint, boat.location.unwrap()).get::<uom::si::length::kilometer>());
                    // println!("Min dist from new loacation to leg line: {:.1} km", min_dist_to_leg_line.get::<uom::si::length::kilometer>());
                    new_location = Haversine.destination(boat.location.unwrap(), boat.heading.unwrap(), travel_dist.get::<uom::si::length::meter>()); // travel_dist in meters, https://docs.rs/geo/0.30.0/geo/algorithm/line_measures/metric_spaces/struct.HaversineMeasure.html#method.destination
                }


                // Update the location of the boat
                boat.location = Some(new_location);

                // Log the new location to the ship log
                let new_log_entry: ShipLogEntry = ShipLogEntry {
                    timestamp: start_time.checked_add(simulation.time_step.checked_mul((i + 1) as i32).expect("Could not multiply time::Duration with value")).expect("Could not add time::Duration to time::UtcDateTime. Maybe an overflow occurred?"),
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