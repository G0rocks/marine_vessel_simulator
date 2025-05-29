/// Marine vessel simulator simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-04-14
/// Note that a dimensional anlysis is not performed in this code using uom (https://crates.io/crates/uom)
/// ## To do
/// Make another crate, sailplanner, that can make route plans for marine vessels.

/// External crates
use csv;    // CSV reader to read csv files
use uom::{self};    // Units of measurement. Makes sure that the correct units are used for every calculation
use geo::{self, Haversine, Bearing, Distance, Destination};    // Geographical calculations. Used to calculate the distance between two coordinates and bearings
use year_helper; // Year helper to calculate the number of days in a year based on the month and if it's a leap year or not
use std::{io}; // To use errors
use plotters::prelude::*; // Plotters for visualizing data

// Internal modules
pub mod simulators;
pub use crate::simulators::*; // Import the simulators module

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

/// Struct to hold timestamps in a way that makes it easier to work with than uom::si::f64::Time in Huldars opinion
#[derive(Debug)]
#[derive(Copy, Clone)]
pub struct Timestamp {
    pub year: u16,
    pub month: u8,
    pub day: u16,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl Timestamp {
    /// Creates a new timestamp
    pub fn new(year: u16, month: u8, day: u16, hour: u8, minute: u8, second: u8) -> Timestamp {
        Timestamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
        }
    }

    pub fn to_string(&self) -> String {
        let time_string: String = format!("{:04}-{:02}-{:02} {:02}:{:02}", self.year, self.month, self.day, self.hour, self.minute);
        return time_string;
    }

    /// Adds days to the timestamp
    pub fn add_days(&self, days: f64) -> Timestamp{
        let mut temp_timestamp: Timestamp = Timestamp::new(self.year, self.month, self.day, self.hour, self.minute, self.second);
        let mut whole_days_left = days as u64;
        let day_fraction = days - days.floor();
        let mut whole_hours_left = (day_fraction * 24.0) as u64;
        let hour_fraction = day_fraction * 24.0 - whole_hours_left as f64;
        let mut whole_minutes_left = (hour_fraction * 60.0) as u64;
        let minute_fraction = hour_fraction * 60.0 - whole_minutes_left as f64;
        let mut whole_seconds_left = (minute_fraction * 60.0) as u64;
        
        // Add the seconds
        while whole_seconds_left > 0 {
            temp_timestamp.second += 1;
            whole_seconds_left -= 1;
            if temp_timestamp.second > 59 {
                temp_timestamp.second = 0;
                whole_minutes_left += 1;
            }
        }

        
        // Add the minutes
        while whole_minutes_left > 0 {
            temp_timestamp.minute += 1;
            whole_minutes_left -= 1;
            if temp_timestamp.minute > 59 {
                temp_timestamp.minute = 0;
                whole_hours_left += 1;
            }
        }

        // Add the hours
        while whole_hours_left > 0 {
            temp_timestamp.hour += 1;
            whole_hours_left -= 1;
            if temp_timestamp.hour > 23 {
                temp_timestamp.hour = 0;
                whole_days_left += 1;
            }
        }

        // While days are left, increment the timestamp
        while whole_days_left > 0 {
            temp_timestamp.day += 1;
            whole_days_left -= 1;
            // Check if the day is valid
            match temp_timestamp.month {
                // 31 day month?
                1 | 3 | 5 | 7 | 8 | 10 | 12 => {
                    if temp_timestamp.day > 31 {
                        temp_timestamp.day = 1;
                        temp_timestamp.month += 1;
                    }
                }
                // 30 day month?
                4 | 6 | 9 | 11 => {
                    if temp_timestamp.day > 30 {
                        temp_timestamp.day = 1;
                        temp_timestamp.month += 1;
                    }
                }
                // February?
                2 => {
                    if year_helper::is_leap_year(temp_timestamp.year as i32) && temp_timestamp.day > 29 {
                        temp_timestamp.day = 1;
                        temp_timestamp.month += 1;
                    } else if !year_helper::is_leap_year(temp_timestamp.year as i32) && temp_timestamp.day > 28 {
                        temp_timestamp.day = 1;
                        temp_timestamp.month += 1;
                    }
                }
                _ => {}
            }
        }
        return temp_timestamp

    }
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
    pub timestamp: Timestamp,
    pub coordinates_initial: geo::Point,
    pub coordinates_current: geo::Point,
    pub coordinates_final: geo::Point,
    pub cargo_on_board: uom::si::f64::Mass,
}


/// Struct to hold boat metadata
/// All fields are optional, so that the struct can be created without knowing all the values
pub struct Boat {
    pub imo: Option<u32>,
    pub name: Option<String>,
    pub min_angle_of_attack: Option<uom::si::f64::Angle>,
    pub location: Option<geo::Point>,
    pub bearing: Option<uom::si::f64::Angle>,   //  North: 0°, East: 90°, South: 180°, West: 270°
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
    pub ship_log: Vec<ShipLogEntry>,
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
            bearing: None,
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
            ship_log: Vec::new(),
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
    (uom::si::f64::Velocity, uom::si::f64::Velocity, Option<uom::si::f64::Mass>, Option<uom::si::f64::Mass>, uom::si::f64::Time, uom::si::f64::Time, uom::si::f64::Length, uom::si::f64::Length, u64) {

    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(file_path)
        .expect("Failed to open the file");

    // Initialize variables to store the sum and count of speed and cargo values
    let mut speed_vec: Vec<uom::si::f64::Velocity> = Vec::new();
    let mut cargo_vec: Vec<Option<uom::si::f64::Mass>> = Vec::new();
    let mut dist_vec: Vec<uom::si::f64::Length> = Vec::new();
    let mut travel_time_vec: Vec<uom::si::f64::Time> = Vec::new();

    // Init empty csv column variable
    let mut timestamp: uom::si::f64::Time;
    let mut coordinates_initial: geo::Point;
    let mut coordinates_current: geo::Point;
    let mut coordinates_final: geo::Point;
    let mut cargo_on_board_option: Option<uom::si::f64::Mass>;         // weight in tons

    // Init empty working variables
    let mut dist;
    let mut trip_dist: uom::si::f64::Length = uom::si::f64::Length::new::<uom::si::length::meter>(0.0);
    let mut last_timestamp: uom::si::f64::Time = uom::si::f64::Time::new::<uom::si::time::second>(0.0);
    let mut start_time: uom::si::f64::Time = uom::si::f64::Time::new::<uom::si::time::second>(0.0);
    let mut cargo_on_trip: Option<uom::si::f64::Mass> = None;
    let mut num_trips: u64 = 0;
    let mut coordinates_last: geo::Point = geo::Point::new(0.0, 0.0);

    // Iterate through each line of the CSV file to calculate the mean and standard deviation of speed and cargo values, using each leg (each leg is 2 points) of the trip/s
    for result in csv_reader.records() {
        match result {
            Ok(log_entry) => {
                // Get all values in row as usable data
                timestamp = string_to_timestamp(log_entry.get(0).expect("No timestamp found").to_string());
                coordinates_initial = string_to_point(log_entry.get(1).expect("No initial coordinate found").to_string());
                coordinates_current = string_to_point(log_entry.get(2).expect("No initial coordinate found").to_string());
                coordinates_final = string_to_point(log_entry.get(3).expect("No initial coordinate found").to_string());
                cargo_on_board_option = string_to_tons(log_entry.get(4).unwrap().to_string());

                // If initial coordinate, the trip just started
                if coordinates_current == coordinates_initial {
                    // Increment the number of trips
                    num_trips += 1;
                    // Log start time
                    last_timestamp = timestamp;
                    start_time = timestamp;
                    // Set the last coordinates to the initial coordinates
                    coordinates_last = coordinates_initial;
                }
                // Else then it's a working point or the endpoint and we can calculate the distance
                else {
                    // Add the distance traveled from last coordinates
                    dist = haversine_distance_uom_units(coordinates_last, coordinates_current);
                    // Update trip distance
                    trip_dist += dist;
                    // Calculate the speed
                    let speed = dist / (timestamp - last_timestamp);

                    // Update last_timestamp
                    last_timestamp = timestamp;

                    // Add speed value to speed vector
                    speed_vec.push(speed);
                }

                // If there is cargo on board, set cargo_on_trip to the cargo on board. If the cargo changes then that should be the end of the trip
                if cargo_on_board_option.is_some() {
                    cargo_on_trip = cargo_on_board_option;                    
                }


                // If current coord is not inital or final this is a working point, set current coordinates as last coordinates
                if coordinates_current != coordinates_initial && coordinates_current != coordinates_final {
                    // Update last coordinates
                    coordinates_last = coordinates_current;
                }

                // If final coordinate, the trip just ended
                if coordinates_current == coordinates_final {
                    // Add travel time to travel time vector
                    travel_time_vec.push(timestamp - start_time);
                    // Add trip distance to distance vector
                    dist_vec.push(trip_dist);
                    // If there is cargo, Add cargo to cargo vector
                    if cargo_on_trip.is_some() {
                        cargo_vec.push(cargo_on_trip);
                    }
                     
                    // Reset trip distance distance
                    trip_dist = uom::si::f64::Length::new::<uom::si::length::meter>(0.0);
                    // Reset cargo
                    cargo_on_trip = None;
                }
            }
            // Handle the error if the log_entry cannot be read
            Err(err) => {
                eprintln!("Error reading log_entry: {}", err);
            }
        }
    }

    // Calculate the mean and standard deviation of the vectors
    let speed_mean: uom::si::f64::Velocity;
    let speed_std: uom::si::f64::Velocity;
    let cargo_mean: Option<uom::si::f64::Mass>;
    let cargo_std: Option<uom::si::f64::Mass>;
    let travel_time_mean: uom::si::f64::Time;
    let travel_time_std: uom::si::f64::Time;
    let dist_mean: uom::si::f64::Length;
    let dist_std: uom::si::f64::Length;

    (speed_mean, speed_std) = get_speed_mean_and_std(&speed_vec);
    (cargo_mean, cargo_std) = get_weight_mean_and_std(&cargo_vec);
    (travel_time_mean, travel_time_std) = get_time_mean_and_std(&travel_time_vec);
    (dist_mean, dist_std) = get_distance_mean_and_std(&dist_vec);

    // Return the values
    return (speed_mean, speed_std, cargo_mean, cargo_std, travel_time_mean, travel_time_std, dist_mean, dist_std, num_trips)
}


/// Function for visualizing shipping_logs
/// The starting point is green and the final point is red, the coordinates of those points are shown in the figure.
/// Note: Currently plots to a flat X-Y plane, so the coordinates are not projected onto a globe.
/// ship_logs_file_path: Path to the CSV file where the ship logs are stored
/// figure_file_path: Path to the file where the figure will be saved
/// Example:
pub fn visualize_ship_logs(ship_logs_file_path: &str, figure_file_path: &str) -> Result<(), io::Error>{
    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(ship_logs_file_path)
        .expect("Failed to open the file");

    // Init figure
    let figure = BitMapBackend::new(figure_file_path, (640, 480)).into_drawing_area();

    figure.fill(&RGBColor(255, 255, 255)).expect("Failed to fill background");

    let figure = figure.apply_coord_spec(Cartesian2d::<plotters::coord::types::RangedCoordf32, plotters::coord::types::RangedCoordf32>::new(
        0f32..1f32,
        0f32..1f32,
        (0..640, 0..480),
    ));

    let dot_and_label = |x: f32, y: f32, lat: f64, lon: f64, color: plotters::style::RGBColor | {
        return EmptyElement::at((x, y))
            + Circle::new((0, 0), 3, ShapeStyle::from(&color).filled())
            + Text::new(
                format!("({:.2},{:.2})", lat, lon),
                (10, 0),
                ("sans-serif", 15.0).into_font(),
            );
    };

    let dot = |x: f32, y: f32| {
    return EmptyElement::at((x, y))
        + Circle::new((0, 0), 1, ShapeStyle::from(&BLACK).filled());
    };

    // Iterate through each line of the CSV file to draw the values
    for (i, result) in csv_reader.records().enumerate() {
        match result {
            Ok(log_entry) => {
                // Get all values in row as usable data
                // let timestamp = log_entry.get(0).expect("No timestamp found").to_string();
                let coordinates_current = string_to_point(log_entry.get(2).expect("No current coordinate found").to_string());
                // let cargo_on_board = log_entry.get(4).unwrap().to_string();

                // if first value, draw initial and final coordinates
                if i == 0 {
                    // Get coordinates
                    let coordinates_initial = string_to_point(log_entry.get(1).expect("No initial coordinate found").to_string());
                    let coordinates_final = string_to_point(log_entry.get(3).expect("No final coordinate found").to_string());

                    // Draw initial coordinate
                    let (x,y) = geo_point_to_xy(coordinates_initial);
                    figure.draw(&dot_and_label(x, y, coordinates_initial.y(), coordinates_initial.x(), GREEN)).expect("Failed to draw dot and label");

                    // Draw final coordinate
                    let (x,y) = geo_point_to_xy(coordinates_final);
                    figure.draw(&dot_and_label(x, y, coordinates_final.y(), coordinates_final.x(), RED)).expect("Failed to draw dot and label");
                }

                // Draw point on figure. Each coordinate must be represented by a value between 0 and 1, so we need to convert the coordinates to a value between 0 and 1
                // x = 0 is the left side, x = 1 is the right side, y = 0 is the top, y = 1 is the bottom
                // figure.draw(&dot_and_label(0.5, 0.6)).expect("Failed to draw dot and label");
                let (x, y) = geo_point_to_xy(coordinates_current);
                figure.draw(&dot(x, y)).expect("Failed to draw dot");
            }
            Err(err) => {
                eprintln!("Error reading log_entry: {}", err);
            }
        } // End match
    } // End for loop

    // TODO: Figure out what this does
    figure.present().expect("Failed to present the figure");


    // Save figure to file

    // Return Ok if all went well
    return Ok(());
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
    let day_of_month:    uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::day>(working_str[8..10].parse::<f64>().expect("Invalid day"));
    let hour:   uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::hour>(working_str[11..13].parse::<f64>().expect("Invalid hour"));
    let minute: uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::minute>(working_str[14..16].parse::<f64>().expect("Invalid minute"));

    let days: uom::si::f64::Time = days_from_month(month_u8, year_i32) + day_of_month;

    // Attempt to parse the string into a uom::si::f64::Time object
    let time_out: uom::si::f64::Time = year_uom + days + hour + minute;
    
    // Return
    return time_out;
}

/// Converts a time_stamp to a string in the format YYYY-MM-DD hh:mm
pub fn timestamp_to_string(time_stamp: uom::si::f64::Time) -> String {
    // Get the year and day from the time_stamp
    let year: i32 = time_stamp.get::<uom::si::time::year>() as i32;
    let time_left: uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::year>(time_stamp.get::<uom::si::time::year>() - (year as f64));

    let day_of_year: u16 = 1 + time_left.get::<uom::si::time::day>() as u16;

    // Find the month from the day and year (if it's a leap year)
    let (month, day_of_month): (u8, u16) = month_from_day(day_of_year, year);

    // Find hour
    let time_left: uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::year>(time_left.get::<uom::si::time::day>() - (day_of_year as f64));
    let hour: u16 = time_left.get::<uom::si::time::hour>() as u16;

    // Find minute
    let time_left: uom::si::f64::Time = uom::si::time::Time::new::<uom::si::time::year>(time_left.get::<uom::si::time::hour>() - (hour as f64));
    let minute: u16 = time_left.get::<uom::si::time::minute>() as u16;

    // Format the string
    let time_string: String = format!("{:04}-{:02}-{:02} {:02}:{:02}", year, month, day_of_month, hour, minute);

    // return string
    return time_string;
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

/// Finds out which month of the year it is given the day number and year (in case it is a leap year)
pub fn month_from_day(day_of_year: u16, year: i32) -> (u8, u16) {
    let mut days_left: u16 = day_of_year;

    println!("Day of year: {}", days_left);

    // Check if the day is valid
    if days_left > 366 {
        panic!("Invalid day");
    }

    // Init month
    let month: u8;

    // Leap year check
    let is_leap_year: bool = year_helper::is_leap_year(year);

    // match through the number of days
    match days_left {
        1..=31 => {
            month = 1; // January
        }
        32..=59 => {
            month = 2; // February
            days_left -= 31; // January has 31 days
        }
        60..=90 => {
            // if leap year, day 60 is 29th of February
            if is_leap_year && days_left == 60 {
                month = 2; // February
            } else {
                month = 3; // March
            }
            days_left -= 59;    // remove days in previous months of a normal year
        }
        91..=120 => {
            month = 4; // April
            days_left -= 90;
        }
        121..=151 => {
            month = 5; // May
            days_left -= 120;
        }
        152..=181 => {
            month = 6; // June
            days_left -= 151;
        }
        182..=212 => {
            month = 7; // July
            days_left -= 181;
        }
        213..=243 => {
            month = 8; // August
            days_left -= 212;
        }
        244..=273 => {
            month = 9; // September
            days_left -= 243;
        }
        274..=304 => {
            month = 10; // October
            days_left -= 273;
        }
        305..=334 => {
            month = 11; // November
            days_left -= 304;
        }
        335..=366 => {
            month = 12; // December
            days_left -= 334;
        }
        _ => {
            panic!("Invalid day");
        }
    }

    // If month is big enough and it's a leap year, subtract 1 day from days left
    if is_leap_year && month > 2 {
        days_left -= 1; // Leap year, add one day to February
    }

    println!("Month: {}, Day of month : {}", month, days_left);

    // Return the month and how many days are left
    return (month, days_left);
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

    // Make return value
    let return_cargo: Option<uom::si::f64::Mass> = Some(uom::si::f64::Mass::new::<uom::si::mass::ton>(cargo));
    return return_cargo;
}


/// Returns the average and standard deviation of a vector of uom::si::f64::Velocity objects
/// speed_vec: The vector of uom::si::f64::Velocity objects
/// Example:
/// let (my_mean, my_std) = get_speed_mean_and_std(&my_vec);
pub fn get_speed_mean_and_std(speed_vec: &Vec<uom::si::f64::Velocity>) ->
    (uom::si::f64::Velocity, uom::si::f64::Velocity) {
    // Validate that the speed_vec has at least 1 value
    if speed_vec.is_empty() {
        panic!("Speed vector is empty, cannot calculate mean and standard deviation");
    }
    
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



/// Returns the average and standard deviation of a vector of uom::si::f64::Length objects
/// dist_vec: The vector of uom::si::f64::Length objects
/// Example:
/// let (my_mean, my_std) = get_dist_mean_and_std(&my_vec);
pub fn get_distance_mean_and_std(dist_vec: &Vec<uom::si::f64::Length>) -> (uom::si::f64::Length, uom::si::f64::Length) {
    // Calculate the mean of the vector
    let mut total = uom::si::f64::Length::new::<uom::si::length::meter>(0.0);

    // loop through vector, add all values to the total
    for dist in dist_vec {
        total = total + *dist;
    }
    // Find mean
    let mean: uom::si::f64::Length = total / dist_vec.len() as f64;
    let mean_f64: f64 = mean.get::<uom::si::length::meter>();

    // Calculate the standard deviation of the vector
    let mut variance: f64 = 0.0;

    // loop through vector, add all values to variance, then divide by number of values -1 to create variance
    for dist in dist_vec {
        variance = variance + (dist.get::<uom::si::length::meter>() - mean_f64).powi(2);
    }
    variance = variance / ((dist_vec.len() - 1) as f64);

    // Find standard deviation from variance
    let std: uom::si::f64::Length = uom::si::f64::Length::new::<uom::si::length::meter>(variance.sqrt());

    // Return the mean and standard deviation
    return (mean, std);
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


/// Function that writes the ship logs to a CSV file with the following columns:
/// timestamp;coordinates_initial;coordinates_current;coordinates_final;cargo_on_board
/// Note that the coordinates are in the format of ISO 6709 using decimal places with a comma between latitude and longitude. "latitude,longitude" (e.g., "52.5200,13.4050")
/// The cargo is in metric tons (1 metric ton = 1000 kg)
/// csv_file_path: Path to the CSV file
/// boat: The boat object containing the ship logs
/// Note: The csv file delimieter is a semicolon
pub fn ship_logs_to_csv(csv_file_path: &str, boat: &Boat) -> Result<(), io::Error> {
    // Create a CSV writer with a semicolon delimiter
    // let mut wtr = csv::WriterBuilder::new().delimiter(b';').from_path(csv_file_path)?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(csv_file_path)?;

    // Write the header
    wtr.write_record(&["timestamp", "coordinates_initial", "coordinates_current", "coordinates_final", "cargo_on_board"])?;

    // Write the ship log entries
    for entry in boat.ship_log.iter() {
        wtr.write_record(&[
            entry.timestamp.to_string(), // timestamp_to_string(entry.timestamp),
            format!("{},{}", entry.coordinates_initial.y(), entry.coordinates_initial.x()),
            format!("{},{}", entry.coordinates_current.y(), entry.coordinates_current.x()),
            format!("{},{}", entry.coordinates_final.y(), entry.coordinates_final.x()),
            entry.cargo_on_board.get::<uom::si::mass::ton>().to_string(),
        ])?;
    }

    // Flush and close the writer
    wtr.flush()?;
    Ok(())
}



/// Function that translates coordinates to x,y values between 0 and 1 for plotting
pub fn geo_point_to_xy(point_in: geo::Point) -> (f32, f32) {
    // Normalize latitude to 0..1 where 0.5 is equator
    let y = (-point_in.y() + 90.0) / 180.0;
    // Normalize longitude to 0..1 where 0.5 is prime meridian
    let x = (point_in.x() + 180.0) / 360.0;

    // Return the coordinates as a tuple
    return (x as f32, y as f32);
}










// Set up tests here
