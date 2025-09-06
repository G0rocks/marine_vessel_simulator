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
    /// Tacking width in [m]
    pub tacking_width: f64,
    /// The minimum proximity in [m] to p2 to consider the vessel "at p2"
    pub min_proximity: f64
}

/// Struct to hold ship long entry
/// For every ship log you must know the time, where you started, where you are now and where you are going
/// Other fields are optional, but potentially useful for analysis later
#[derive(Debug)]
pub struct ShipLogEntry {
    pub timestamp: time::UtcDateTime,
    pub coordinates_initial: geo::Point,
    pub coordinates_current: geo::Point,
    pub coordinates_final: geo::Point,
    pub cargo_on_board: Option<uom::si::f64::Mass>,
    pub velocity: Option<PhysVec>,  // Current velocity of the boat
    pub course: Option<f64>,  // Rhumb line course over from initial coordinates to final coordinates in degrees. North: 0°, East: 90°, South: 180°, West: 270°
    pub heading: Option<f64>,  // Heading in degrees. North: 0°, East: 90°, South: 180°, West: 270°
    pub track_angle: Option<f64>,   // The angle, in degrees, from the last ShipLogEntry to the current location. North: 0°, East: 90°, South: 180°, West: 270°
    pub true_bearing: Option<f64>,  // True bearing from vessel to coordinates_final in degrees. North: 0°, East: 90°, South: 180°, West: 270°
    pub draft: Option<uom::si::f64::Length>,  // draft of the boat at the time of the log entry
    pub navigation_status: Option<NavigationStatus>,  // Navigation status of the boat at the time of the log entry
}

/// Navigational status of the vessel based on the AIS navigation status codes
/// See: https://support.marinetraffic.com/en/articles/9552867-what-is-the-significance-of-the-ais-navigational-status-values
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
    pub cargo_max_capacity: Option<uom::si::f64::Mass>,
    pub cargo_current: uom::si::f64::Mass,
    pub cargo_mean: Option<uom::si::f64::Mass>,
    pub cargo_std: Option<uom::si::f64::Mass>,
    pub imo: Option<u32>,
    pub min_angle_of_attack: Option<f64>,
    pub name: Option<String>,
    pub navigation_status: Option<NavigationStatus>,
    pub location: Option<geo::Point>,
    pub heading: Option<f64>,   /// Heading in degrees. North: 0°, East: 90°, South: 180°, West: 270°
    pub sail: Option<Sail>,
    pub rudder: Option<Rudder>,
    pub route_plan: Option<Vec<SailingLeg>>,
    pub current_leg: Option<u32>,
    pub length: Option<uom::si::f64::Length>,
    pub width: Option<uom::si::f64::Length>,
    pub draft: Option<uom::si::f64::Length>,
    pub mass: Option<uom::si::f64::Mass>,   /// Mass of the boat without cargo or fuel (a.k.a dry weight)
    pub velocity_current: Option<PhysVec>,  /// Current velocity of the boat with magnitude and direction
    pub velocity_mean: Option<uom::si::f64::Velocity>,  /// The average velocity of the boat, only magnitude
    pub velocity_std: Option<uom::si::f64::Velocity>,   /// The standard deviation of the velocity of the boat, only magnitude
    pub wind_preferred_side: VesselSide,  /// Preferred side of the boat for the wind to hit
    pub hull_drag_coefficient: Option<f64>,  /// Coefficient of drag for the hull
    pub ship_log: Vec<ShipLogEntry>,
    /// The current time for the boat
    pub time_now: time::UtcDateTime,
    /// The true bearing (true as in from north) to the next waypoint
    pub true_bearing: Option<f64>,
}

// Implementation of the Boat struct
//----------------------------------------------------
impl Boat {
    /// Creates a new Boat instance with mostly None in the fields, though some fields have default values
    /// Make sure to set the values you need to use to the correct values 
    pub fn new() -> Boat {
        Boat {
            imo: None,
            name: None,
            navigation_status: None,
            min_angle_of_attack: None,
            location: None,
            heading: None,
            sail: None,
            rudder: None,
            route_plan: None,
            current_leg: None,
            length: None,
            width: None,
            draft: None,
            mass: None,
            velocity_current: None,
            velocity_mean: None,
            velocity_std: None,
            cargo_max_capacity: None,
            cargo_current: uom::si::f64::Mass::new::<uom::si::mass::ton>(0.0),
            cargo_mean: None,
            cargo_std: None,
            wind_preferred_side: VesselSide::Starboard,  // Default to starboard since then we have the right of way in most cases
            hull_drag_coefficient: None,
            ship_log: Vec::new(),
            time_now: UtcDateTime::now(),
            true_bearing: None,
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



