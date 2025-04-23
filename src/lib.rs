/// Marine vessel simulator simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-04-14
/// Note that a dimensional anlysis is not performed in this code using uom (https://crates.io/crates/uom)

/// External crates
use csv;    // CSV reader to read csv files
use uom;    // Units of measurement. Makes sure that the correct units are used for every calculation
use geo;    // Geographical calculations. Used to calculate the distance between two coordinates


/// This function evaluates the cargo shipping logs from a CSV file and calculates the mean and standard deviation of the speed and cargo delivery values. The CSV file is expected to have the following columns:<br>
/// timestamp;coordinates_initial;coordinates_current;coordinates_final;cargo_on_board (weight in tons)<br><br>
/// The delimiter is a semicolon.
/// file_path: Path to the CSV file
/// distance: The total sailing distance. Note if distance = 0 the function evaluates the sailing distance by drawing a straight line for each leg of the trip 
/// Note: Coordinates are expected to be in the format of ISO 6709 using decimal places with a comma between latitude and longitude. "latitude,longitude" (e.g., "52.5200,13.4050") 
/// Example:
/// let filename: &str = "../data/mydata.csv";
/// let distance: u64 = 8000; // Distance in km
/// let (speed_mean, speed_std, cargo_mean, cargo_std) = evaluate_cargo_shipping_logs(filename, distance);
pub fn evaluate_cargo_shipping_logs(file_path: &str, distance: u64) ->
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
    // let mut timestamp: uom::si::f64::Time;
    let coordinates_initial_str: &str;
    let coordinates_initial_coord: geo::Coord;
    // let mut coordinates_current: geo::Coord;
    // let mut coordinates_final: geo::Coord;
    // let mut cargo_on_board: uom::si::f64::Mass;         // weight in tons

    // Iterate through each line of the CSV file to calculate the mean and standard deviation of speed and cargo values
    for result in csv_reader.records() {
        match result {
            Ok(record) => {
                // Get start and end coordinates
                // timestamp = record.get(0).unwrap().parse::<uom::si::f64::Time>().unwrap();
                coordinates_initial_str = string_to_coordinate(record.get(1).expect("No initial coordinate found").to_string()); // .unwrap().parse::<geo::Coord>().unwrap();
                // coordinates_current = record.get(2).unwrap().parse::<geo::Coord>().unwrap();
                // coordinates_final = record.get(3).unwrap().parse::<geo::Coord>().unwrap();
                // cargo_on_board = record.get(4).unwrap().parse::<uom::si::f64::Mass>().unwrap();
                // Print the values for debugging
                // println!("Timestamp: {}", timestamp);
                println!("Coordinates Initial str: {}", coordinates_initial_str);

                // Convert the string to a geo::Coord object
                coordinates_initial_coord = str_to_coordinate(coordinates_initial_str.to_string());
                // Print them
                print!("Coordinates Initial coord: {:?}", coordinates_initial_coord);

                println!("Coordinates Current: {:?}", geo::coord! {
                    x: 40.02f64,
                    y: 116.34,
                });
                // println!("Coordinates Current: {:?}", coordinates_current);
                // println!("Coordinates Final: {:?}", coordinates_final);
                // println!("Cargo on Board: {}", cargo_on_board);
                // Calculate the distance between the coordinates

                println!("{:?}", record);
                break; 
            }
            // Handle the error if the record cannot be read
            Err(err) => {
                eprintln!("Error reading record: {}", err);
            }
        }
    }

    // Check if distance is 0
    if distance == 0 {
        // Calculate the distance based on the coordinates
        // This is a placeholder for the actual distance calculation logic
        // You would need to implement the logic to calculate the distance based on the coordinates
    }

    // Return the values
    return (speed_avg, speed_std, cargo_avg, cargo_std)
}

// Helper functions
/// str_to_coordinate converts a string into a geo::Coord object
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
