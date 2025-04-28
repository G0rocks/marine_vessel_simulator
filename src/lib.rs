/// Marine vessel simulator simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-04-14
/// Note that a dimensional anlysis is not performed in this code using uom (https://crates.io/crates/uom)

/// External crates
use csv;    // CSV reader to read csv files
use uom;    // Units of measurement. Makes sure that the correct units are used for every calculation
use geo;    // Geographical calculations. Used to calculate the distance between two coordinates
use year_helper; // Year helper to calculate the number of days in a year based on the month and if it's a leap year or not


/// This function evaluates the cargo shipping logs from a CSV file and calculates the mean and standard deviation of the speed and cargo delivery values. The CSV file is expected to have the following columns:<br>
/// timestamp;coordinates_initial;coordinates_current;coordinates_final;cargo_on_board (weight in tons)<br><br>
/// The delimiter is a semicolon.
/// file_path: Path to the CSV file
/// distance: The total sailing distance. Note if distance = 0 the function evaluates the sailing distance by drawing a straight line for each leg of the trip 
/// Notes:
/// Timestamps are expected to be in the ISO format of YYYY-MM-DD hh:mm.
/// Coordinates are expected to be in the format of ISO 6709 using decimal places with a comma between latitude and longitude. "latitude,longitude" (e.g., "52.5200,13.4050") 
/// Example:
/// let filename: &str = "../data/mydata.csv";
/// let distance: u64 = 8000; // Distance in km
/// let (speed_mean, speed_std, cargo_mean, cargo_std) = evaluate_cargo_shipping_logs(filename, distance);
pub fn evaluate_cargo_shipping_logs(file_path: &str) ->
    (uom::si::f64::Velocity, f64, f64, f64) {

    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(file_path)
        .expect("Failed to open the file");

    // Initialize variables to store the sum and count of speed and cargo values   
    let speed_avg = uom::si::f64::Velocity::new::<uom::si::velocity::kilometer_per_hour>(2.0);
    let speed_std: f64 = 0.0;
    let cargo_avg: f64 = 0.0;
    let cargo_std: f64 = 0.0;

    // Init empty csv column variable
    let timestamp: uom::si::f64::Time;
    let coordinates_initial_coord: geo::Coord;
    // let mut coordinates_current: geo::Coord;
    // let mut coordinates_final: geo::Coord;
    // let mut cargo_on_board: uom::si::f64::Mass;         // weight in tons

    // Iterate through each line of the CSV file to calculate the mean and standard deviation of speed and cargo values, using each leg of the trip/s
    for result in csv_reader.records() {
        match result {
            Ok(leg) => {
                // Get all values in row
                timestamp = string_to_timestamp(leg.get(0).expect("No timestampe found").to_string());
                // Convert the string to a geo::Coord object
                coordinates_initial_coord = string_to_coordinate(leg.get(1).expect("No initial coordinate found").to_string());
                // coordinates_current = leg.get(2).unwrap().parse::<geo::Coord>().unwrap();
                // coordinates_final = leg.get(3).unwrap().parse::<geo::Coord>().unwrap();
                // cargo_on_board = leg.get(4).unwrap().parse::<uom::si::f64::Mass>().unwrap();
                // Print the values for debugging
                // println!("Timestamp: {}", timestamp);

                // Print them
                println!("Timestamp woooohooooo: {:?}", timestamp);
                println!("Coordinates Initial coord: {:?}", coordinates_initial_coord);

                println!("Coordinates Current: {:?}", geo::coord! {
                    x: 40.02f64,
                    y: 116.34,
                });
                // println!("Coordinates Current: {:?}", coordinates_current);
                // println!("Coordinates Final: {:?}", coordinates_final);
                // println!("Cargo on Board: {}", cargo_on_board);
                // Calculate the distance between the coordinates

                println!("{:?}", leg);
                break; 
            }
            // Handle the error if the leg cannot be read
            Err(err) => {
                eprintln!("Error reading leg: {}", err);
            }
        }
    }

    // Return the values
    return (speed_avg, speed_std, cargo_avg, cargo_std)
}

// Helper functions

/// Converts a string into an uom::si::f64::Time object
/// coord_str: The string to convert in the format YYYY-MM-DD hh:mm
/// Example:
/// let my_timestamp: uom::si::f64::Time = str_to_coordinate("52.5200,13.4050");
pub fn string_to_timestamp(time_string: String) -> uom::si::f64::Time {
    println!("String to timestamp: {}", time_string);

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
    if ((month < 1) || (month > 12)) {
        panic!("Invalid month");
    }

    // Init days
    let mut days: u64 = 0;

    // for each month, add number of days in month to days
    for i in (1..month) {
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
    if (year_helper::is_leap_year(year) && (month > 2)) {
        days += 1;
    }

    // Return the number of days
    return uom::si::f64::Time::new::<uom::si::time::day>(days as f64);
}



/// Converts a string into a geo::Coord object
/// coord_str: The string to convert
/// Example:
/// let my_coord: geo::Coord = str_to_coordinate("52.5200,13.4050");
pub fn string_to_coordinate(coord_string: String) -> geo::Coord {
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

    // Make return coordinate
    let return_coord = geo::coord! {
                                x: latitude,
                                y: longitude,};
    return return_coord;
}


// Set up tests here
