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

// Structs and enums
//----------------------------------------------------
/// enum of boat propulsion system types
pub enum Propulsion {
    Diesel,
    Electric,
    Hybrid,
    Sail,
    Kite,
    FlettnerRotor,
    Nuclear,
    Other,
}


/// Struct to hold boat metadata
/// All fields are optional, so that the struct can be created without knowing all the values
pub struct Boat {
    pub imo: Option<u32>,
    pub name: Option<String>,
    pub location: Option<geo::Point>,
    pub route_plan: Option<Vec<geo::Point>>,
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
}

impl Boat {
    pub fn new() -> Boat {
        Boat {
            imo: None,
            name: None,
            location: None,
            route_plan: None,
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
    (uom::si::f64::Velocity, uom::si::f64::Velocity, Option<uom::si::f64::Mass>, Option<uom::si::f64::Mass>, u64) {

    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(file_path)
        .expect("Failed to open the file");

    // Initialize variables to store the sum and count of speed and cargo values
    let mut speed_vec: Vec<uom::si::f64::Velocity> = Vec::new();
    let mut cargo_vec: Vec<Option<uom::si::f64::Mass>> = Vec::new();

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


    // Calculate the mean and standard deviation of the speed and cargo vectors
    let speed_avg: uom::si::f64::Velocity;
    let speed_std: uom::si::f64::Velocity;
    let cargo_avg: Option<uom::si::f64::Mass>;
    let cargo_std: Option<uom::si::f64::Mass>;

    (speed_avg, speed_std) = get_speed_mean_and_std(&speed_vec);
    (cargo_avg, cargo_std) = get_weight_mean_and_std(&cargo_vec);

    // Return the values
    return (speed_avg, speed_std, cargo_avg, cargo_std, num_trips)
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
/// let my_coord: geo::Point = str_to_coordinate("52.5200,13.4050");
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

// Set up tests here
