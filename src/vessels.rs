/// Everything vessel related for the Marine vessel simulator that simulates the behaviour of marine vessels out at sea.
/// Author: G0rocks
/// Date: 2025-05-29

use crate::*;   // To use everything from the crate

// Structs and Enums
//----------------------------------------------------
/// Struct to hold sailing leg data
#[derive(Debug, Copy, Clone)]
pub struct SailingLeg {
    pub p1: geo::Point,
    pub p2: geo::Point,
    pub tacking_width: uom::si::f64::Length,
}

/// Struct to hold ship long entry
#[derive(Debug)]
pub struct ShipLogEntry {
    pub timestamp: time::UtcDateTime,
    pub coordinates_initial: geo::Point,
    pub coordinates_current: geo::Point,
    pub coordinates_final: geo::Point,
    pub cargo_on_board: uom::si::f64::Mass,
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
    pub imo: Option<u32>,
    pub name: Option<String>,
    pub min_angle_of_attack: Option<f64>,
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
    pub velocity_current: Option<uom::si::f64::Velocity>,  /// Current velocity of the boat
    pub velocity_mean: Option<uom::si::f64::Velocity>,
    pub velocity_std: Option<uom::si::f64::Velocity>,
    pub cargo_max_capacity: Option<uom::si::f64::Mass>,
    pub cargo_current: uom::si::f64::Mass,
    pub cargo_mean: Option<uom::si::f64::Mass>,
    pub cargo_std: Option<uom::si::f64::Mass>,
    pub wind_preferred_side: VesselSide,  // Preferred side for the wind to come from, if any
    pub hull_drag_coefficient: Option<f64>,  /// Coefficient of drag for the hull
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
        }
    }

    /// Tacks the boat to the other side
    /// Switches the preferred wind side and sets the heading to the minimum angle of attack with respect to the wind angle and the new preferred wind side.
    pub fn tack(&mut self, wind_angle: f64) {
        // Print debug message
        println!("Tacking the boat to the other side\n Current location {:?}", self.location);
        // Switch preferred wind side
        self.wind_preferred_side.switch();
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



