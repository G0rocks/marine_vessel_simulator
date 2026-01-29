# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

-

### Changed

- 

### Removed

- 


## [1.0.0] - 2026-01-29

### Added

- This changelog
- load_route_plan() function
- ShipLogEntry struct
- ship_logs_to_csv() function
- csv_to_ship_logs() function
- sim_waypoint_missions() function (a COOL thing) which simulates multiple waypoint missions
- Plotting capacity through plotly (visualize_ship_logs() function) aaaand the plot is RESPONSIVE LET'S GOOO!!!! Had to add this capacity to plotly for it to work here, am proud.
- Simulator struct to replace the Boat.simulator_method field 
- Sail and Rudder structs (currently unused) and a PhysVec (Physics vector) struct (named Wind for a while) which is very used.
- min_haversine_distance() function
- Wind and current now impacts simulations of sail propelled vessels
- Get weather from copernicusmarine_rs crate (new dependency)
- Progress bar capacity from indicatif (new dependency). Works for both interactive and non-interactive terminals. For good ETA, currently waiting on a merge for this PR (https://github.com/console-rs/indicatif/pull/722) but until then it is possible to use the G0rocks fork of indicatif, here: https://github.com/G0rocks/indicatif
- A lot of error messages
- Markdown to the documentation
- Possibility of running a simplified simulation where you download the weather data preemptively for the most direct route and assume the weather stays the same, takes a bit of time to prep but you can run many simulations very fast after doing the prep. Though probably less reliable, have not measured.
- wind_velocity_multiplier to Boat struct
- make_polar_speed_plot_csv() function which generates the underlying data for a polar plot using the ship logs.

### Changed

- evaluate_cargo_shipping_logs() returns mean and std for travel time
- time things now rely on the standard library and the time crate (UtcDateTime)
- get_distance_mean_and_std() for uom distances
- Simulators and related code have a specific file for them in the simulators.rs file
- Vessel related things (the Boat struct and other things) have been moved to a dedicated vessels.rs file
- Improved a lot of error messages
- Improved a lot of the documentation
- "draught" --> "draft"
- Boat.heading is now always in [0, 360]

### Removed

- plotters dependency
- main.rs (since this is a library)
- Some bugs
- Boat.simulator_method field has been removed and replaced with the Simulator struct

## [0.1.1] - 2025-06-14
Second release, now the crate has some simulative capabilities! Woohoo!

### Added

- Changelog
- Basic simulative capabilities
- [Copernicus_rs][https://crates.io/crates/copernicusmarine_rs] so that it is possible to use historical weather data through the [copernicus service][https://www.copernicus.eu/en]
- Some more structs and enums and helper functions that make things easier to think about.

### Fixed

- Nothing

### Changed

- Split codebase into lib.rs, simulators.rs and vessels.rs to organize better what is where
- Changed some names of variables, it is nothing groundbreaking, if you're using the code, it'll be easy for you to fix it
- The timestamp system to use the [time crate][https://crates.io/crates/time] instead since there really was no reason to reinvent the wheel
- Reduced the usage of the uom crate since it is quite cumbersome to write it


### Removed

- main.rs since it was not being used
- plotters has been commented out, since plotly is easier to use and has more capabilities


## [0.1.0] - 2025-04-30
The first release of package with some structs and enums and functions but not much simulative capability. Can basically only evaluate shipping logs.
Glad to be here :)
Looking forward to having some more capabilities.

### Added

- lib.rs
- main.rs

### Fixed

- Nothing was fixed

### Changed

- Nothing was changed

### Removed

- Nothing was removed


## List of releases
[unreleased]: https://github.com/G0rocks/marine_vessel_simulator/compare/v0.1.0...main
[0.1.1]: https://github.com/G0rocks/marine_vessel_simulator/releases/tag/v0.1.1
[0.1.0]: https://github.com/G0rocks/marine_vessel_simulator/releases/tag/v0.1.0
