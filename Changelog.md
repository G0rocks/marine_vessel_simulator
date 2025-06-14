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

- main.rs since it was not


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