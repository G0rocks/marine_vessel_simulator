/// Marine vessel simulator simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-04-14
/// Note that a dimensional anlysis is not performed in this code using uom (https://crates.io/crates/uom)
/// ## To do
/// Make another crate, sailplanner, that can make route plans for marine vessels.

/// External crates
use csv;    // CSV reader to read csv files
use uom;    // Units of measurement. Makes sure that the correct units are used for every calculation
use geo::{self, haversine_distance, Distance};    // Geographical calculations. Used to calculate the distance between two coordinates
use year_helper; // Year helper to calculate the number of days in a year based on the month and if it's a leap year or not
use io;     // To use Result type with Ok and Err

// Structs and enums
//----------------------------------------------------
/// enum of boat propulsion system types
pub enum Propulsion {
    Sail,
    // Diesel,
    // Electric,
    // Hybrid,
    // Kite,
    // FlettnerRotor,
    // Nuclear,
    // Other,
}

/// Struct to hold sailing leg data
#[derive(Debug)]
pub struct SailingLeg {
    pub p1: geo::Point,
    pub p2: geo::Point,
    pub tacking_width: uom::si::f64::Length,
}

/// Struct to hold ship long entry
#[derive(Debug)]
pub struct ShipLogEntry {
    pub timestamp: uom::si::f64::Time,
    pub coordinates_initial: geo::Point,
    pub coordinates_current: geo::Point,
    pub coordinates_final: geo::Point,
    pub cargo_on_board: uom::si::f64::Mass,
}

/// Enum of simulation methods
pub enum SimMethod {
    /// Constant velocity, uses the mean velocity of the boat
    ConstVelocity,
    // Use the mean and std of the boat speed
    // Mean_and_STD_Velocity,
    // Use downloaded weather data from file
    // Weather_data_from_file,
    // Use the copernicus weather data from the past for the exact location of the boat to simulate the boat movements
    // Copernicus_Weather_Data,
    // Use the copernicus weather forecast data for the exact location of the boat to simulate the boat movements
    // Copernicus_Weather_Forecast,
}


/// Struct to hold boat metadata
/// All fields are optional, so that the struct can be created without knowing all the values
pub struct Boat {
    pub imo: Option<u32>,
    pub name: Option<String>,
    pub min_angle_of_attack: Option<uom::si::f64::Angle>,
    pub location: Option<geo::Point>,
    pub heading: Option<uom::si::f64::Angle>,
    pub route_plan: Option<Vec<SailingLeg>>,
    pub current_leg: Option<u32>,
    pub length: Option<uom::si::f64::Length>,
    pub width: Option<uom::si::f64::Length>,
    pub draft: Option<uom::si::f64::Length>,
    pub mass: Option<uom::si::f64::Mass>,
    pub propulsion: Option<Propulsion>,
    pub velocity_mean: Option<uom::si::f64::Velocity>,
    pub velocity_std: Option<uom::si::f64::Velocity>,
    pub cargo_max_capacity: Option<uom::si::f64::Mass>,
    pub cargo_current: uom::si::f64::Mass,
    pub cargo_mean: Option<uom::si::f64::Mass>,
    pub cargo_std: Option<uom::si::f64::Mass>,
    pub simulation_method: Option<SimMethod>,
}

// Implementation of the Boat struct
//----------------------------------------------------
impl Boat {
    pub fn new() -> Boat {
        Boat {
            imo: None,
            name: None,
            min_angle_of_attack: None,
            location: None,
            heading: None,
            route_plan: None,
            current_leg: None,
            length: None,
            width: None,
            draft: None,
            mass: None,
            propulsion: None,
            velocity_mean: None,
            velocity_std: None,
            cargo_max_capacity: None,
            cargo_current: uom::si::f64::Mass::new::<uom::si::mass::ton>(0.0),
            cargo_mean: None,
            cargo_std: None,
            simulation_method: None,
        }
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    pub fn load_cargo(&mut self, cargo: uom::si::f64::Mass) {
        // Check if the cargo is too heavy
        match self.cargo_max_capacity {
            Some(max_capacity) => {
                if cargo > max_capacity {
                    panic!("Cargo is too heavy");
                }
            }
            None => {}  // No max capacity set, so do nothing
        }

        // Set the cargo
        self.cargo_current = cargo;
    }

    /// Function to simulate the boat moving to a new location
    /// This function takes in the timestep and updates the location of the boat after the timestep
    pub fn sim_step(&mut self, time_step: uom::si::f64::Time){
        todo!();
    }
}



// Functions
//----------------------------------------------------

/// This function evaluates the cargo shipping logs from a CSV file and calculates the mean and standard deviation of the speed and cargo delivery values. The CSV file is expected to have the following columns:<br>
/// timestamp;coordinates_initial;coordinates_current;coordinates_final;cargo_on_board (weight in tons)<br><br>
/// The delimiter is a semicolon.
/// file_path: Path to the CSV file
/// distance: The total sailing distance. Note if distance = 0 the function evaluates the sailing distance by drawing a straight line for each leg of the trip 
/// Notes:
/// Timestamps are expected to be in the ISO format of YYYY-MM-DD hh:mm.
/// Coordinates are expected to be in the format of ISO 6709 using decimal places with a comma between latitude and longitude. "latitude,longitude" (e.g., "52.5200,13.4050") 
/// The first current coordinate must match the initial coordinate and the last current coordinate must match the final coordinate.
/// Example:
/// let filename: &str = "../data/mydata.csv";
/// let distance: u64 = 8000; // Distance in km
/// let (speed_mean, speed_std, cargo_mean, cargo_std) = evaluate_cargo_shipping_logs(filename, distance);
pub fn evaluate_cargo_shipping_logs(file_path: &str) ->
    (uom::si::f64::Velocity, uom::si::f64::Velocity, Option<uom::si::f64::Mass>, Option<uom::si::f64::Mass>, uom::si::f64::Time, uom::si::f64::Time, u64) {

    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(file_path)
        .expect("Failed to open the file");

    // Initialize variables to store the sum and count of speed and cargo values
    let mut speed_vec: Vec<uom::si::f64::Velocity> = Vec::new();
    let mut cargo_vec: Vec<Option<uom::si::f64::Mass>> = Vec::new();
    let mut travel_time_vec: Vec<uom::si::f64::Time> = Vec::new();

    // Init empty csv column variable
    let mut timestamp: uom::si::f64::Time;
    let mut coordinates_initial: geo::Point;
    let mut coordinates_current: geo::Point;
    let mut coordinates_final: geo::Point;
    let mut cargo_on_board_option: Option<uom::si::f64::Mass>;         // weight in tons

    // Init empty working variables
    let mut dist: uom::si::f64::Length = uom::si::f64::Length::new::<uom::si::length::meter>(0.0);
    let mut start_time: uom::si::f64::Time = uom::si::f64::Time::new::<uom::si::time::second>(0.0);
    let mut cargo_on_trip: Option<uom::si::f64::Mass> = None;
    let mut num_trips: u64 = 0;
    let mut coordinates_last: geo::Point = geo::Point::new(0.0, 0.0);

    // Iterate through each line of the CSV file to calculate the mean and standard deviation of speed and cargo values, using each leg (each leg is 2 points) of the trip/s
    for result in csv_reader.records() {
        match result {
            Ok(leg) => {
                // Get all values in row
                timestamp = string_to_timestamp(leg.get(0).expect("No timestampe found").to_string());
                // Convert the string to a geo::Coord object
                coordinates_initial = string_to_point(leg.get(1).expect("No initial coordinate found").to_string());
                coordinates_current = string_to_point(leg.get(2).expect("No initial coordinate found").to_string());
                coordinates_final = string_to_point(leg.get(3).expect("No initial coordinate found").to_string());
                cargo_on_board_option = string_to_tons(leg.get(4).unwrap().to_string());
                if cargo_on_board_option.is_some() {
                    cargo_on_trip = cargo_on_board_option;
                }

                // If current coord is not inital or final this is a working point,
                if coordinates_current != coordinates_initial && coordinates_current != coordinates_final {
                    // Add the distance between the coordinates
                    dist = dist + haversine_distance_uom_units(coordinates_last, coordinates_current);
                }   // Otherwise, if initial coordinate, the trip just started
                else if coordinates_current == coordinates_initial {
                    // Increment the number of trips
                    num_trips += 1;
                    // Log start time
                    start_time = timestamp;
                    // Set the last coordinates to the initial coordinates
                    coordinates_last = coordinates_initial;
                }   // Otherise, if final coordinate, the trip just ended
                else if coordinates_current == coordinates_final {
                    // Add the distance between the last coordinates and the final coordinates  
                    dist = dist + haversine_distance_uom_units(coordinates_last, coordinates_final);
                    // Calculate the speed
                    let speed = dist / (timestamp - start_time);

                    // Add speed and cargo values to speed and cargo vectors
                    speed_vec.push(speed);
                    cargo_vec.push(cargo_on_trip);
                    travel_time_vec.push(timestamp - start_time);
                    
                    // Reset distance
                    dist = uom::si::f64::Length::new::<uom::si::length::meter>(0.0);
                    // Reset cargo
                    cargo_on_trip = None;
                }
            }
            // Handle the error if the leg cannot be read
            Err(err) => {
                eprintln!("Error reading leg: {}", err);
            }
        }
    }


    // Calculate the mean and standard deviation of the speed, cargo and travel time vectors
    let speed_avg: uom::si::f64::Velocity;
    let speed_std: uom::si::f64::Velocity;
    let cargo_avg: Option<uom::si::f64::Mass>;
    let cargo_std: Option<uom::si::f64::Mass>;
    let travel_time_avg: uom::si::f64::Time;
    let travel_time_std: uom::si::f64::Time;

    (speed_avg, speed_std) = get_speed_mean_and_std(&speed_vec);
    (cargo_avg, cargo_std) = get_weight_mean_and_std(&cargo_vec);
    (travel_time_avg, travel_time_std) = get_time_mean_and_std(&travel_time_vec);

    // Return the values
    return (speed_avg, speed_std, cargo_avg, cargo_std, travel_time_avg, travel_time_std, num_trips)
}


/// Function to simulate the boat following a waypoint mission
/// Is basically a simulation handler that pipes the boat to the correct simulation function
/// TODO: Add what to return, save csv file? Return travel time and more? Also improve documentation
pub fn sim_waypoint_mission(boat: &mut Boat, start_time: uom::si::f64::Time, time_step: uom::si::f64::Time, results_file_path: &str) -> Result<String, io::Error> {
    // Check if the boat has a route plan, if no route plan
    if boat.route_plan.is_none() {
        return Err("No route plan found");
    }

    // match simulation method and run corresponding simulation function
    match boat.simulation_method {
        Some(SimMethod::ConstVelocity) => {
            // Simulate the boat using constant velocity
            sim_const_velocity(boat: &mut Boat, start_time: uom::si::f64::Time, time_step: uom::si::f64::Time, results_file_path: &str);
        }
        // Add other simulation methods here
        _ => panic!("Invalid simulation method"),
    }
}


// Simulators
//----------------------------------------------------
/// Simulates the boat using constant velocity
pub fn sim_waypoint_mission_constanct_velocity(boat: &mut Boat, start_time: uom::si::f64::Time, time_step: uom::si::f64::Time, max_iterations: usize, results_file_path: &str) -> Result<String, io::Error> {
    // Set boats current location to the first waypoint
    boat.location = Some(boat.route_plan.as_ref().expect("Route plan missing?")[0].p1);
    // Set current leg to 1
    boat.current_leg = Some(1);

    // Init empty ship log
    let mut ship_log: Vec<ShipLogEntry> = Vec::new();

    // Loop through each time step
    for i in [0..max_iterations] {
        // Simulate the boat moving towards the next waypoint
        // Get next waypoint
        let next_waypoint: geo::Point = boat.route_plan.expect("Route plan missing?")[boat.current_leg.unwrap() as usize].p2;
        // Get distance to next waypoint from current location
        let dist: uom::si::f64::Length = haversine_distance_uom_units(boat.location.unwrap(), boat.route_plan.as_ref().expect("Route plan missing?").p2);

        // Get distance traveled in time step

        // While still have some distance left to travel during time step
            // if distance traveled is greater than the distance to the next waypoint move to next waypoint, update current leg number and go to next while loop iteration
            // If the boat has reached the last waypoint, stop the simulation

            // Get heading to next waypoint
            // Get the new location of the boat with distance left to travel during timestep and heading to next waypoint

        // Add the new location to the ship log
        
    }

    // Write the results to a CSV file
}





// Helper functions
//----------------------------------------------------
/// Converts a string into an uom::si::f64::Time object
/// time_string: The string to convert in the format YYYY-MM-DD hh:mm
/// Example:
/// let my_timestamp: uom::si::f64::Time = str_to_coordinate("52.5200,13.4050");
pub fn string_to_timestamp(time_string: String) -> uom::si::f64::Time {
    // Remove all whitespaces in string
    let working_str: &str = (&time_string[..]).trim();

    // Check if the string is valid
    if working_str.len() != 16 {
        panic!("Invalid time format");
    }

    // Get parts from string
    let year_i32:    i32 = working_str[0..4].parse::<i32>().expect("Invalid year");
    let year_uom:   uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::year>(working_str[0..4].parse::<f64>().expect("Invalid year"));
    let month_u8:   u8 = working_str[5..7].parse::<u8>().expect("Invalid month");
    let day:    uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::day>(working_str[8..10].parse::<f64>().expect("Invalid day"));
    let hour:   uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::hour>(working_str[11..13].parse::<f64>().expect("Invalid hour"));
    let minute: uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::minute>(working_str[14..16].parse::<f64>().expect("Invalid minute"));

    let days: uom::si::f64::Time = days_from_month(month_u8, year_i32) + day;

    // Attempt to parse the string into a uom::si::f64::Time object
    let time_out: uom::si::f64::Time = year_uom + days + hour + minute;
    
    // Return
    return time_out;
}

// Finds out how many days have passed in the year since first of january that year until the beginning of the given month
fn days_from_month(month: u8, year: i32) -> uom::si::f64::Time {
    // Check if the month is valid
    if (month < 1) || (month > 12) {
        panic!("Invalid month");
    }

    // Init days
    let mut days: u64 = 0;

    // for each month, add number of days in month to days
    for i in 1..month {
        match i {
            1 => days += 31,    // 31 days in January
            2 => days += 28,    // 28 days in February, leap year is handled later
            3 => days += 31,    // 31 days in March
            4 => days += 30,    // 30 days in April
            5 => days += 31,    // 31 days in May
            6 => days += 30,    // 30 days in June
            7 => days += 31,    // 31 days in July
            8 => days += 31,    // 31 days in August
            9 => days += 30,    // 30 days in September
            10 => days += 31,   // 31 days in October
            11 => days += 30,   // 30 days in November
            12 => days += 31,   // 31 days in December
            _ => panic!("Invalid month"),
        }
    }

    // If leap year and after january and february, add 1 day to February
    if year_helper::is_leap_year(year) && (month > 2) {
        days += 1;
    }

    // Return the number of days
    return uom::si::f64::Time::new::<uom::si::time::day>(days as f64);
}

/// Converts a string into a geo::Point object
/// point_string: The string to convert
/// Example:
/// let my_coord: geo::Point = string_to_point("52.5200,13.4050");
/// Note that the output is a geo::Point::new(longitude, latitude) but the input string must be in the format of latitude,longitude so the order is reversed
pub fn string_to_point(coord_string: String) -> geo::Point {
    // Remove all spaces in string
    let coord_str_vec: Vec<&str> = coord_string.trim().split(',').collect();

    // Check if the coordinates are valid, should have latitude and longitude
    if coord_str_vec.len() != 2 {
        panic!("Invalid coordinate format");
    }
        
    // Parse the latitude and longitude as f64
    let mut latitude: f64 = coord_str_vec[0].trim().parse::<f64>().expect("Invalid latitude");
    let mut longitude: f64 = coord_str_vec[1].trim().parse::<f64>().expect("Invalid longitude");

    // Make sure longitude is between -180° and 360°
    while longitude < -180.0 {
        longitude += 360.0;
    }
    while longitude > 360.0 {
        longitude -= 360.0;
    }

    // Make sure latitude is between -90° and 90°
    while latitude < -90.0 {
        latitude += 180.0;
    }
    while latitude > 90.0 {
        latitude -= 180.0;
    }

    // Make return point
    let return_point = geo::Point::new(longitude, latitude);
    
    return return_point;
}

/// Calculates the haversine distance between two points and returns the distance in uom::si::f64::Length
pub fn haversine_distance_uom_units(p1: geo::Point, p2: geo::Point) -> uom::si::f64::Length {
    // Calculate the haversine distance between two points
    let dist: uom::si::f64::Length = uom::si::length::Length::new::<uom::si::length::meter>(geo::Haversine.distance(p1, p2));
    return dist;
}


/// Converts a string into a uom::si::f64::Mass object
/// cargo_string: The string to convert, must be in metric tons (1 metric ton = 1000 kg)
/// Example:
/// let my_tons: uom::si::f64::Mass = string_to_tons("500.3");
pub fn string_to_tons(cargo_string: String) -> Option<uom::si::f64::Mass> {
    // Remove all spaces in string
    let cargo_str: &str = (&cargo_string[..]).trim();
    
    // Check if the string is valid
    if cargo_str.len() == 0 {
        return None;
    }

    // Parse the cargo as f64
    let cargo: f64 = cargo_str.parse::<f64>().expect("Invalid cargo");

    // Make return coordinate
    let return_cargo: Option<uom::si::f64::Mass> = Some(uom::si::f64::Mass::new::<uom::si::mass::ton>(cargo));
    return return_cargo;
}


/// Returns the average and standard deviation of a vector of uom::si::f64::Velocity objects
/// speed_vec: The vector of uom::si::f64::Velocity objects
/// Example:
/// let (my_mean, my_std) = get_speed_mean_and_std(&my_vec);
pub fn get_speed_mean_and_std(speed_vec: &Vec<uom::si::f64::Velocity>) ->
    (uom::si::f64::Velocity, uom::si::f64::Velocity) {
    // Calculate the mean of the speed vector
    let mut tot_speed = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>(0.0);

    // loop through vector, add all values to tot_speed
    for speed in speed_vec {
        tot_speed = tot_speed + *speed;
    }
    // Find mean
    let speed_mean: uom::si::f64::Velocity = tot_speed / speed_vec.len() as f64;
    let speed_mean_f64: f64 = speed_mean.get::<uom::si::velocity::meter_per_second>();

    // Calculate the standard deviation of the speed vector
    let mut variance: f64 = 0.0;

    // loop through vector, add all values to variance, then divide by number of values -1 to create variance
    for speed in speed_vec {
        variance = variance + (speed.get::<uom::si::velocity::meter_per_second>() - speed_mean_f64).powi(2);
    }
    variance = variance / ((speed_vec.len() - 1) as f64);

    // Find standard deviation from variance
    let speed_std: uom::si::f64::Velocity = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>(variance.sqrt());

    // Return the mean and standard deviation
    return (speed_mean, speed_std);
}

/// Returns the average and standard deviation of a vector of Option<uom::si::f64::Mass> objects
/// cargo_vec: The vector of Option<uom::si::f64::Mass> objects
/// Example:
/// let (my_mean, my_std) = get_speed_mean_and_std(&my_vec);
pub fn get_weight_mean_and_std(weight_vec: &Vec<Option<uom::si::f64::Mass>>) ->
    (Option<uom::si::f64::Mass>, Option<uom::si::f64::Mass>) {
    // Calculate the mean of the vector
    let mut tot_weight = uom::si::f64::Mass::new::<uom::si::mass::kilogram>(0.0);
    let mut counter: u64 = 0;
    let mut useful_weight_vec: Vec<uom::si::f64::Mass> = Vec::new();

    // loop through vector, add all values to tot_weight, count how many have some value
    for weight in  weight_vec{
       match weight {
            // If there is some value, add it to the total, the useful_weight_vec and count it, otherwise do nothing
            Some(w) => {
                tot_weight = tot_weight + *w;
                useful_weight_vec.push(*w);
                counter += 1;
            }
            None => {}
        };
    }

    // If there are no values, return None
    if counter == 0 {
        return (None, None);
    }

    // Find mean
    let weight_mean: uom::si::f64::Mass = tot_weight / counter as f64;
    let weight_mean_f64: f64 = weight_mean.get::<uom::si::mass::kilogram>();

    // Calculate the standard deviation of the speed vector
    let mut variance: f64 = 0.0;

    // loop through vector, add all values to variance, then divide by number of values -1 to create variance
    for weight in useful_weight_vec {
        variance = variance + (weight.get::<uom::si::mass::kilogram>() - weight_mean_f64).powi(2);
    }
    variance = variance / ((counter - 1) as f64);

    // Find standard deviation from variance
    let weight_std: uom::si::f64::Mass = uom::si::f64::Mass::new::<uom::si::mass::kilogram>(variance.sqrt());

    // Return the mean and standard deviation
    return (Some(weight_mean), Some(weight_std));
}


/// Returns the average and standard deviation of a vector of uom::si::f64::Time objects
/// time_vec: The vector of uom::si::f64::Time objects
/// Example:
/// let (my_mean, my_std) = get_time_mean_and_std(&my_vec);
pub fn get_time_mean_and_std(time_vec: &Vec<uom::si::f64::Time>) ->
    (uom::si::f64::Time, uom::si::f64::Time) {
    // Calculate the mean of the vector
    let mut tot_time = uom::si::f64::Time::new::<uom::si::time::day>(0.0);

    // loop through vector, add all values to the total
    for time in time_vec {
        tot_time = tot_time + *time;
    }
    // Find mean
    let time_mean: uom::si::f64::Time = tot_time / time_vec.len() as f64;
    let time_mean_f64: f64 = time_mean.get::<uom::si::time::day>();

    // Calculate the standard deviation of the vector
    let mut variance: f64 = 0.0;

    // loop through vector, add all values to variance, then divide by number of values -1 to create variance
    for time in time_vec {
        variance = variance + (time.get::<uom::si::time::day>() - time_mean_f64).powi(2);
    }
    variance = variance / ((time_vec.len() - 1) as f64);

    // Find standard deviation from variance
    let time_std: uom::si::f64::Time = uom::si::f64::Time::new::<uom::si::time::day>(variance.sqrt());

    // Return the mean and standard deviation
    return (time_mean, time_std);
}


/// Loads route plan from a CSV file
/// Returns a vector of SailingLeg objects where each entry is a a leg of the trip
/// The CSV file is expected to have the following columns in order but the header names are not important:
/// Leg number;start_latitude;start_longitude;end_latitude;end_longitude;tacking_width[meters]
/// The delimiter is a semicolon.
/// file_path: Path to the CSV file
/// Example:
/// let file_path: &str = "my_route_plan.csv";
pub fn load_route_plan(file_path: &str) -> Vec<SailingLeg> {
    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(file_path)
        .expect("Failed to open the file");

    // Initialize a vector to store the route plan
    let mut route_plan: Vec<SailingLeg> = Vec::new();

    // Iterate through each line of the CSV file and add the coordinates to the route plan
    for result in csv_reader.records() {
        match result {
            Ok(leg) => {
                // Get the SailingLeg data from the CSV file
                // First column is the leg number, so we skip it
                // Start_coord
                let start_lat = leg.get(1).expect("Start latitude missing").to_string();
                let start_long = leg.get(2).expect("Start longitude missing").to_string();
                // End_coord
                let end_lat = leg.get(3).expect("End latitude missing").to_string();
                let end_long = leg.get(4).expect("End longitude missing").to_string();
                // Tacking width
                let tacking_width = leg.get(5).expect("Tacking width missing").to_string();

                // Make a SailingLeg object
                let temp_sailing_leg: SailingLeg = SailingLeg {
                    p1: string_to_point(format!("{},{}", start_lat, start_long)),
                    p2: string_to_point(format!("{},{}", end_lat, end_long)),
                    tacking_width: uom::si::f64::Length::new::<uom::si::length::meter>(tacking_width.parse::<f64>().expect("Invalid tacking width")),
                };

                // Add the SailingLeg object to the route plan
                route_plan.push(temp_sailing_leg);
            }
            Err(err) => {
                eprintln!("Error reading leg: {}", err);
            }
        }
    }

    // Return the route plan
    return route_plan;
}


// Set up tests here
