/// Marine vessel simulator simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-04-14
/// Note that a dimensional anlysis is not performed in this code using uom (https://crates.io/crates/uom)
/// ## To do
/// Make another crate, sailplanner, that can make route plans for marine vessels.

/// External crates
use csv; // CSV reader to read csv files
use geo::{self, Haversine, Rhumb, Bearing, Distance, Destination};    // Geographical calculations. Used to calculate the distance between two coordinates and bearings
use year_helper; // Year helper to calculate the number of days in a year based on the month and if it's a leap year or not
use std::{io, fmt, f64::consts, fs::File, io::Write}; // To use errors, formatting, constants, write to file
// use plotters; // Plotters for visualizing data on a map. Uses only rust, no javascript. Will probably be removed in favor of plotly
use plotly; // Plotly for visualizing data on a map. Testing in comparison agains plotters
use copernicusmarine_rs;    // To get weather data
use time;   // To do time calculations
use time::UtcDateTime;  // To use UtcDateTime
use indicatif;   // For progress bar
use atty;       // To check if terminal is interactive or not


// Internal modules
pub mod simulators;
pub use crate::simulators::*; // Import the simulators module
pub mod vessels;
pub use crate::vessels::*; // Import the simulators module

// Constants
//----------------------------------------------------


// Structs and enums
//----------------------------------------------------
/// A physics vector struct that holds vector data... for physics :)
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PhysVec {
    /// Magnitude, make sure that the units are correct
    pub magnitude: f64,
    /// Direction in degrees, 0° is north, 90° is east, 180° is south, 270° is west
    pub angle: f64,
}

impl PhysVec {
    /// Creates a new wind object
    pub fn new(magnitude: f64, angle: f64) -> PhysVec {
        PhysVec {
            magnitude,
            angle,
        }
    }
}

/// std::Display for PhysVec
impl fmt::Display for PhysVec {
    /// format for PhysVec
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "magnitude: {}, angle: {}", self.magnitude, self.angle)
    }
}

/// std::ops::Add (addition) for PhysVec
impl std::ops::Add for PhysVec {
    type Output = Self; // The output type is also Point

    fn add(self, other: Self) -> Self {
        // Get x,y coord of first PhysVec
        let x1 = self.magnitude * (self.angle*std::f64::consts::PI/180.0).cos();
        let y1 = self.magnitude * (self.angle*std::f64::consts::PI/180.0).sin();
        // Get x,y coord of second PhysVec
        let x2 = other.magnitude * (other.angle*std::f64::consts::PI/180.0).cos();
        let y2 = other.magnitude * (other.angle*std::f64::consts::PI/180.0).sin();
        // Get x,y coord of output PhysVec
        let x3 = x1 + x2;
        let y3 = y1 + y2;
        // Get magnitude
        let magnitude = (x3*x3 + y3*y3).sqrt();
        // Get angle
        let angle = y3.atan2(x3) * 180.0 / std::f64::consts::PI;
        // Return output with magnitude and angle of output PhysVec
        return PhysVec::new(magnitude, angle);
    }
}

/// std::ops::Sub (subtraction) for PhysVec
impl std::ops::Sub for PhysVec {
    type Output = Self; // The output type is also Point

    fn sub(self, other: Self) -> Self {
        // Get x,y coord of first PhysVec
        let x1 = self.magnitude * (self.angle*std::f64::consts::PI/180.0).cos();
        let y1 = self.magnitude * (self.angle*std::f64::consts::PI/180.0).sin();
        // Get x,y coord of second PhysVec
        let x2 = other.magnitude * (other.angle*std::f64::consts::PI/180.0).cos();
        let y2 = other.magnitude * (other.angle*std::f64::consts::PI/180.0).sin();
        // Get x,y coord of output PhysVec
        let x3 = x1 - x2;
        let y3 = y1 - y2;
        // Get magnitude
        let magnitude = (x3*x3 + y3*y3).sqrt();
        // Get angle
        let angle = y3.atan2(x3) * 180.0 / std::f64::consts::PI;
        // Return output with magnitude and angle of output PhysVec
        return PhysVec::new(magnitude, angle);
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
/// # Example:
/// ```
/// let filename: &str = "../data/mydata.csv";
/// // Distance in meters
/// let distance: f64 = 50;
/// let (speed_mean, speed_std, cargo_mean, cargo_std) = evaluate_cargo_shipping_logs(filename, distance);
/// ```
pub fn evaluate_cargo_shipping_logs(file_path: &str, destination_minimum_proximity: f64) ->
    (Option<f64>, Option<f64>,
        Option<f64>, Option<f64>,
        Option<time::Duration>, Option<time::Duration>,
        Option<f64>, Option<f64>, u64) {

    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .flexible(true)
        .from_path(file_path)
        .expect(format!("Failed to open file: {}", file_path).as_str());

    // Initialize variables to store the sum and count of speed and cargo values
    let mut speed_vec: Vec<f64> = Vec::new();
    let mut cargo_vec: Vec<f64> = Vec::new();
    let mut dist_vec: Vec<f64> = Vec::new();
    let mut travel_time_vec: Vec<time::Duration> = Vec::new();

    // Init empty csv column variable
    let mut timestamp: time::UtcDateTime;
    let mut coordinates_initial: geo::Point;
    let mut coordinates_current: geo::Point;
    let mut coordinates_final: geo::Point;
    let mut cargo_on_board_option: Option<f64>;         // weight in tons

    // Init empty working variables
    // Distances are in meters
    let mut dist: f64;
    let mut trip_dist: f64 = 0.0;
    let mut last_timestamp = time::UtcDateTime::now();
    let mut start_time = time::UtcDateTime::now();
    let mut cargo_on_trip: Option<f64> = None;
    let mut num_trips: u64 = 0;
    let mut coordinates_last: geo::Point = geo::Point::new(0.0, 0.0);

    // Iterate through each line of the CSV file to calculate the mean and standard deviation of speed and cargo values, using each leg (each leg is 2 points) of the trip/s
    for result in csv_reader.records() {
        match result {
            Ok(log_entry) => {
                // Get all values in row as usable data
                timestamp = string_to_utc_date_time(log_entry.get(0).expect("No timestamp found").to_string());
                coordinates_initial = string_to_point(log_entry.get(1).expect("No initial coordinate found").to_string());
                coordinates_current = string_to_point(log_entry.get(2).expect("No initial coordinate found").to_string());
                coordinates_final = string_to_point(log_entry.get(3).expect("No initial coordinate found").to_string());
                cargo_on_board_option = match log_entry.get(4).unwrap().to_string().parse() {
                    Ok(cargo) => Some(cargo),
                    Err(_) => None,
                };

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
                    dist = Haversine.distance(coordinates_last, coordinates_current);
                    // Update trip distance
                    trip_dist += dist;
                    // Calculate the speed in m/s
                    let speed = dist / (timestamp - last_timestamp).as_seconds_f64();

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
                if Haversine.distance(coordinates_current, coordinates_final) <= destination_minimum_proximity {
                    // Add travel time to travel time vector
                    travel_time_vec.push(timestamp - start_time);
                    // Add trip distance to distance vector
                    dist_vec.push(trip_dist);
                    // If there is cargo, Add cargo to cargo vector
                    if cargo_on_trip.is_some() {
                        cargo_vec.push(cargo_on_trip.unwrap());
                    }
                     
                    // Reset trip distance distance
                    trip_dist = 0.0;
                    // Reset cargo
                    cargo_on_trip = None;
                }
            }
            // Handle the error if the log_entry cannot be read
            Err(ref err) => {
                eprintln!("Error reading log_entry: {:?}\nError: {}", result, err);
            }
        }
    }

    // Calculate the mean and standard deviation of the vectors
    let speed_mean: Option<f64>;
    let speed_std: Option<f64>;
    let cargo_mean: Option<f64>;
    let cargo_std: Option<f64>;
    let travel_time_mean: Option<time::Duration>;
    let travel_time_std: Option<time::Duration>;
    let dist_mean: Option<f64>;
    let dist_std: Option<f64>;

    match get_vec_f64_mean_and_std(&speed_vec) {
        Ok((mean, std)) => {
            speed_mean = Some(mean);
            speed_std = Some(std);
        },
        Err(_) => {
            // eprintln!("Error calculating speed mean and std. Set to zero. Error message: {}", e);
            speed_mean = None;
            speed_std = None;
        }
    }
    match get_vec_f64_mean_and_std(&cargo_vec) {
        Ok((mean, std)) => {
            cargo_mean = Some(mean);
            cargo_std = Some(std);
        },
        Err(_) => {
            // eprintln!("Error calculating cargo mean and std. Set to None. Error message: {}", e);
            cargo_mean = None;
            cargo_std = None;
        }
    }

    // Parse travel_time_vec to travel_time_vec_secs
    let travel_time_vec_secs = travel_time_vec.iter().map(|d| d.as_seconds_f64()).collect::<Vec<f64>>();
    match get_vec_f64_mean_and_std(&travel_time_vec_secs) {
        Ok((mean, std)) => {
            let mean_secs = mean as i64;
            travel_time_mean = Some(time::Duration::new(mean_secs, ((mean - mean_secs as f64)*1000000000.0) as i32));
            let std_secs = std as i64;
            travel_time_std = Some(time::Duration::new(std_secs, ((std - std_secs as f64)*1000000000.0) as i32));
        },
        Err(e) => {
            eprintln!("Error calculating travel time mean and std. Set to zero. Error message: {}", e);
            travel_time_mean = None;
            travel_time_std = None;
        }
    }
    match get_vec_f64_mean_and_std(&dist_vec) {
        Ok((mean, std)) => {
            dist_mean = Some(mean);
            dist_std = Some(std);
        },
        Err(e) => {
            eprintln!("Error calculating distance mean and std. Set to zero. Error message: {}", e);
            dist_mean = None;
            dist_std = None;
        }
    }
    // Return the values
    return (speed_mean, speed_std, cargo_mean, cargo_std, travel_time_mean, travel_time_std, dist_mean, dist_std, num_trips)
}

/// Saves the given parameters to a csv file at csv_file_path
/// Will overwrite any file with the same file name at csv_file_path.
/// Does not append rows to existing csv files.
/// csv_file_path must end with ".csv"
/// names is the first column of the csv file and will help indicate what the statistics are for.
/// All vectors must have the same length
/// Returns mean distance in kilometers and distance standard deviation in meters
pub fn save_shipping_logs_evaluation_to_csv(csv_file_path: &str, name_vec: Vec<&str>, speed_mean_vec: Vec<Option<f64>>, speed_std_vec: Vec<Option<f64>>, cargo_mean_vec: Vec<Option<f64>>, cargo_std_vec: Vec<Option<f64>>, travel_time_mean_vec: Vec<Option<time::Duration>>, travel_time_std_vec: Vec<Option<time::Duration>>, dist_mean_vec: Vec<Option<f64>>, dist_std_vec: Vec<Option<f64>>, num_trips_vec: Vec<u64>) -> Result<String, io::Error> {
    // Check if csv_file_path ends with ".csv"
    let num_chars = csv_file_path.chars().count();
    if &csv_file_path[(num_chars-4)..] != ".csv" {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "The filepath must end with \".csv\""));
    }

    // Check if vectors are the same size
    let vec_size = name_vec.len();
    if speed_mean_vec.len() != vec_size || speed_std_vec.len() != vec_size || cargo_mean_vec.len() != vec_size || cargo_std_vec.len() != vec_size || travel_time_mean_vec.len() != vec_size || travel_time_std_vec.len() != vec_size || dist_mean_vec.len() != vec_size || dist_std_vec.len() != vec_size || num_trips_vec.len() != vec_size {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "All input vectors must have the same length"));
    }

    // Create a CSV writer with a semicolon delimiter
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(csv_file_path)?;

    // Write the header
    wtr.write_record(&["name","speed_mean[m/s]","speed_std[m/s]","cargo_mean[tons]","cargo_std[tons]","travel_time_mean[days]","travel_time_std[days]","dist_mean[m]","dist_std[m]","num_trips:"])?;

    // Write the ship log entries
    for i in 0..vec_size {
        // Get name
        let name = name_vec[i];
        // Get speed_mean
        let speed_mean = &speed_mean_vec[i].unwrap().to_string();
        // Get speed_std
        let speed_std = &speed_std_vec[i].unwrap().to_string();
        // Get cargo_mean, if None, set to empty string
        let cargo_mean = &match cargo_mean_vec[i] {
            Some(c) => c.to_string(),
            None => String::from(""),
        };
        // Get cargo_std, if None, set to empty string
        let cargo_std = &match cargo_std_vec[i] {
            Some(c) => c.to_string(),
            None => String::from(""),
        };
        // Get travel_time_mean
        let travel_time_mean = &travel_time_mean_vec[i].unwrap().to_string();
        // Get travel_time_std
        let travel_time_std = &travel_time_std_vec[i].unwrap().to_string();
        // Get dist_mean in meters
        let dist_mean = &(dist_mean_vec[i].unwrap()).to_string();
        // Get dist_std in meters
        let dist_std = &dist_std_vec[i].unwrap().to_string();
        // Get num_trips
        let num_trips = &num_trips_vec[i].to_string();

        // Write the record
        wtr.write_record(&[
            name,
            speed_mean,
            speed_std,
            cargo_mean,
            cargo_std,
            travel_time_mean,
            travel_time_std,
            dist_mean,
            dist_std,
            num_trips,
        ])?;
    }

    // Flush and close the writer
    wtr.flush()?;
    return Ok(("Saved shipping log statistics to csv file").to_string());
}

/// Visualize ship logs with plotly on map
/// figure_file_path: Option<&str> - Path to the file where the figure will be saved. If None, the figure will not be saved to a file.
pub fn visualize_ship_logs_and_route(ship_logs_file_path: &str, route_plan_file_path: &str, figure_file_path: Option<&str>) -> Result<(), io::Error> {
    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(ship_logs_file_path)
        .expect("Failed to open the file");

    // Init vectors for coordinates
    let mut y_vec: Vec<f64> = Vec::new();
    let mut x_vec: Vec<f64> = Vec::new();

    // Iterate through each line of the CSV file to draw the values
    for result in csv_reader.records() {
        match result {
            Ok(log_entry) => {
                // Get current coordinates
                let coordinates_current = string_to_point(log_entry.get(2).expect("No current coordinate found").to_string());

                // Add coordinates to vectors
                x_vec.push(coordinates_current.x());
                y_vec.push(coordinates_current.y());
            }
            Err(err) => {
                eprintln!("Error reading log_entry: {}", err);
            }
        } // End match
    } // End for loop

    // Setup trace of ship logs
    let trace = plotly::ScatterGeo::new(y_vec, x_vec)
                    .name("Ship logs")
                    .mode(plotly::common::Mode::LinesMarkersText)
                    .show_legend(true);  // ScatterGeo::new(latitudes, longitudes).name("Ship Logs").marker_color("blue"));

    // Set layout as instructed by andrei-ng https://github.com/plotly/plotly.rs/pull/301
    let layout = plotly::Layout::new()
        .drag_mode(plotly::layout::DragMode::Zoom)
        .margin(plotly::layout::Margin::new().top(20).left(10).bottom(30).right(10))
        .auto_size(true)
        .geo(
            plotly::layout::LayoutGeo::new()
                .showocean(true)
                .showlakes(true)
                .showcountries(true)
                .showland(true)
                .oceancolor(plotly::color::Rgb::new(0, 255, 255))
                .lakecolor(plotly::color::Rgb::new(0, 255, 255))
                .landcolor(plotly::color::Rgb::new(230, 145, 56))
                .lataxis(
                    plotly::layout::Axis::new()
                        .show_grid(true)
                        .grid_color(plotly::color::Rgb::new(102, 102, 102)),
                )
                .lonaxis(
                    plotly::layout::Axis::new()
                        .show_grid(true)
                        .grid_color(plotly::color::Rgb::new(102, 102, 102)),
                )
                .projection(
                    plotly::layout::Projection::new().projection_type(plotly::layout::ProjectionType::Orthographic),
                ),
        );


    // Create a plotly figure with the coordinates
    let mut figure = plotly::Plot::new();
    // Add trace
    figure.add_trace(trace);
    // Set layout to orthographic
    figure.set_layout(layout);
    // Get configuration and make responsive for automatically sizing according to window size
    let fig_config = figure.configuration().clone().responsive(true).fill_frame(true);
    // Set config
    figure.set_configuration(fig_config);


    // Init vectors for coordinates
    let mut x_vec: Vec<f64> = Vec::new();
    let mut y_vec: Vec<f64> = Vec::new();

    // Add each waypoint
    // TODO: with label to plot
    let route_plan = load_route_plan(route_plan_file_path);
    for leg in &route_plan {
        // Add the start point to the vectors
        x_vec.push(leg.p1.y());
        y_vec.push(leg.p1.x());
    }
    // Add last point to the vectors
    let last_leg = route_plan.last().unwrap();
    x_vec.push(last_leg.p2.y());
    y_vec.push(last_leg.p2.x());

    // Add a line between the start and end points
    figure.add_trace(plotly::ScatterGeo::new(x_vec, y_vec)
        .mode(plotly::common::Mode::LinesMarkersText)
        .name("Route Plan"));

    // Init vectors for coordinates
    let mut x_vec_port: Vec<f64> = Vec::new();
    let mut y_vec_port: Vec<f64> = Vec::new();
    let mut x_vec_starboard: Vec<f64> = Vec::new();
    let mut y_vec_starboard: Vec<f64> = Vec::new();

    // TODO: Add tacking boundary
    for (i, leg) in route_plan.iter().enumerate() {
        // If tacking width changes between legs, we should first plot the previous width at the current location before plotting the new width
        if i > 0 && leg.tacking_width != route_plan[i-1].tacking_width {
            // Get last_leg
            let last_leg = &route_plan[i-1];
            // Get last tacking width
            let last_tacking_width = last_leg.tacking_width;
            // Get point half a tacking width to the left and right of the leg
            let bearing = Haversine.bearing(last_leg.p1, last_leg.p2);
            // Get the left and right points but at the location of the current leg
            let port_point = Haversine.destination(leg.p1, bearing - 90.0, last_tacking_width / 2.0);
            let starboard_point = Haversine.destination(leg.p1, bearing + 90.0, last_tacking_width / 2.0);

            x_vec_port.push(port_point.y());
            y_vec_port.push(port_point.x());
            x_vec_starboard.push(starboard_point.y());
            y_vec_starboard.push(starboard_point.x());
        }

        // Get point half a tacking width to the left and right of the leg
        let bearing = Haversine.bearing(leg.p1, leg.p2);
        // Get the left and right points
        let port_point = Haversine.destination(leg.p1, bearing - 90.0, leg.tacking_width / 2.0);
        let starboard_point = Haversine.destination(leg.p1, bearing + 90.0, leg.tacking_width / 2.0);

        x_vec_port.push(port_point.y());
        y_vec_port.push(port_point.x());
        x_vec_starboard.push(starboard_point.y());
        y_vec_starboard.push(starboard_point.x());
    }
    // Add last point to the vectors
    let bearing = Haversine.bearing(last_leg.p1, last_leg.p2);
    // Get the left and right points
    let port_point = Haversine.destination(last_leg.p2, bearing - 90.0, last_leg.tacking_width / 2.0);
    //let right_point = leg.p1.destination(leg.tacking_width / 2.0, bearing + 90.0);
    let starboard_point = Haversine.destination(last_leg.p2, bearing + 90.0, last_leg.tacking_width / 2.0);
    // Append points
    x_vec_port.push(port_point.y());
    y_vec_port.push(port_point.x());
    x_vec_starboard.push(starboard_point.y());
    y_vec_starboard.push(starboard_point.x());

    // Add a lines for the tacking boundary to plot
    figure.add_trace(plotly::ScatterGeo::new(x_vec_starboard, y_vec_starboard)
        .mode(plotly::common::Mode::LinesMarkersText)
        .name("Tacking boundary starboard side"));
    figure.add_trace(plotly::ScatterGeo::new(x_vec_port, y_vec_port)
        .mode(plotly::common::Mode::LinesMarkersText)
        .name("Tacking boundary port side"));
        //.line(plotly::Line::new().color("red")));



    // TODO: Add vector at each point that shows wind direction at that point at that points time?????


    // Open plot
    figure.show();

    // Save the figure to a file if file path is provided
    if let Some(file_path) = figure_file_path {
        figure.write_html(file_path);
    }

    // Return Ok if all went well
    return Ok(());
}



// Helper functions
//----------------------------------------------------
/// Converts a string into an uom::si::f64::Time object
/// time_string: The string to convert in the format YYYY-MM-DD hh:mm
/// # Example:
/// `let my_timestamp: uom::si::f64::Time = str_to_coordinate("52.5200,13.4050");`
pub fn string_to_utc_date_time(time_string: String) -> time::UtcDateTime {
    // Remove all whitespaces in string
    let mut working_str: &str = (&time_string[..]).trim();

    // If string is longer than 16 characters but shorter than 25, just take first 16 characters
    if working_str.len() > 16 && working_str.len() < 25 {
        working_str = &working_str[0..16];
    }

    // Check if the string is valid
    if !((working_str.len() == 16) || (working_str.len() == 25)) {
        panic!("Invalid time format with length {}:\n{}", working_str.len(), working_str);
    }

    // Get parts from string
    let year:    i32 = working_str[0..4].parse::<i32>().expect(format!("Invalid year: {}\nInput string: {}\nError\n", &working_str[0..4], working_str).as_str());
    let month = time::Month::try_from(working_str[5..7].parse::<u8>().expect(format!("Invalid month: {}\nInput string: {}\nError\n", &working_str[5..7], working_str).as_str())).expect("Invalid month");
    let day_of_month: u8 = working_str[8..10].parse::<u8>().expect(format!("Invalid day: {}\nInput string: {}\nError\n", &working_str[8..10], working_str).as_str());
    let date = time::Date::from_calendar_date(year, month, day_of_month).expect("Could not create time::Date from values");

    let hour: u8 = working_str[11..13].parse::<u8>().expect(format!("Invalid hour: {}\nInput string: {}\nError\n", &working_str[11..13], working_str).as_str());
    let minutes: u8 = working_str[14..16].parse::<u8>().expect(format!("Invalid minute: {}\nInput string: {}\nError\n", &working_str[14..16], working_str).as_str());
    let mut seconds: u8 = 0;
    // If we have seconds, get them
    if working_str.len() >= 19 {
        seconds = working_str[17..19].parse::<u8>().expect(format!("Invalid second: {}\nInput string: {}\nError\n", &working_str[17..19], working_str).as_str());
    }
    let time_hms = time::Time::from_hms(hour, minutes, seconds).expect("Could not create time::Time from values");

    // Attempt to parse the string into a uom::si::f64::Time object
    let time_out = time::UtcDateTime::new(date, time_hms);
    
    // Return
    return time_out;
}

/// Converts a time_stamp to a string in the format YYYY-MM-DD hh:mm
/// Is this function never called?
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
/// # Example:
/// `let my_coord: geo::Point = string_to_point("52.5200,13.4050");`
/// Note that the output is a geo::Point::new(longitude, latitude) but the input string must be in the format of latitude,longitude so the order is reversed
pub fn string_to_point(coord_string: String) -> geo::Point {
    // Remove all spaces in string
    let coord_str_vec: Vec<&str> = coord_string.trim().split(',').collect();

    // Check if the coordinates are valid, should have latitude and longitude
    if coord_str_vec.len() != 2 {
        panic!("Invalid coordinate format");
    }

    // Parse the latitude and longitude as f64
    let mut latitude: f64 = coord_str_vec[0].trim().parse::<f64>().expect(format!("Invalid latitude: {:?}", coord_str_vec).as_str());
    let mut longitude: f64 = coord_str_vec[1].trim().parse::<f64>().expect(format!("Invalid longitude: {:?}", coord_str_vec).as_str());

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

/// Get shortest distance between a line and a point on a sphere
/// The line is the haversine line with endpoints p1 and p2
/// Point p3 is the point that the shortest distance to the line between p1 and p2 will be calculated from.
/// The distance is calculated with the spherical law of sines
/// Returns the distance in meters
pub fn get_min_point_to_great_circle_dist(p1: geo::Point, p2: geo::Point, p3: geo::Point) -> f64 {
    // Quick check if already at end points
    if p1 == p3 || p2 == p3 {
        return 0.0;
    }
    // Using analytical solution from https://www.reddit.com/r/askmath/comments/1n6kc8d/whats_the_shortest_distance_d_from_a_point_on_a/
    // Where p1 is U, P2 is V and P3 is W.
    // Radius of sphere (Earth) is r
    let r = geo::Haversine.radius();
    // b is the distance from U to W (from p1 to p3)
    let b = geo::Haversine.distance(p1, p3);
    // Get the angle VUW (the angle between p2 and p3 as seen from p1), c_angle_radians is in [0, 2PI]
    let c_angle_radians = (geo::Haversine.bearing(p1, p2) - geo::Haversine.bearing(p1, p3)).abs() * consts::PI/180.0;

    // Calculate distance based on spherical law of sines https://en.wikipedia.org/wiki/Law_of_sines#Spherical_law_of_sines
    // Note b/r gives an angle in radians that should always be in [0, PI] meaning that (b/r).sin() is always zero or a positive number and
    // c_angle_radians.sin() can be a number in [-1,1] so we could be taking the arcsin of a negative number which results in a negative number
    // Since the distance is always the same, regardless of the sign, we take the absolute value
    let d = r*(c_angle_radians.sin() * (b/r).sin()).asin().abs();
    return d;
}

/// Converts a string into a f64 object
/// cargo_string: The string to convert, must be in metric tons (1 metric ton = 1000 kg)
/// # Example:
/// `let my_tons: f64 = string_to_tons("500.3");`
pub fn string_to_tons(cargo_string: String) -> Option<f64> {
    // Remove all spaces in string
    let cargo_str: &str = (&cargo_string[..]).trim();
    
    // Check if the string is valid
    if cargo_str.len() == 0 {
        return None;
    }

    // Parse the cargo as f64
    let cargo: f64 = cargo_str.parse::<f64>().expect("Invalid cargo");

    // return cargo
    return Some(cargo);
}

/// Returns the average and standard deviation of all values in a vector of f64 objects
/// data_vec: The vector of f64 objects
/// # Example:
/// `let (my_mean, my_std) = get_vec_f64_mean_and_std(&my_vec);`
pub fn get_vec_f64_mean_and_std(data_vec: &Vec<f64>) -> Result<(f64, f64), io::Error> {
    // Validate that the input vector has at least 1 value
    if data_vec.is_empty() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "vector is empty, cannot calculate mean and standard deviation"));
    }
    
    // Calculate the mean of the vector
    let mut total: f64 = 0.0;

    // loop through vector, add all values to total
    for value in data_vec {
        total = total + *value;
    }
    // Find mean
    let vec_mean: f64 = total / data_vec.len() as f64;

    // Calculate the standard deviation of the speed vector
    let mut variance: f64 = 0.0;

    // loop through vector, add all values to variance, then divide by number of values -1 to create variance
    for value in data_vec {
        variance = variance + (value - vec_mean).powi(2);
    }
    variance = variance / ((data_vec.len() - 1) as f64);

    // Find standard deviation from variance
    let vec_std: f64 = variance.sqrt();

    // Return the mean and standard deviation
    return Ok((vec_mean, vec_std));
}


/// Returns the average and standard deviation of a vector
/// # Example:
/// `let (my_mean, my_std) = get_time_mean_and_std(&my_vec);`
pub fn get_duration_mean_and_std(duration_vec: &Vec<time::Duration>) ->
    Result<(time::Duration, time::Duration), io::Error> {
        println!("DURATION EVALUATION THINGY: DURATION VEC: {:?}", duration_vec);
        println!("DURATION VEC IS EMPTY?: {:?}", duration_vec.is_empty());
    // Validate that the vector has at least 1 value
    if duration_vec.is_empty() {
        println!("DURATION VEC IS EMPTY? DOUBLE TAKE: {:?}", duration_vec.is_empty());
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Duration vector is empty, cannot calculate mean and standard deviation"));
    }

    // Calculate the mean of the vector
    let mut tot_duration = time::Duration::new(0, 0);

    // loop through vector, add all values to the total
    for duration in duration_vec {
        tot_duration = tot_duration + *duration;
    }
    // Find mean
    let duration_mean = tot_duration.checked_div(duration_vec.len() as i32).unwrap();

    // Calculate the standard deviation of the vector
    let mut variance: i64 = 0;

    // loop through vector, add all values to variance, then divide by number of values -1 to create variance
    for duration in duration_vec {
        let multiplier = duration.checked_sub(duration_mean).expect("Could not subtract value from time::Duration. Maybe an overflow occurred?").whole_seconds();
        variance = variance + multiplier*multiplier;
    }
    let variance_f64 = (variance as f64) / ((duration_vec.len() - 1) as f64);

    // Find standard deviation from variance
    let duration_std = time::Duration::new((variance_f64).sqrt() as i64, 0);

    // Return the mean and standard deviation
    return Ok((duration_mean, duration_std));
}


/// Loads route plan from a CSV file
/// Returns a vector of SailingLeg objects where each entry is a a leg of the trip
/// The CSV file is expected to have the following columns in order but the header names are not important:
/// Leg number;start_latitude;start_longitude;end_latitude;end_longitude;tacking_width\[meters\]
/// The delimiter is a semicolon.
/// file_path: Path to the CSV file
/// # Example:
/// `let file_path: &str = "my_route_plan.csv";`
pub fn load_route_plan(file_path: &str) -> Vec<SailingLeg> {
    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(file_path)
        .expect(format!("Failed to open route plan file: {}", file_path).as_str());

    // Initialize a vector to store the route plan
    let mut route_plan: Vec<SailingLeg> = Vec::new();

    // Iterate through each line of the CSV file and add the coordinates to the route plan
    for result in csv_reader.records() {
        match result {
            Ok(leg) => {
                // Get the SailingLeg data from the CSV file
                // First column is the leg number, so we skip it
                // Start_coord
                let start_lat = leg.get(1).expect("Start latitude missing from route plan").to_string();
                let start_long = leg.get(2).expect("Start longitude missing from route plan").to_string();
                // End_coord
                let end_lat = leg.get(3).expect("End latitude missing from route plan").to_string();
                let end_long = leg.get(4).expect("End longitude missing from route plan").to_string();
                // Tacking width
                let tacking_width = leg.get(5).expect("Tacking width missing from route plan").to_string();
                // Get minimum proximity
                let min_prox = leg.get(6).expect("Minimum proximity missing from route plan").to_string();

                // Make a SailingLeg object
                let temp_sailing_leg: SailingLeg = SailingLeg {
                    p1: string_to_point(format!("{},{}", start_lat, start_long)),
                    p2: string_to_point(format!("{},{}", end_lat, end_long)),
                    tacking_width: tacking_width.parse::<f64>().expect("Invalid tacking width"),
                    min_proximity: min_prox.parse::<f64>().expect("Invalid minimum proximity"),
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
/// ship_logs: The ship logs from the vessel
/// Note: The csv file delimiter is a semicolon
pub fn ship_logs_to_csv(csv_file_path: &str, ship_logs: &Vec<ShipLogEntry>) -> Result<(), io::Error> {
    // Create a CSV writer with a semicolon delimiter
    // let mut wtr = csv::WriterBuilder::new().delimiter(b';').from_path(csv_file_path)?;
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(csv_file_path)?;

    // Write the header
    wtr.write_record(&["timestamp", "coordinates_initial", "coordinates_current", "coordinates_final", "cargo_on_board[ton]", "velocity[m/s]", "course[°]", "heading", "true_bearing[°]", "draught[m]", "navigation_status"])?;

    // Write the ship log entries
    for entry in ship_logs.iter() {
        let mut _timestamp_string: String = String::new();  //Underscored to avoid unused variable warning since it is used in wtr.write_record
        _timestamp_string.push_str(entry.timestamp.year().to_string().as_str());
        _timestamp_string.push_str("-");
        // If month is 1 digit, add a leading zero
        if (entry.timestamp.month() as i16) < 10 {
            _timestamp_string.push_str("0");
        }
        _timestamp_string.push_str((entry.timestamp.month() as i8).to_string().as_str());
        _timestamp_string.push_str("-");
        // If day is 1 digit, add a leading zero
        if entry.timestamp.day() < 10 {
            _timestamp_string.push_str("0");
        }
        _timestamp_string.push_str(entry.timestamp.day().to_string().as_str());
        _timestamp_string.push_str(" ");
        // If hour is 1 digit, add a leading zero
        if entry.timestamp.hour() < 10 {
            _timestamp_string.push_str("0");
        }
        _timestamp_string.push_str(entry.timestamp.hour().to_string().as_str());
        _timestamp_string.push_str(":");
        // If minute is 1 digit, add a leading zero
        if entry.timestamp.minute() < 10 {
            _timestamp_string.push_str("0");
        }
        _timestamp_string.push_str(entry.timestamp.minute().to_string().as_str());
        _timestamp_string.push_str(":");
        // If second is 1 digit, add a leading zero
        if entry.timestamp.second() < 10 {
            _timestamp_string.push_str("0");
        }
        _timestamp_string.push_str(entry.timestamp.second().to_string().as_str());

        // If cargo is None, set to empty string
        let cargo = match entry.cargo_on_board {
            Some(c) => c.get::<uom::si::mass::ton>().to_string(),
            None => String::from(""),
        };

        // If velocity is None, set to empty string
        let velocity = match entry.velocity {
            Some(v) => v.magnitude.to_string(),
            None => String::from(""),
        };

        // If course is None, set to empty string
        let course = match entry.course {
            Some(c) => c.to_string(),
            None => String::from(""),
        };

        // If heading is None, set to empty string
        let heading = match entry.heading {
            Some(h) => h.to_string(),
            None => String::from(""),
        };

        // If true_bearing is None, set to empty string
        let true_bearing = match entry.true_bearing {
            Some(tb) => tb.to_string(),
            None => String::from(""),
        };

        // If draught is None, set to empty string
        let draft = match entry.draft {
            Some(d) => d.to_string(),
            None => String::from(""),
        };

        // If navigation_status is None, set to empty string
        let navigation_status = match &entry.navigation_status {
            Some(ns) => (*ns as u64).to_string(),
            None => String::from(""),
        };

        // Write the record
        wtr.write_record(&[
            _timestamp_string, //entry.timestamp.to_string(), // timestamp_to_string(entry.timestamp),
            format!("{},{}", entry.coordinates_initial.y(), entry.coordinates_initial.x()),
            format!("{},{}", entry.coordinates_current.y(), entry.coordinates_current.x()),
            format!("{},{}", entry.coordinates_final.y(), entry.coordinates_final.x()),
            cargo,            // entry.cargo_on_board.unwrap().get::<uom::si::mass::ton>().to_string(),
            velocity,
            course,
            heading,
            true_bearing,
            draft,
            navigation_status,
        ])?;
    }

    // Flush and close the writer
    wtr.flush()?;
    Ok(())
}

/// The reciprocal function to ship_logs_to_csv takes a csv file and returns the ship logs.
/// Function that writes the ship logs to a CSV file with the following columns:
/// timestamp;coordinates_initial;coordinates_current;coordinates_final;cargo_on_board
/// Note that the coordinates are in the format of ISO 6709 using decimal places with a comma between latitude and longitude. "latitude,longitude" (e.g., "52.5200,13.4050")
/// The cargo is considered to be in metric tons (1 metric ton = 1000 kg)
/// csv_file_path: Path to the CSV file
/// boat: The boat object containing the ship logs
/// Note: The csv file delimiter is a semicolon
/// Note: It is assumed that the course over ground is specified in 10 times the degrees. That is 3593 degrees course in the csv file are 359.3 degrees 
pub fn csv_to_ship_log(csv_file_path: &str) -> Result<Vec<ShipLogEntry>, io::Error> {
    // Check if the file ends with ".csv" and if it does not return an error
    if csv_file_path.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect::<String>() != ".csv" {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("File path does not end with .csv\nFile: {:?}", csv_file_path)));
    }
 
    // Init empty Ship Log book
    let mut ship_log = vec![];

    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(csv_file_path)
        .expect(format!("Failed to open file: {}\n", csv_file_path).as_str());

    // Loop through all lines of file, append each line to the ship log
    for result in csv_reader.records() {
        match result {
            Ok(entry) => {
                let timestamp = string_to_utc_date_time(entry.get(0).unwrap().to_owned());
                let coordinates_initial = geo::Point::new(
                    entry.get(1).unwrap().split(',').next().unwrap().parse::<f64>().expect("Error getting initial coordinates from csv file"),
                    entry.get(1).unwrap().split(',').nth(1).unwrap().parse::<f64>().expect("Error getting initial coordinates from csv file")
                );
                let coordinates_current = geo::Point::new(
                    entry.get(2).unwrap().split(',').next().unwrap().parse::<f64>().expect("Error getting current coordinates from csv file"),
                    entry.get(2).unwrap().split(',').nth(1).unwrap().parse::<f64>().expect("Error getting current coordinates from csv file")
                );
                let coordinates_final = geo::Point::new(
                    entry.get(3).unwrap().split(',').next().unwrap().parse::<f64>().expect("Error getting final coordinates from csv file"),
                    entry.get(3).unwrap().split(',').nth(1).unwrap().parse::<f64>().expect("Error getting final coordinates from csv file")
                );
                // If there is no cargo written down, set to None
                let cargo_on_board = match entry.get(4).unwrap() {
                    "" => None,
                    cargo => Some(uom::si::f64::Mass::new::<uom::si::mass::ton>(cargo.parse::<f64>().unwrap())),
                };
                // If no course written down, set to None
                let course = match entry.get(6).unwrap() {
                    "" => None,
                    course => Some(course.parse::<f64>().unwrap()/10.0),
                };
                // Init velocity
                let velocity: Option<PhysVec>;
                // If speed and course are known, set velocity PhysVec to use them otherwise set velocity to None
                if entry.get(5).unwrap().parse::<f64>().is_ok() && course.is_some() {
                    velocity = Some(PhysVec::new(entry.get(5).unwrap().parse::<f64>().expect("Error getting velocity from csv file"), course.expect("Unknown course over ground when parsing velocity from ship log csv file")));
                }
                else {
                    velocity = None;
                }
                // if there is a heading written down, set the heading to that, otherwise, set to None
                let heading: Option<f64> = match entry.get(7).unwrap() {
                    "" => None,
                    h => Some(h.parse::<f64>().unwrap()),
                };
                // Track angle is between last and current ship log entry, if this is the first entry, set to None
                let track_angle = match ship_log.len() {
                    0 => None,
                    _ => {
                        let last_entry: &ShipLogEntry = ship_log.last().unwrap();
                        let last_coords: geo::Point = last_entry.coordinates_current;
                        let curr_coords: geo::Point = coordinates_current;
                        Some(geo::Haversine.bearing(last_coords, curr_coords))
                    }
                };
                // If no true_bearing written down, set to None
                let true_bearing = match entry.get(8).unwrap() {
                    "" => None,
                    bearing => Some(bearing.parse::<f64>().unwrap()),
                };
                // If no draft written doen, set to None
                let draft = match entry.get(9).unwrap() {
                    "" => None,
                    draft => Some(draft.parse::<f64>().unwrap()),
                };
                let navigation_status: Option<NavigationStatus> = match NavigationStatus::try_from(entry.get(10).unwrap().parse::<u8>().unwrap()) {
                    Ok(status) => Some(status),
                    Err(_) => None,                    
                }; //Some(entry.get(10).map(|s| s.parse::<u8>().expect("Failed to parse navigation status")).expect("Failed to parse navigation status"));

                ship_log.push(
                    ShipLogEntry {
                        timestamp,
                        coordinates_initial,
                        coordinates_current,
                        coordinates_final,
                        cargo_on_board,
                        velocity,
                        course,
                        heading,
                        track_angle,
                        true_bearing,
                        draft,
                        navigation_status,
                    });
                }
            Err(err) => {
                eprintln!("Error reading ship log entry: {}", err);
                }
            }
        }

    // Return ship log
    return Ok(ship_log);
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

/// Function that gets the angle from north given the northward PhysVec property (effectively, the magnitude going from north to south) and eastward PhysVec property (effectively, the magnitude going from west to east)
pub fn get_north_angle_from_northward_and_eastward_property(eastward: f64, northward: f64) -> f64 {
    let mut north_angle = northward.atan2(eastward) * 180.0 / std::f64::consts::PI;

    // Adjust result from atan2 to be between 0 and 360
    if north_angle < 0.0 {
        north_angle += 360.0;
    }
    
    // transform angle to be based from north not east
    north_angle -= 90.0;

    // Adjusting if went out of bounds
    while north_angle >= 360.0 {
        north_angle -= 360.0;
    }
    while north_angle < 0.0 {
        north_angle += 360.0;
    }

    return north_angle;
}

/// Segments a waypoint mission
pub fn segment_waypoint_mission(route_plan: Vec<SailingLeg>, n_segments: u64) -> (Vec<geo::Point>, f64) {
    // Get total length of route, in meters, if going shortest path
    let mut total_dist: f64 = 0.0;

    // For each leg, add leg distance to total_dist
    for leg in &route_plan {
        // get leg points
        let p1 = leg.p1;
        let p2 = leg.p2;
        total_dist += geo::Haversine.distance(p1, p2);
    }

    // Get number of segments with a sanity check against zero n_segments:
    let n: u64;
    if n_segments < 1 {
        n = 1;
    } else {
        n = n_segments;
    }

    // Get length of each segment
    let segment_dist: f64 = total_dist/(n as f64);

    // Make points of analysis using the path along the waypoint route and the n_segments from the simulaion
    // Init with first point
    let mut waypoints: Vec<geo::Point> = vec![route_plan[0].p1];

    // Until the last waypoint has been reached, travel the segment_dist along the path and add the waypoint to the waypoints vector
    // Init distance left of segment
    let mut seg_dist_left: f64;
    // Init leg number
    let mut current_leg = 0;
    // Init boat location
    let mut location = route_plan[0].p1;
    let mut num_points = 1;
    while waypoints.last().unwrap() != &route_plan.last().unwrap().p2 {
        // If number of waypoints is number of segments then the next waypoint should be the last point, make it happen explicitly since floating point error can
        // cause issues where we have 1 too many points otherwise
        if num_points == n_segments {
            waypoints.push(route_plan.last().unwrap().p2);
            break;
        }
        // Reset seg_dist_left
        seg_dist_left = segment_dist.clone();

        // While there is still distance left in seg_dist_left, keep going towards the next point on the leg
        while seg_dist_left > 0.0 {
            // Get p2 of the current leg
            let p2 = route_plan[(current_leg) as usize].p2;

            // Get distance to next waypoint
            let dist_to_next_waypoint = geo::Haversine.distance(location, p2);

            // If distance to next waypoint is shorter or same as segment distance left, then travel to waypoint, update boat location and leg number and reduce segment distance left
            if dist_to_next_waypoint <= seg_dist_left {
                location = p2;
                current_leg = current_leg + 1;

                seg_dist_left -= dist_to_next_waypoint;

                // If last point, set seg_dist_left to zero
                if location == route_plan.last().unwrap().p2 {
                    seg_dist_left = 0.0;
                }
            }
            // Else, travel the segment distance along the path between the last waypoint and the next waypoint, set the seg_dist_left to zero and append that point to waypoints
            else {
                let heading = geo::Haversine.bearing(location, p2);
                location = geo::Haversine.destination(location, heading, seg_dist_left);
                seg_dist_left = 0.0;
            }
        }
        // Add location
        waypoints.push(location);
        num_points += 1;
    }

    // Return waypoints and segment distance
    return (waypoints, segment_dist);
}

/// Downloads the weather data needed to run the fast_sim_waypoint_mission_weather_data_from_copernicus
/// points: the locations to get weather data for
/// timestamp: the time that the weather happened
/// path_to_file: where to save the data
pub fn get_weather_data_for_points(points: Vec<geo::Point>, timestamp: UtcDateTime, path_to_file: String, copernicus: copernicusmarine_rs::Copernicus) -> Result<String, io::Error> {
    println!("Getting weather data");
    // Initialize weather data vectors
    let mut wind_vec: Vec<PhysVec> = Vec::new();
    let mut ocean_current_vec: Vec<Option<PhysVec>> = Vec::new();
    // Get number of points
    let num_points = points.len();

    // Start a progress bar with twice the tasks as num_points
    let progress_bar = indicatif::ProgressBar::new((num_points) as u64);
    // Set progress bar style
    progress_bar.set_style(indicatif::ProgressStyle::with_template("[{elapsed_precise}] {bar} {pos:>3}/{len:3} ETA:{duration_precise:>1}").unwrap()); //.progress_chars("##-"));
    // Configure live redraw
    progress_bar.set_draw_target(indicatif::ProgressDrawTarget::stdout());
    progress_bar.enable_steady_tick(std::time::Duration::from_millis(500));
    // Start progress bar
    progress_bar.inc(0);

    // For each point, get the wind and current
    for i in 0..num_points {
        // Get the wind data
        let dataset_id: String = match copernicusmarine_rs::get_dataset_id(copernicusmarine_rs::CopernicusVariable::EastwardWind, timestamp, timestamp) {
            Ok(id) => id,
            Err(e) => panic!("Error getting dataset id from copernicusmarine: {}", e),
        };
        // let wind_data = match copernicus.get_f64_values("cmems_obs-wind_glo_phy_nrt_l4_0.125deg_PT1H".to_string(), vec!["eastward_wind".to_string(), "northward_wind".to_string()], timestamp, timestamp, points[i].x(), points[i].x(), points[i].y(), points[i].y(), None, None) {
        let wind_data = match copernicus.get_f64_values(dataset_id, vec!["eastward_wind".to_string(), "northward_wind".to_string()], timestamp, timestamp, points[i].x(), points[i].x(), points[i].y(), points[i].y(), None, None) {
            Ok(w) => w,
            Err(e) => panic!("Error getting wind data from copernicusmarine: {}", e),
        };
        let wind_east_data = &wind_data[0];
        let wind_north_data = &wind_data[1];

        // Wind speed and direction
        let wind_east: f64 = wind_east_data[0].unwrap();
        let wind_north: f64 = wind_north_data[0].unwrap();
        let wind_angle: f64 = get_north_angle_from_northward_and_eastward_property(wind_east, wind_north);   // Angle in degrees
        let wind_speed = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>((wind_east*wind_east + wind_north*wind_north).sqrt().into());
        wind_vec.push(PhysVec::new(wind_speed.get::<uom::si::velocity::meter_per_second>(), wind_angle));    // unit [m/s]

        // Get ocean current data from Copernicus
        // "uo" is the eastward sea water velocity and "vo" is the northward sea water velocity
        let dataset_id: String = match copernicusmarine_rs::get_dataset_id(copernicusmarine_rs::CopernicusVariable::EastwardSeaWaterVelocity, timestamp, timestamp) {
            Ok(id) => id,
            Err(e) => panic!("Error getting dataset id from copernicusmarine: {}", e),
        };
        // let ocean_current_data = match copernicus.get_f64_values("cmems_mod_glo_phy-cur_anfc_0.083deg_PT6H-i".to_string(), vec!["uo".to_string(), "vo".to_string()], timestamp, timestamp, points[i].x(), points[i].x(), points[i].y(), points[i].y(), Some(1.0), Some(1.0)){
        // let ocean_current_data = match copernicus.get_f64_values(dataset_id, vec!["uo".to_string(), "vo".to_string()], timestamp, timestamp, points[i].x(), points[i].x(), points[i].y(), points[i].y(), Some(0.49402499198913574), Some(0.49402499198913574)){
        // let ocean_current_data = match copernicus.get_f64_values(dataset_id, vec!["uo".to_string(), "vo".to_string()], timestamp, timestamp, points[i].x(), points[i].x(), points[i].y(), points[i].y(), Some(0.0), Some(50.0)){
        let ocean_current_data = match copernicus.get_f64_values(dataset_id, vec!["uo".to_string(), "vo".to_string()], timestamp, timestamp, points[i].x(), points[i].x(), points[i].y(), points[i].y(), Some(0.0), Some(1.0)){
            Ok(o) => o,
            Err(e) => panic!("Error getting ocean current data from copernicusmarine: {}", e),
        };
        let ocean_current_east_data = &ocean_current_data[0];
        let ocean_current_north_data = &ocean_current_data[1];

        // Ocean current speed and direction
        // If we don't have ocean_current data, push None to ocean_current_vec.
        if ocean_current_east_data[0].is_none() && ocean_current_north_data[0].is_none() {
            ocean_current_vec.push(None);
        }
        else {
            let mut ocean_current_east: f64 = 0.0;
            let mut ocean_current_north: f64 = 0.0;
            if ocean_current_east_data[0].is_some() {
                ocean_current_east = ocean_current_east_data[0].expect("Ocean current fill value?");
            }
            if ocean_current_north_data[0].is_some() {
                ocean_current_north = ocean_current_north_data[0].expect("Ocean current fill value?");
            }
            let ocean_current_angle: f64 = get_north_angle_from_northward_and_eastward_property(ocean_current_east, ocean_current_north);   // Angle in degrees
            let ocean_current_speed = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>((ocean_current_east*ocean_current_east + ocean_current_north*ocean_current_north).sqrt().into());
            ocean_current_vec.push(Some(PhysVec::new(ocean_current_speed.get::<uom::si::velocity::meter_per_second>(), ocean_current_angle)));    // unit [m/s]
        }

        // Update progress bar
        progress_bar.inc(1);
    }

    // Save all the points in a csv file
    // Check if csv_file_path ends with ".csv"
    let num_chars = path_to_file.chars().count();
    if &path_to_file[(num_chars-4)..] != ".csv" {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "The filepath must end with \".csv\""));
    }


    // Check if vectors are the same size
    if &wind_vec.len() != &ocean_current_vec.len() || wind_vec.len() != num_points {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "All vectors must have the same length"));
    }

    // Create a CSV writer with a semicolon delimiter
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(path_to_file)?;

    // Write the header
    wtr.write_record(&["time","longitude","latitude","wind_speed[m/s]","wind_angle[°]","ocean_current_speed[m/s]","ocean_current_angle[°]"])?;

    // Write the ship log entries
    for i in 0..num_points {
        // Get longitude
        let longitude = points[i].x().to_string();
        // Get latitude
        let latitude = points[i].y().to_string();
        // Get wind_speed
        let wind_speed = wind_vec[i].magnitude.to_string();
        // Get wind_angle
        let wind_angle = wind_vec[i].angle.to_string();
        // Get ocean_current_speed and angle
        let ocean_current_speed: String;
        let ocean_current_angle: String;
        if ocean_current_vec[i].is_some() {
            ocean_current_speed = ocean_current_vec[i].unwrap().magnitude.to_string();
            ocean_current_angle = ocean_current_vec[i].unwrap().angle.to_string();
        }
        else {
            ocean_current_speed = "".to_string();
            ocean_current_angle = String::new();
            // ocean_current_speed = "None".to_string();
            // ocean_current_angle = "None".to_string();
        }

        // Write the record
        wtr.write_record(&[
            copernicusmarine_rs::utc_date_time_to_string(timestamp),
            longitude,
            latitude,
            wind_speed,
            wind_angle,
            ocean_current_speed,
            ocean_current_angle,])?;        
    }

    // Flush and close the writer
    wtr.flush()?;

    // Finish progress_bar
    progress_bar.finish();

    // Return ok
    return Ok("weather data retrieved and saved successfully".to_string());
}

/// Function that gets weather data from file
/// The output tuple is the (timstamp, location, wind vector, ocean current vector)
pub fn get_weather_data_from_csv_file(path_to_file: String) -> (Vec<UtcDateTime>, Vec<geo::Point>, Vec<PhysVec>, Vec<Option<PhysVec>>) {
    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(path_to_file.clone())
        .expect(format!("Failed to open file: {}\n", path_to_file.as_str()).as_str());

    // Initialize return vectors
    let mut timestamps: Vec<UtcDateTime> = Vec::new();
    let mut points: Vec<geo::Point> = Vec::new();
    let mut wind_vec: Vec<PhysVec> = Vec::new();
    let mut ocean_current_vec: Vec<Option<PhysVec>> = Vec::new();

    // Iterate through each line of the CSV file and add the coordinates to the route plan
    for result in csv_reader.records() {
        match result {
            Ok(entry) => {
                // timestamp
                let timestamp = entry.get(0).expect("timestamp missing from weather data file").to_string();
                timestamps.push(string_to_utc_date_time(timestamp));
                // Point
                let longitude: f64 = entry.get(1).expect("longitude missing from weather data file").parse::<f64>().unwrap();
                let latitude: f64 =  entry.get(2).expect("latitude missing from weather data file").parse::<f64>().unwrap();
                let point = geo::Point::new(longitude, latitude);
                points.push(point);
                // Wind
                let wind_speed = entry.get(3).expect("wind speed missing from weather data file").parse::<f64>().unwrap();
                let wind_angle = entry.get(4).expect("wind angle missing from weather data file").parse::<f64>().unwrap();
                let wind = PhysVec::new(wind_speed, wind_angle);
                wind_vec.push(wind);
                // Ocean current
                let ocean_current_speed_csv_entry = entry.get(5).unwrap();
                let ocean_current_angle_csv_entry = entry.get(6).unwrap();
                let ocean_current_speed: f64;
                let ocean_current_angle: f64;
                let ocean_current: Option<PhysVec>;
                if ocean_current_speed_csv_entry == "" || ocean_current_angle_csv_entry == "" {
                    ocean_current = None;
                } else {
                    ocean_current_speed = ocean_current_speed_csv_entry.parse::<f64>().unwrap();
                    ocean_current_angle = ocean_current_angle_csv_entry.parse::<f64>().unwrap();
                    ocean_current = Some(PhysVec::new(ocean_current_speed, ocean_current_angle));
                }
                ocean_current_vec.push(ocean_current);
            }
            Err(err) => {
                eprintln!("Error reading weather data from file: {}", err);
            }
        }
    }
    
    // Return (timestamps, points, wind_vec, ocean_current_vec)
    return (timestamps, points, wind_vec, ocean_current_vec);
}


/// Function that saves the settings of the simulation to a text file.
/// Note: This function does not care about overwriting existing files, it will always overwrite.
pub fn save_sim_settings_to_file(file_path: &str, sim: Simulation) -> Result<(), io::Error> {
    // Check that file_path ends with ".txt"
    let num_chars = file_path.chars().count();
    if &file_path[(num_chars-4)..] != ".txt" {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "The filepath must end with \".txt\""));
    }

    // Make string to write to file
    let mut settings_string: String = String::new();
    settings_string.push_str("Simulation settings:\n");
    settings_string.push_str(&format!("Simulation method: {:?}\n", sim.simulation_method));
    settings_string.push_str(&format!("Simulation start times: {:?}\n", sim.start_times));
    settings_string.push_str(&format!("Simulation time step: {}\n", sim.time_step));
    settings_string.push_str(&format!("Simulation max iterations: {}\n", sim.max_iterations));
    settings_string.push_str(&format!("Simulation weather_data_file: {:?}\n", sim.weather_data_file));
    settings_string.push_str(&format!("Simulation copernicus: {:?}\n", sim.copernicus));
    settings_string.push_str(&format!("Simulation progress bar: {:?}\n", sim.progress_bar));
    settings_string.push_str(&format!("Simulation number of segments: {:?}\n", sim.n_segments));

    // Write string to file
    println!("Saving simulation settings to file: {}", settings_string);

    let mut f = File::create(file_path)?;
    f.write_all(&settings_string.into_bytes())?;


    // Return ok
    return Ok(());
}

/// Function that takes generates and saves a polar speed plot csv file
/// for a wind propelled vessel.
/// Based on this issue: https://github.com/G0rocks/marine_vessel_simulator/issues/50
/// The file_path is where the result will be saved as a csv file
/// Until this issue has been dealt with (https://github.com/G0rocks/marine_vessel_simulator/issues/42) then marine_vessel_simulator does not support using the polar plot but it can be uploaded to openCPN or similar programs to use them.
/// The polar plot data vector columns are: Column 1 is the apparent wind angle, column 2 is the apparent wind speed and column 3 is the vessel speed through water
/// The navigation status filter will set it so that the polar speed plot is made up of only ship log entries which are logged under the same status. Set to None to use all values in the ship log.
/// Warning: All calculations assume meters per second are being used and if knots are being used the vessel speed will be multiplied by 1.94384 to transform into knots and the columns (with the wind speed) will be multiplied by 2 (to ensure that the file can be opened by openCPN) meaning that if knots are used then the potentially there will be issues in using the data than if meters per second are used.
/// Note: If not ocean current data is retrieved, the current is assumed to be flowing at zero meters per second
/// Note: If no degree_segment_size is given, defaults to 5°. If a segment size is given it must be so that 180° is divisible by the segment size
/// Note: If no wind_speed_segment_size is given, defaults to 1 m/s. If a segment size is given it must be so that 40 m/s is divisible by the segment size. Will always use m/s and not knots.
/// Note: As of 2026-02-06 OpenCPN polar plugin only accepts values in degree increments of 5° and column increments of 2 (no unit). In order to generate a polar speed plot csv file which can be opened by this plugin the same constraints are put on the input degree and wind speed segment sizes, that is that they must be divisible by 5° and 2 m/s. Follow this issue for updates: https://github.com/G0rocks/marine_vessel_simulator/issues/56
pub fn make_polar_speed_plot_csv(ship_log: Vec<ShipLogEntry>, simulation: &Simulation, file_path: &str, true_if_knots_false_if_meters_per_second: bool, degree_segment_size: Option<f64>, wind_speed_segment_size: Option<f64>, navigation_status_filter: Option<NavigationStatus>) -> Result<Vec<Vec<f64>>, io::Error> {
    // Add ".csv" to the end of the file path if it is not there already
    let mut working_file_path: String = file_path.to_owned();
    if file_path.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect::<String>() != ".csv" {
        working_file_path = file_path.to_owned() + ".csv";
    }

    // Get working degree segment size from degree_segment_size and evaluate if it is so that 180° are divisible by it
    let working_degree_segment_size: f64 = degree_segment_size.unwrap_or_else(|| 5.0);
    if 180.0 % working_degree_segment_size != 0.0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid input: degree segment size: {}°.\nThe degree segment size must be so that 180° is divisible by the angle", working_degree_segment_size)));
    }
    if working_degree_segment_size % 5.0 != 0.0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid input: degree segment size: {}°.\nThe degree segment size must be divisible by 5° to ensure compatibility with openCPN polar plugin", working_degree_segment_size)));
    }
    // Get working wind speed segment size from wind_speed_segment_size and evaluate if it is so that 40 m/s is divisible by it
    let working_wind_speed_segment_size: f64 = wind_speed_segment_size.unwrap_or_else(|| 1.0);
    if 40.0 % working_wind_speed_segment_size != 0.0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid input: wind speed segment size: {} m/s.\nThe wind speed segment size must be so that 40 m/s is divisible by the segment size", working_wind_speed_segment_size)));
    }
    if working_wind_speed_segment_size % 2.0 != 0.0 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Invalid input: wind speed segment size: {} m/s.\nThe wind speed segment size must be divisible by 2 m/s to ensure compatibility with openCPN polar plugin", working_wind_speed_segment_size)));
    }

    // Init empty polar plot data vector which will have subvectors. Column 1 is the apparent wind angle, column 2 is the apparent wind speed and column 3 is the vessel speed through water
    let mut polar_plot_data_vector: Vec<Vec<f64>> = Vec::new();

    // Check for interactive terminal for progress bar
    let is_interactive_terminal = atty::is(atty::Stream::Stdout);

    // If simulation has progress bar, set it up and use it
    if !(simulation.progress_bar.is_none()) {
        // Set length
        // Hiding progress bar to prevent two lines of progress bar being drawn
        simulation.progress_bar.as_ref().unwrap().set_draw_target(indicatif::ProgressDrawTarget::hidden());
        simulation.progress_bar.as_ref().unwrap().set_length((ship_log.len() as u64) + 1);
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

    // Loop through ship_log
    for entry in ship_log {
        // If filtering by navigational status, check if this entry's navigational status is the correct one
        if navigation_status_filter.is_some() {
            // variable that tells us if we need to skip this entry or not
            let mut skip_this_entry: bool = false;

            // Check if the ship log entry has a navigational status, if not, skip this entry since we are filtering by navigational status
            if entry.navigation_status.is_some() {
                // If the navigational status of the ship log entry is different from the filter then skip this entry. Make sure navigation_status is some first
                if entry.navigation_status.unwrap() != navigation_status_filter.unwrap() {
                    skip_this_entry = true;
                }
            }
            else {
                skip_this_entry = true;
            }


            // If we should skip this entry, update the progress bar and continue to next entry
            if skip_this_entry {
                // Update progress bar if a progress bar is in use
                if !(simulation.progress_bar.is_none()) {
                    // update progress bar
                    simulation.progress_bar.as_ref().unwrap().inc(1);
                    // If not interactive terminal, print progressbar manually
                    if is_interactive_terminal == false {
                        let eta = time::UtcDateTime::now().saturating_add(time::Duration::new(simulation.progress_bar.as_ref().unwrap().eta().as_secs() as i64, 0)); // What time the simulations will end
                        println!("Elapsed: {} secs, Steps {}/{}, ETA: {}-{}-{} {}:{}:{}", simulation.progress_bar.as_ref().unwrap().elapsed().as_secs(), simulation.progress_bar.as_ref().unwrap().position(), simulation.progress_bar.as_ref().unwrap().length().unwrap(), eta.year(), eta.month() as u8, eta.day(), eta.hour(), eta.minute(), eta.second());
                    }   // End if
                }   // End if

                // Continue to next entry in the ship logs
                continue;
            }
        }
        // Get timestamp
        let timestamp = entry.timestamp;

        // Get vessel location coordinates
        let location = entry.coordinates_current;
        let longitude = location.x();
        let latitude = location.y();
        
        // Get velocity and heading
        let vessel_velocity = entry.velocity;
        let heading = entry.heading;

        // If vessel velocity or heading is None skip this entry since we need both data fields
        if vessel_velocity == None || heading == None {
            // Update progress bar if a progress bar is in use
            if !(simulation.progress_bar.is_none()) {
                // update progress bar
                simulation.progress_bar.as_ref().unwrap().inc(1);
                // If not interactive terminal, print progressbar manually
                if is_interactive_terminal == false {
                    let eta = time::UtcDateTime::now().saturating_add(time::Duration::new(simulation.progress_bar.as_ref().unwrap().eta().as_secs() as i64, 0)); // What time the simulations will end
                println!("Elapsed: {} secs, Steps {}/{}, ETA: {}-{}-{} {}:{}:{}", simulation.progress_bar.as_ref().unwrap().elapsed().as_secs(), simulation.progress_bar.as_ref().unwrap().position(), simulation.progress_bar.as_ref().unwrap().length().unwrap(), eta.year(), eta.month() as u8, eta.day(), eta.hour(), eta.minute(), eta.second());
                }   // End if
            }   // End if

            // Skip to next iteration of loop
            continue;
        }

        // Get wind and ocean current data from timestamp and location from Copernicus
        let dataset_id: String = match copernicusmarine_rs::get_dataset_id(copernicusmarine_rs::CopernicusVariable::EastwardWind, timestamp, timestamp) {
            Ok(id) => id,
            Err(e) => panic!("Error getting dataset id from copernicusmarine: {}", e),
        };
        // let wind_data = match simulation.copernicus.as_ref().unwrap().get_f64_values("cmems_obs-wind_glo_phy_nrt_l4_0.125deg_PT1H".to_string(), vec!["eastward_wind".to_string(), "northward_wind".to_string()], boat_time_now, boat_time_now, longitude, longitude, latitude, latitude, None, None) {
        let wind_data = match simulation.copernicus.as_ref().unwrap().get_f64_values(dataset_id, vec!["eastward_wind".to_string(), "northward_wind".to_string()], timestamp, timestamp, longitude, longitude, latitude, latitude, None, None) {
            Ok(w) => w,
            Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Error getting wind data from copernicusmarine: {}", e))),
        };
        let wind_east_data = &wind_data[0];
        let wind_north_data = &wind_data[1];

        // Wind speed and direction
        let wind_east: f64 = wind_east_data[0].unwrap();
        let wind_north: f64 = wind_north_data[0].unwrap();
        let wind_angle: f64 = get_north_angle_from_northward_and_eastward_property(wind_east, wind_north);   // Angle in degrees
        let wind_speed = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>((wind_east*wind_east + wind_north*wind_north).sqrt().into());
        let wind = PhysVec::new(wind_speed.get::<uom::si::velocity::meter_per_second>(), wind_angle);    // unit [m/s]

        // Get ocean current data from Copernicus
        // "uo" is the eastward sea water velocity and "vo" is the northward sea water velocity
        let dataset_id: String = match copernicusmarine_rs::get_dataset_id(copernicusmarine_rs::CopernicusVariable::EastwardSeaWaterVelocity, timestamp, timestamp) {
            Ok(id) => id,
            Err(e) => panic!("Error getting dataset id from copernicusmarine: {}", e),
        };
        let ocean_current_data = match simulation.copernicus.as_ref().unwrap().get_f64_values(dataset_id, vec!["uo".to_string(), "vo".to_string()], timestamp, timestamp, longitude, longitude, latitude, latitude, Some(0.0), Some(1.0)){
            Ok(o) => o,
            Err(e) => panic!("Error getting ocean current data from copernicusmarine: {}", e),
        };
        let ocean_current_east_data = &ocean_current_data[0];
        let ocean_current_north_data = &ocean_current_data[1];

        // Ocean current speed and direction
        let ocean_current_east: f64 = match ocean_current_east_data[0] {
            Some(v) => v,
            None => 0.0,
        };
        let ocean_current_north: f64 = match ocean_current_north_data[0] {
            Some(v) => v,
            None => 0.0,
        };
        let ocean_current_angle: f64 = get_north_angle_from_northward_and_eastward_property(ocean_current_east, ocean_current_north);   // Angle in degrees
        let ocean_current_speed = uom::si::f64::Velocity::new::<uom::si::velocity::meter_per_second>((ocean_current_east*ocean_current_east + ocean_current_north*ocean_current_north).sqrt().into());
        let ocean_current = PhysVec::new(ocean_current_speed.get::<uom::si::velocity::meter_per_second>(), ocean_current_angle);    // unit [m/s]

        // Offset velocity by ocean current velocity (a.k.a. account for set and drift)
        let mut vessel_velocity_through_water: PhysVec = match vessel_velocity {
            Some(v) => v - ocean_current,
            None => {
                // If no vessel velocity, return error since no polar plot data can be generated
                return Err(io::Error::new(io::ErrorKind::Other, "No vessel velocity data in ship log entry, cannot generate polar plot data"));
            }
        };
        // Make sure the angle is between 0.0 and 360.0 degrees
        while vessel_velocity_through_water.angle < 0.0 {
            vessel_velocity_through_water.angle += 360.0;
        }
        while vessel_velocity_through_water.angle >= 360.0 {
            vessel_velocity_through_water.angle -= 360.0;
        }

        // Compute apparent wind
        let apparent_wind = wind - ocean_current;
        // Include heading
        let mut apparent_wind = PhysVec::new(apparent_wind.magnitude, apparent_wind.angle - heading.unwrap());
        // Make sure the angle is between 0.0 and 360.0 degrees
        while apparent_wind.angle < 0.0 {
            apparent_wind.angle += 360.0;
        }
        while apparent_wind.angle >= 360.0 {
            apparent_wind.angle -= 360.0;
        }

        // Log apparent wind angle, wind speed and vessel speed to polar plot data vector
        polar_plot_data_vector.push(vec![apparent_wind.angle, apparent_wind.magnitude, vessel_velocity_through_water.magnitude]);

        // Update progress bar if a progress bar is in use
        if !(simulation.progress_bar.is_none()) {
            // update progress bar
            simulation.progress_bar.as_ref().unwrap().inc(1);
            // If not interactive terminal, print progressbar manually
            if is_interactive_terminal == false {
                let eta = time::UtcDateTime::now().saturating_add(time::Duration::new(simulation.progress_bar.as_ref().unwrap().eta().as_secs() as i64, 0)); // What time the simulations will end
            println!("Elapsed: {} secs, Steps {}/{}, ETA: {}-{}-{} {}:{}:{}", simulation.progress_bar.as_ref().unwrap().elapsed().as_secs(), simulation.progress_bar.as_ref().unwrap().position(), simulation.progress_bar.as_ref().unwrap().length().unwrap(), eta.year(), eta.month() as u8, eta.day(), eta.hour(), eta.minute(), eta.second());
            }   // End if
        }   // End if
    }   // End for loop

    // Make mutable standardized polar plot data vector (based off of opencpn polar plot csv files but with wind in m/s). Include option for having None for unknown values
    // Column 0 is the angle of the apparent wind in degrees
    // column 1-40 contains tuples which are (n, VTW.magnitude) where n signifies how many values have been used to generate the average value VTW.magnitude which is the magnitude of the Velocity through water vector.
    // Note the VTW.magnitude is given in m/s in 1 m/s increments
    let mut standard_data_vector: Vec<Vec<(usize, Option<f64>)>> = Vec::new();

    // First find how many degree and wind speed segments we have
    let num_degree_segments: u16 = (180.0/working_degree_segment_size) as u16 + 1;   // +1 for the degrees since both 0° and 180° are included  // Keeping this even though it is currently unused since when this issue gets resolved we can use it again: https://github.com/G0rocks/marine_vessel_simulator/issues/56
    let num_wind_speed_segments: u8 = (40.0/working_wind_speed_segment_size) as u8;     // No +1 since 0 m/s is not included but 40 m/s is included
    
    // Fill the first column of the standard data vector with working_degree_segment_size increments, set all unknown values to None
    // Use 0..37 since OpenCPN polar plugin only accepts values in 5° increments
    for i in 0..37 {
        let angle = (i as f64) * 5.0;
        let mut sub_vec: Vec<(usize, Option<f64>)> = Vec::new();
        sub_vec.push((angle as usize, Some(angle)));
        for _k in 0..num_wind_speed_segments {
            sub_vec.push((0, None));
        }
        standard_data_vector.push(sub_vec);
    }

    // Now that all the data has been collected for the polar plot, we should loop through it and standardize it to be formatted in the similar numbers that the weather routing programs would use it
    for i in 0..polar_plot_data_vector.len() {
        // We only need one side of the polar plot, let's use the right side, everything else can be mirrored afterwards (effectively potentially doubles the available data)
        if polar_plot_data_vector[i][0] > 180.0 {
            polar_plot_data_vector[i][0] = 360.0 - polar_plot_data_vector[i][0];
        }

        // Find the nearest wind angle (in 5° increments) to this wind angle
        let nearest_angle_diff: f64 = polar_plot_data_vector[i][0] % working_degree_segment_size;
        let nearest_angle: f64;
        if nearest_angle_diff < working_degree_segment_size/2.0 {
            // Round down to the nearest working_degree_segment_size
            nearest_angle = polar_plot_data_vector[i][0] - nearest_angle_diff;
        } else {
            // Round up to the nearest working_degree_segment_size
            nearest_angle = polar_plot_data_vector[i][0] + (working_degree_segment_size - nearest_angle_diff);
        }

        // Find the row in the standard_data_vector that corresponds to this nearest angle. Rows given in 5° increments until OpenCPN polar plugin accepts other angles
        // Use this when the OpenCPN polar plugin accepts custom angles: let row: usize = (nearest_angle/working_degree_segment_size) as usize;
        let row: usize = (nearest_angle/5.0) as usize;

        // Find the nearest wind speed (in 1 m/s increments) to this wind speed
        let nearest_wind_speed_diff: f64 = polar_plot_data_vector[i][1] % working_wind_speed_segment_size;
        let mut nearest_wind_speed: f64;
        if nearest_wind_speed_diff < working_wind_speed_segment_size/2.0 {
            // Round down to the nearest working_wind_speed_segment_size
            nearest_wind_speed = polar_plot_data_vector[i][1] - nearest_wind_speed_diff;
        } else {
            // Round up to the nearest working_wind_speed_segment_size
            nearest_wind_speed = polar_plot_data_vector[i][1] + (working_wind_speed_segment_size - nearest_wind_speed_diff);
        }
        // If the nearest wind speed is set to zero then the column will be zero so we move the nearest wind speed up one segment size
        if nearest_wind_speed == 0.0 {
            nearest_wind_speed += working_wind_speed_segment_size;
        }

        // Find the row in the standard_data_vector that corresponds to this nearest angle
        let column: usize = (nearest_wind_speed/working_wind_speed_segment_size) as usize;

        // Imrpovement idea: If there are any values in the standard_data_vector in that index and the index that surrounds the current value, linearly interpolate the current value in the direction of the index
        // Let's average it directly and skip the linear interpolation for now, adding an issue about it
        if standard_data_vector[row][column].1.is_some() {
            // Get current number of values used to make the average
            let current_n: usize = standard_data_vector[row][column].0;
            // Get current average vessel speed
            let current_speed: f64 = standard_data_vector[row][column].1.unwrap();
            // Make new average vessel speed by adding the new value and incrementing the number of values used to make the average
            standard_data_vector[row][column] = (current_n + 1, Some((current_speed*(current_n as f64) + polar_plot_data_vector[i][2])/((current_n + 1) as f64)));
        }
        // Otherwise if this is the first value, assume it stays the same and put it directly into the standard_data_vector
        else {
            standard_data_vector[row][column] = (1, Some(polar_plot_data_vector[i][2]));
        }
    }

    // Now that the underlying polar plot data is ready, save the results to a csv file
    // Create a CSV writer with a semicolon delimiter
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(working_file_path)?;

    // Write the header
    let mut header_vec: Vec<String> = Vec::new();
    // First column in the header is "TWA\TWS" and not "wind angle [°]" because that is what openCPN uses
    header_vec.push("TWA\\TWS".to_string());
    for i in 1..(num_wind_speed_segments+1) {
        // If knots, format in knots
        if true_if_knots_false_if_meters_per_second {
            header_vec.push(format!("{}", (i as f64)*working_wind_speed_segment_size*1.94384));
        } // Otherwise use meters per second (preferred)
        else {
            header_vec.push(format!("{}", (i as f64)*working_wind_speed_segment_size));
        }
    }
    
    wtr.write_record(&header_vec)?;

    // Write the standard_data_vector into the csv file
    for row in standard_data_vector.iter() {
        // Init empty record to write
        let mut record: Vec<String> = Vec::new();
        // Add the wind angle to the first column
        // record.push(row[0].0);

        // For the rest of the row, if the boat speed is None, add an empty string, else, add the boat speed to the record
        // Note: The first cell should always be some as it should include the apparent wind angle
        for (i, cell) in row.iter().enumerate() {
            // If the value is Some, add it
            if cell.1.is_some() {
                // If it's the first column, just add it since it is the angle of the wind
                if i == 0 {
                    record.push(cell.1.unwrap().to_string());
                } // Otherwise, check if we're using knots or meters per second
                else {
                    // If knots, transform to knots
                    if true_if_knots_false_if_meters_per_second {
                        record.push((cell.1.unwrap() * 1.94384).to_string());
                    } // Otherwise, use meters_per_second
                    else {
                        record.push(cell.1.unwrap().to_string());
                    }
                }
            } // Otherwise, add empty string
            else {
                record.push(String::new());
            }
        }

        // Write the record
        wtr.write_record(&record)?;
    }

    // Flush and close the writer
    wtr.flush()?;

    // Finish progress bar
    simulation.progress_bar.as_ref().unwrap().finish();

    // Return data vector
    return Ok(polar_plot_data_vector);
}

/// Function that copies a csv file of ship logs taken from (aishub_data_collector)[https://crates.io/crates/aishub_data_collector]
/// and saves a copy of the ship log csv file formatted for marine_vessel_simulator
/// Note both the input and output filepaths must end with ".csv"
/// Note the initial coordinates default to (0.0, 0.0) degrees.
/// Note that this function assumes the aishub data is stored in the AIS encoding not the human readable format
/// More info on aishub api: https://www.aishub.net/api
/// The navigation status filter will set it so that the marine vessel simulator shiplog csv file will only contain the aishub data collector ship log entries which are logged under the same status. Set to None to use all values in the ship log.
/// 
/// If this function is no longer working since there is a new version of aishub_data_collector or similar, please submit an issue on the (marine_vessel_simulator issue tracker)[https://github.com/G0rocks/marine_vessel_simulator/issues]
/// Last updated 2026-02-01, it works with aishub_data_collector version 1.1.0
/// aishub_data_collector currently saves data into a csv file with the heading:
/// 
/// A,B,C,CALLSIGN,COG,D,DEST,DRAUGHT,DEVICE,ETA,HEADING,IMO,LATITUDE,LONGITUDE,MMSI,NAME,NAVSTAT,PAC,ROT,SOG,TSTAMP,TYPE
pub fn aishub_shiplog_csv_to_marine_vessel_simulator_shiplog_csv(filepath_input: &str, filepath_output: &str, navigation_status_filter: Option<NavigationStatus>) -> Result<Vec<ShipLogEntry>, io::Error> {
    // Check if filepath_input ends with ".csv", if not, return an invalid input error
    if !check_file_extension(filepath_input, ".csv") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Input file path must end with '.csv'"));
    }
    // Check if filepath_output ends with ".csv", if not, return an invalid input error
    if !check_file_extension(filepath_output, ".csv") {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Output file path must end with '.csv'"));
    }
    // Check if file is delimited with ';' and if not, return an error
    // To do this, get the second column in the first row delimited by ';' and if it does not exist then assume the file is not delimited with ';'
    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(filepath_input)
        .expect(format!("Failed to open file: {}\n", filepath_input).as_str());
    let first_entry = csv_reader.records().next().unwrap().unwrap();
    let column_2 = first_entry.get(1);
    if column_2.is_none() {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "Input file must be delimited with ';'"));
    }

    // Read the aishub ship log csv file into a Shiplog struct
    // Init ship logs vector
    let mut aishub_logs: Vec<ShipLogEntry> = Vec::new();

    // Read the CSV file
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(filepath_input)
        .expect(format!("Failed to open file: {}\n", filepath_input).as_str());

    // Since aishub does not provide information on cargo on board, set cargo on board to None
    let cargo_on_board = None;

    // Init which row to start from
    let mut entry_row_to_start = 0;
    let num_entries = csv_reader.records().count(); // Takes us to the last entry

    // Read the CSV file again to start from the beginning
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(filepath_input)
        .expect(format!("Failed to open file: {}\n", filepath_input).as_str());

    // Check if navigation status filter is in use
    if navigation_status_filter.is_some() {
        // Loop through the records of the aishub data collector shiplog until we find an entry which has the same navigation status
        while entry_row_to_start < num_entries {
            let result = csv_reader.records().next().expect("Could not get next result from csv file");
            match result {
                Ok(entry) => {
                    // Get navigation status
                    let navstat = match entry.get(16).unwrap().parse::<u8>() {
                        Ok(n) => Some(n),
                        Err(_) => None,
                    };
                    println!("Navigation status in entry {}: {:?}", entry_row_to_start, navstat.unwrap());
                    // If there is no navstat in the aishub shiplog then increment entry row to start and continue to next loop
                    if navstat == None {
                        entry_row_to_start += 1;
                        continue;
                    }
                    // If navigation status matches the filter then exit the loop
                    if navstat.unwrap() == navigation_status_filter.unwrap() as u8 {
                        println!("Heyy it's the same!! Breaking out of the loop");
                        // We found an entry with the same navigation status, break out of the loop and continue with the rest of the function
                        break;
                    }
                    // If the navigation status does not match the filter, increment the starting row 
                    entry_row_to_start += 1;
                },
                Err(e) => return Err(io::Error::new(io::ErrorKind::Other, format!("Error reading aishub_data_collector csv file: {}", e))),
            }
        }

        // If no entry with the same navigation status is found (entry row to start is the same as the number of rows), return an error which explains the situation
        if entry_row_to_start >= num_entries {
            return Err(io::Error::new(io::ErrorKind::Other, format!("Could not find any shiplog entry in aishub shiplog csv file with a navigation status: {:?}", navigation_status_filter.unwrap())));
        }
    }

    // Start new CSV file reader to start from the beginning
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(filepath_input)
        .expect(format!("Failed to open file: {}\n", filepath_input).as_str());


    // Get the initial and final coordinates
    // Init coordinates_initial
    let first_record = match csv_reader.records().nth(entry_row_to_start).expect("Could not get first entry from file") {
        Ok(r) => r,
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Error getting first record from aishub_data_collector file: {}", e)))?,
    };
    let latitude = match first_record.get(12).unwrap().parse::<f64>() {
        Ok(v) => v/600000.0,
        Err(e) => panic!("Error getting initial coordinates from aishub_data_collector csv file: {}", e),
    };
    let longitude = match first_record.get(13).unwrap().parse::<f64>() {
        Ok(v) => v/600000.0,
        Err(e) => panic!("Error getting initial coordinates from aishub_data_collector csv file: {}", e),
    };

    let coordinates_initial = geo::Point::new(longitude, latitude);

    // Get final coordinates
    let last_record =  match csv_reader.records().last().expect("Could not get last entry from csv file") {
        Ok(r) => r,
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Error getting last record from aishub_data_collector csv file: {}", e)))?,
    };
    let latitude = match last_record.get(12).unwrap().parse::<f64>() {
        Ok(v) => v/600000.0,
        Err(e) => panic!("Error getting final coordinates from aishub_data_collector csv file: {}", e),
    };
    let longitude = match last_record.get(13).unwrap().parse::<f64>() {
        Ok(v) => v/600000.0,
        Err(e) => panic!("Error getting final coordinates from aishub_data_collector csv file: {}", e),
    };
    let coordinates_final: geo::Point = geo::Point::new(longitude, latitude);

    // Start another csv reader stream since we already went through the whole stream.
    // This way we can start from the first record again
    let mut csv_reader = csv::ReaderBuilder::new()
        .delimiter(b';')
        .has_headers(true)
        .from_path(filepath_input)
        .expect(format!("Failed to open file: {}\n", filepath_input).as_str());

    // Move to the starting row if the entry row is not zero
    if entry_row_to_start != 0 {
        let _ = csv_reader.records().nth(entry_row_to_start - 1);
    }

    // Loop through all lines of file, append each line to the ship log
    for result in csv_reader.records() {
        match result {
            Ok(entry) => {
                // Get the entry simplified to reduce how verbose everything is
                // let entry = entry.get(0).unwrap().split(',');
                // Get all aishub_data_collector csv file fields
                // For each entry which aishub could signal is not available, check if the value is valid or not
                let _a = entry.get(0);
                let _b = entry.get(1);
                let _c = entry.get(2);
                let _callsign = entry.get(3);
                // If cog is 3600 the value is unknown
                let mut cog: Option<f64> = Some(entry.get(4).unwrap().parse::<f64>().unwrap());
                if cog.unwrap() == 3600.0 {
                    cog = None;
                }
                else {
                    cog = Some(cog.unwrap()/10.0);
                }
                let _d = entry.get(5);
                let _dest = entry.get(6);
                // Draft must be non-zero, if zero, report as None
                let mut draft = Some(0.0);
                if draft.unwrap() == 0.0 {
                    draft = None;
                }
                let _device = entry.get(8);
                let _eta = entry.get(9);
                // A heading of 511 means not available
                let mut heading: Option<f64> = Some(entry.get(10).expect("Could not get heading from file").parse::<f64>().unwrap());
                if heading.unwrap() == 511.0 {
                    heading = None;
                }
                let _imo = entry.get(11);
                // Aishub stores latitude and longitude data in 1/10000 minute
                let latitude = entry.get(12).unwrap().parse::<f64>().unwrap()/600000.0;
                let longitude = entry.get(13).unwrap().parse::<f64>().unwrap()/600000.0;
                let _mmsi = entry.get(14);
                let _name = entry.get(15);
                let navstat: Option<u8> = match entry.get(16) {
                    Some(n) => Some(n.parse::<u8>().unwrap()),
                    None => None
                };
                // If navigation status filter is in use, check if navstat matches navigation status filter, if not skip this entry
                if navigation_status_filter.is_some() {
                    // If the filter is in use and we do not know the navstat of this entry then skip this entry since we can not confirm it is the same as the filter
                    if navstat.is_none() {
                        continue;
                    }
                    // If we make it here then there is a valid navstat value
                    // Check if it matches the navigation status filter, if not then continue
                    if navstat.unwrap() != navigation_status_filter.unwrap() as u8 {
                        continue;
                    }
                }
                let _pac = entry.get(17);
                let _rot = entry.get(18);
                // If sog is 1024 the value is unknown
                let mut sog: Option<f64> = Some(entry.get(19).expect("Could not get sog from file").parse::<f64>().unwrap());
                if sog.unwrap() == 1024.0 {
                    sog = None;
                }
                else {
                    // sog is given in knots so in addition to dividing by 10.0 we must also convert to meters per second
                    sog = Some(sog.unwrap()/10.0/1.944);
                }
                let tstamp = entry.get(20).expect("Could not get tstamp from file").parse::<i64>().unwrap();
                let _vessel_type = entry.get(21);

                // Convert all aishub_data_collector fields into marine_vessel_simulator ShipLogEntry fields. Note aishub stores tstamp in unix time
                let timestamp: time::UtcDateTime = match time::UtcDateTime::from_unix_timestamp(tstamp) {
                    Ok(t) => t,
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Error converting tstamp to UtcDateTime: {}", e)))?,
                };

                let coordinates_current = geo::Point::new(longitude,latitude);
                
                // Init velocity
                let velocity: Option<PhysVec>;
                // If sog is unknown, set velocity to None
                if sog == None {
                    velocity = None;
                }   // Otherwise if cog is None, check if sog is zero
                else if cog == None {
                    // If sog is zero set sog and cog to zero in velocity since the direction does not matter
                    if sog.unwrap() == 0.0 {
                        velocity = Some(PhysVec::new(0.0, 0.0));
                    } // Otherwise set velocity to None since we don't know the direction
                    else {
                        velocity = None;
                    }
                } // Otherwise, we know both cog and sog
                else {
                    velocity = Some(PhysVec::new(sog.expect("Could not get sog when making velocity vector"), cog.expect("Could not get cog when making velocity vector")));
                }

                // Track angle is between last and current ship log entry, if this is the first entry, set to None
                let track_angle = match aishub_logs.len() {
                    0 => None,
                    _ => {
                        let last_entry: &ShipLogEntry = aishub_logs.last().unwrap();
                        let last_coords: geo::Point = last_entry.coordinates_current;
                        let curr_coords: geo::Point = coordinates_current;
                        Some(geo::Haversine.bearing(last_coords, curr_coords))
                    }
                };
                // Set true_bearing to angle between current location and final coordinates
                let true_bearing = Some(geo::Haversine.bearing(coordinates_current, coordinates_final));

                let navigation_status: Option<NavigationStatus> = match navstat {
                    Some(n) => match NavigationStatus::try_from(n) {
                                    Ok(status) => Some(status),
                                    Err(_) => None,
                                },
                    None => None
                };

                // Add ship log entry
                aishub_logs.push(
                    ShipLogEntry {
                        timestamp,
                        coordinates_initial,
                        coordinates_current,
                        coordinates_final,
                        cargo_on_board,
                        velocity,
                        course: cog,
                        heading,
                        track_angle,
                        true_bearing,
                        draft,
                        navigation_status,
                    });
                }
            Err(err) => {
                eprintln!("Error reading ship log entry from aishub_data_collector: {}", err);
            }
        }
    }

    // Write Shiplog to csv file
    ship_logs_to_csv(filepath_output, &aishub_logs)?;

    // Return success
    return Ok(aishub_logs);
}


/// Helper function that checks if a file extensions matches the given file extension.
/// A file called "mydata.csv" passed through this function with either ".csv" or "csv" will return true
pub fn check_file_extension(filepath: &str, extension: &str) -> bool {
    // Get num_chars in filepath and extension
    let num_chars_in_file = filepath.len();
    let num_chars_in_extension = extension.len();

    // Check if filepath has at least the same number of characters as the extension. If fewer, retun false
    if num_chars_in_file < num_chars_in_extension {
        return false;
    }

    // Check if the filepath string ends with the extension, if so return true, otherwise return false
    if &filepath[(num_chars_in_file - num_chars_in_extension)..] == extension {
        return true;
    } else {
        return false;
    }
}

// Set up tests here
//-----------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    // Test get_min_point_to_great_circle_dist function
    #[test]
    fn get_min_point_to_great_circle_dist_test() {
        let tolerance = 1.0;
        println!("Testing get_min_point_to_great_circle_dist function...");
        println!("Tolerance: {} meter", tolerance);
        println!("Earth radius: {} meters", geo::Haversine.radius());
        // First test short distance on both sides of equator and close to both end points
        let lon1 = 0.0;
        let lat1 = 0.0;
        let lon2 = 10.0;
        let lat2 = 0.0;
        let lon3 = 10.0;
        let lat3 = 10.0;
        let lon4 = 50.0;
        let lat4 = -10.0;
        let lon5 = 0.0;
        let lat5 = 0.000001;
        let lon6 = 10.0;
        let lat6 = 0.000001;
        let p1 = geo::Point::new(lon1, lat1);
        let p2 = geo::Point::new(lon2, lat2);
        let p3 = geo::Point::new(lon3, lat3);
        let p4 = geo::Point::new(lon4, lat4);
        let p5 = geo::Point::new(lon5, lat5);
        let p6 = geo::Point::new(lon6, lat6);
        let correct_dist = geo::Haversine.radius() * (lat3*2.0*std::f64::consts::PI/360.0); // 1111.950802335329128468111081452 kilometers
        let dist = get_min_point_to_great_circle_dist(p1, p2, p3);
        // Assert if dist is closer than the tolerance to the correct_dist
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);
        let dist = get_min_point_to_great_circle_dist(p1, p2, p4);
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);
        let correct_dist = geo::Haversine.radius() * (lat5*2.0*std::f64::consts::PI/360.0); // 1111.950802335329128468111081452 kilometers
        let dist = get_min_point_to_great_circle_dist(p1, p2, p5);
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);
        let dist = get_min_point_to_great_circle_dist(p1, p2, p6);
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);
        
        // Then test long distance from prime meridian
        let lon1 = 0.0;
        let lat1 = 89.0;
        let lon2 = 0.0;
        let lat2 = -89.0;
        let lon3 = 0.0;
        let lat3 = 0.0;
        let lon4 = -50.0;
        let lat4 = 0.0;
        let lon5 = 0.0;
        let lat5 = 89.000001;
        let lon6 = 0.0;
        let lat6 = -89.000001;
        let p1 = geo::Point::new(lon1, lat1);
        let p2 = geo::Point::new(lon2, lat2);
        let p3 = geo::Point::new(lon3, lat3);
        let p4 = geo::Point::new(lon4, lat4);
        let p5 = geo::Point::new(lon5, lat5);
        let p6 = geo::Point::new(lon6, lat6);
        // Assert if dist is closer than the tolerance to the correct_dist
        let correct_dist = 0.0;
        let dist = get_min_point_to_great_circle_dist(p1, p2, p3);
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);
        // Assert if dist is closer than the tolerance to the correct_dist
        let correct_dist = geo::Haversine.radius() * (lon4*2.0*std::f64::consts::PI/360.0).abs();
        let dist = get_min_point_to_great_circle_dist(p1, p2, p4);
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);

        let correct_dist = geo::Haversine.radius() * ((lat5-lat1)*2.0*std::f64::consts::PI/360.0).abs();
        let dist = get_min_point_to_great_circle_dist(p1, p2, p5);
        // Assert if dist is closer than the tolerance to the correct_dist
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);
        let correct_dist = geo::Haversine.radius() * ((lat6-lat2)*2.0*std::f64::consts::PI/360.0).abs();
        let dist = get_min_point_to_great_circle_dist(p1, p2, p6);
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);
        
        // Test for edge cases where p1 or p2 and p3 are the same
        let correct_dist = 0.0;
        let dist = get_min_point_to_great_circle_dist(p1, p2, p1);
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0);
        let dist = get_min_point_to_great_circle_dist(p1, p2, p2);
        assert_eq!((correct_dist-dist).abs() <= tolerance, true, "Correct distance: {:.2} km, calculated distance: {:.2} km", correct_dist/1000.0, dist/1000.0); 
    }
}