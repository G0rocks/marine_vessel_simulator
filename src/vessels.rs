/// Everything vessel related for the Marine vessel simulator that simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-05-29

use crate::*;   // To use everything from the crate

// Structs and Enums
//----------------------------------------------------
/// Struct to hold sailing leg data
/// p1: Start point of the leg
/// p2: End point of the leg
/// tacking_width: Width of the tacking zone around the leg line. The boat will try to stay within this zone when sailing the leg. The width will have the line between p1 and p2 in the middle of the tacking zone.
#[derive(Debug, Copy, Clone)]
pub struct SailingLeg {
    pub p1: geo::Point,
    pub p2: geo::Point,
    /// Tacking width in \[m\]
    pub tacking_width: f64,
    /// The minimum proximity in \[m\] to p2 to consider the vessel "at p2"
    pub min_proximity: f64
}

/// Struct to hold ship long entry
/// For every ship log you must know the time, where you started, where you are now and where you are going
/// Other fields are optional, but potentially useful for analysis later
#[derive(Debug)]
pub struct ShipLogEntry {
    pub timestamp: time::UtcDateTime,
    /// The initial coordinates of the voyage, not the leg
    pub coordinates_initial: geo::Point,
    /// The coordinates of the vessel at the time of the ShipLogEntry
    pub coordinates_current: geo::Point,
    /// The final coordinates of the voyage, not the leg
    pub coordinates_final: geo::Point,
    /// How much cargo is on board at the time of the log entry
    pub cargo_on_board: Option<uom::si::f64::Mass>,
    /// Current velocity of the boat
    pub velocity: Option<PhysVec>,
    /// Rhumb line course over from initial coordinates to final coordinates in degrees. North: 0°, East: 90°, South: 180°, West: 270°
    pub course: Option<f64>,
    /// Heading in degrees. North: 0°, East: 90°, South: 180°, West: 270°
    pub heading: Option<f64>,
    /// The angle, in degrees, from the last ShipLogEntry to the current location. North: 0°, East: 90°, South: 180°, West: 270°, if first entry, should be None
    pub track_angle: Option<f64>,
    /// True bearing from vessel to coordinates_final in degrees. North: 0°, East: 90°, South: 180°, West: 270°
    pub true_bearing: Option<f64>,
    /// draft of the boat at the time of the log entry in meters
    pub draft: Option<f64>,
    /// Navigation status of the boat at the time of the log entry
    pub navigation_status: Option<NavigationStatus>,
}

/// Navigational status of the vessel based on the AIS navigation status codes
/// See: <https://support.marinetraffic.com/en/articles/9552867-what-is-the-significance-of-the-ais-navigational-status-values>
#[derive(Debug, Copy, Clone)]
#[repr(u64)]
    pub enum NavigationStatus {
    UnderwayUsingEngine         = 0,
    AtAnchor                    = 1,
    NotUnderCommand             = 2,
    RestrictedManeuverability   = 3,
    ConstrainedByDraft          = 4,
    Moored                      = 5,
    Aground                     = 6,
    EngagedInFishing            = 7,
    UnderwaySailing             = 8,
}


impl TryFrom<u8> for NavigationStatus {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(NavigationStatus::UnderwayUsingEngine),
            1 => Ok(NavigationStatus::AtAnchor),
            2 => Ok(NavigationStatus::NotUnderCommand),
            3 => Ok(NavigationStatus::RestrictedManeuverability),
            4 => Ok(NavigationStatus::ConstrainedByDraft),
            5 => Ok(NavigationStatus::Moored),
            6 => Ok(NavigationStatus::Aground),
            7 => Ok(NavigationStatus::EngagedInFishing),
            8 => Ok(NavigationStatus::UnderwaySailing),
            _ => Err(()),
        }
    }
}

/// Struct to represent a sail
pub struct Sail {
    pub area: uom::si::f64::Area,       // Area of the sail
    pub current_angle_of_attack: f64,   // Current angle of attack in degrees. Angle between sails chordlength and the wind direction
    pub lift_coefficient: f64,          // Lift coefficient of the sail
    pub drag_coefficient: f64,          // Drag coefficient of the sail
}

impl Sail {
    pub fn new(area: uom::si::f64::Area, current_angle_of_attack: f64, lift_coefficient: f64, drag_coefficient: f64) -> Sail {
        Sail {
            area,
            current_angle_of_attack,
            lift_coefficient,
            drag_coefficient,
        }
    }
}

/// Struct to represent rudder
pub struct Rudder {
    /// Area of the rudder
    pub area: uom::si::f64::Area,
    /// Current angle of attack in degrees. Angle between rudders chordlength and the boats heading. 0° means rudder is aligned with the boat's heading. Negative values mean rudder is turned to port, positive values mean rudder is turned to starboard.
    pub current_angle_of_attack: f64,
    /// Lift coefficient of the rudder
    pub lift_coefficient: f64,
    /// Drag coefficient of the rudder
    pub drag_coefficient: f64,
}

impl Rudder {
    pub fn new(area: uom::si::f64::Area, current_angle_of_attack: f64, lift_coefficient: f64, drag_coefficient: f64) -> Rudder {
        Rudder {
            area,
            current_angle_of_attack,
            lift_coefficient,
            drag_coefficient,
        }
    }    
}

/// Enum to represent the side of the marine vessel
#[derive(PartialEq, Debug)]
pub enum VesselSide {
    Port,   // Left side of the boat when onboard and facing the bow
    Starboard, // Right side of the boat when onboard and facing the bow
}

impl VesselSide {
    /// Switches the vessel side to the other side
    pub fn switch(&mut self) {
        *self = match self {
            VesselSide::Port => VesselSide::Starboard,
            VesselSide::Starboard => VesselSide::Port,
        }
    }
}


/// Struct to hold boat metadata
/// All fields are optional, so that the struct can be created without knowing all the values
pub struct Boat {
    /// The vessels maximum cargo storage capacity (by weight)
    pub cargo_max_capacity: Option<uom::si::f64::Mass>,
    pub cargo_current: uom::si::f64::Mass,
    pub cargo_mean: Option<uom::si::f64::Mass>,
    pub cargo_std: Option<uom::si::f64::Mass>,
    pub current_leg: Option<u32>,
    pub destination: Option<geo::Point>,
    /// The draft (a.k.a draught) of the vessel in meters
    pub draft: Option<f64>,
    /// Heading in degrees. North: 0°, East: 90°, South: 180°, West: 270°
    pub heading: Option<f64>,
    /// Coefficient of drag for the hull
    pub hull_drag_coefficient: Option<f64>,
    /// The IMO number of the vessel
    pub imo: Option<u32>,
    /// The length of the vessel
    pub length: Option<uom::si::f64::Length>,
    pub location: Option<geo::Point>,
    /// Mass of the boat without cargo or fuel (a.k.a dry weight)
    pub mass: Option<uom::si::f64::Mass>,
    pub min_angle_of_attack: Option<f64>,
    /// The name of the vessel
    pub name: Option<String>,
    pub navigation_status: Option<NavigationStatus>,
    /// Note that for evaluating the route plan then the minimum proximity of the final point of the roue plan must be zero
    pub route_plan: Option<Vec<SailingLeg>>,
    pub rudder: Option<Rudder>,
    pub sail: Option<Sail>,
    pub ship_log: Vec<ShipLogEntry>,
    /// The current time for the boat
    pub time_now: time::UtcDateTime,
    /// The true bearing (true as in from north) to the next waypoint
    pub true_bearing: Option<f64>,
    /// Current velocity of the boat with magnitude and direction
    pub velocity_current: Option<PhysVec>,
    /// The average velocity of the boat, only magnitude, take care of your units. Good practice to use the same velocity units everywhere, \[m/s\] recommended.
    pub velocity_mean: Option<f64>,
    /// The standard deviation of the velocity of the boat, only magnitude
    pub velocity_std: Option<f64>,
    /// The width of the vessel
    pub width: Option<uom::si::f64::Length>,
    /// Preferred side of the boat for the wind to hit
    pub wind_preferred_side: VesselSide,
    /// Multiplier for the wind velocity to simulate different sail efficiencies. When simulating, the velocity of the vessel in the vessels heading will be the wind velocity multiplied by this multiplier. Note that ocean currents can also impact vessel velocity.
    pub wind_velocity_multiplier: Option<f64>,
}

// Implementation of the Boat struct
//----------------------------------------------------
impl Boat {
    /// Creates a new Boat instance with mostly None in the fields, though some fields have default values
    /// Make sure to set the values you need to use to the correct values 
    /// Defaults all to None except cargo_current to zero, ship_log to an empty vector, time_now to UtcDateTime::now(), wind_preferred_side to starboard since then we have the right of way in most cases.
    pub fn new() -> Boat {
        Boat {
            cargo_current: uom::si::f64::Mass::new::<uom::si::mass::ton>(0.0),
            cargo_max_capacity: None,
            cargo_mean: None,
            cargo_std: None,
            current_leg: None,
            destination: None,
            draft: None,
            heading: None,
            hull_drag_coefficient: None,
            imo: None,
            length: None,
            location: None,
            mass: None,
            min_angle_of_attack: None,
            name: None,
            navigation_status: None,
            route_plan: None,
            rudder: None,
            sail: None,
            ship_log: Vec::new(),
            time_now: UtcDateTime::now(),
            true_bearing: None,
            velocity_current: None,
            velocity_mean: None,
            velocity_std: None,
            width: None,
            wind_preferred_side: VesselSide::Starboard,
            wind_velocity_multiplier: None,
        }
    }

    /// Tacks the boat to the other side
    /// Switches the preferred wind side and sets the heading to the minimum angle of attack with respect to the wind angle and the new preferred wind side.
    pub fn tack(&mut self, wind_angle: f64) {
        // Switch preferred wind side
        self.wind_preferred_side.switch();
        self.hold_tack(wind_angle);
    }

    /// Keeps the heading of the boat based on the preferred wind side from the last tack.
    pub fn hold_tack(&mut self, wind_angle: f64) {
        // Set heading to the minimum angle of attack with respect to the wind angle 
        if self.wind_preferred_side == VesselSide::Port {
            // Wind on port side
            self.heading = Some(wind_angle + self.min_angle_of_attack.unwrap());
        } else if self.wind_preferred_side == VesselSide::Starboard {
            // Wind on starboard side
            self.heading = Some(wind_angle - self.min_angle_of_attack.unwrap());
        }   // If boat has no preferred wind side set, catch and set to starboard
        else {
            self.wind_preferred_side = VesselSide::Starboard; // Default to starboard since then we have the right of way in most cases
            self.heading = Some(wind_angle - self.min_angle_of_attack.unwrap());
        }
        // Make sure the heading is in between [0, 360]
        while self.heading.unwrap() < 0.0 {
            self.heading = Some(self.heading.unwrap() + 360.0);
        }
        while self.heading.unwrap() > 360.0 {
            self.heading = Some(self.heading.unwrap() - 360.0);
        }
    }

    /// Logs a new entry in the ship log
    pub fn log_entry_into_ship_log(&mut self) {
        // If there is a ship log entry already, use the last initial coordinates, otherwise, use boats current location
        let coord_initial = match self.ship_log.len() {
            0 => self.location.expect("Tried to get boats location but no location was found"),
            _ => self.ship_log.last().unwrap().coordinates_initial,
        };
        // Same with final coordinates unless the boat has a destination
        let coord_final: geo::Point;
        if self.destination.is_some() {
            coord_final = self.destination.unwrap()
        }
        else {
            coord_final = match self.ship_log.len() {
                0 => self.location.expect("Tried to get boats location but no location was found"),
                _ => self.ship_log.last().unwrap().coordinates_final,
            };
        }
        // Make the new entry
        let new_log_entry: ShipLogEntry = ShipLogEntry {
            timestamp: self.time_now,
            coordinates_initial: coord_initial,
            coordinates_current: self.location.expect("Tried to get boats location but no location was found"),
            coordinates_final: coord_final,
            cargo_on_board: Some(self.cargo_current),
            velocity: self.velocity_current,
            course: Some(geo::Haversine.bearing(coord_initial, coord_final)),
            track_angle: Some(Rhumb.bearing(coord_initial, self.location.unwrap())),
            heading: self.heading,
            true_bearing: self.true_bearing,
            draft: self.draft,
            navigation_status: self.navigation_status,
            };

        // Push the new log entry to the ship log
        self.ship_log.push(new_log_entry);
    }

    /// Loads cargo, makes sure to compare against the maximum cargo capacity of the vessel
    pub fn load_cargo(&mut self, cargo: uom::si::f64::Mass) {
        // Check if the cargo is too heavy
        match self.cargo_max_capacity {
            Some(max_capacity) => {
                if cargo > max_capacity {
                    // TODO: return error instead of panic
                    panic!("Cargo is too heavy");
                }
            }
            None => {}  // No max capacity set, so do nothing
        }

        // Set the cargo
        self.cargo_current = cargo;
    }
}



// Implementation of the ShipLogEntry struct
//----------------------------------------------------
impl ShipLogEntry {
    pub fn new(timestamp: UtcDateTime, coord_initial: geo::Point, coord_current: geo::Point, coord_final: geo::Point, cargo: Option<uom::si::f64::Mass>, velocity: Option<PhysVec>, course: Option<f64>, heading: Option<f64>, track_angle: Option<f64>, true_bearing: Option<f64>, draft: Option<f64>, navigation_status: Option<NavigationStatus>) -> ShipLogEntry {
        ShipLogEntry {
            timestamp: timestamp,
            coordinates_initial: coord_initial,
            coordinates_current: coord_current,
            coordinates_final: coord_final,
            cargo_on_board: cargo,
            velocity: velocity,
            course: course,
            heading: heading,
            track_angle: track_angle,
            true_bearing: true_bearing,
            draft: draft,
            navigation_status: navigation_status}
    }
}