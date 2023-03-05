# play-files
A library for parsing Polyend Play projects into Rust structs.

[![Crates.io](https://img.shields.io/crates/v/play-files)](https://crates.io/crates/play-files)

## Usage

Add to your `Cargo.toml`:
```
play-files = "0.1"
```
Or
```
$ cargo add play-files
```

## TODO
Substantial:
- Parse `samplesMetadata`
- Lots of unknown `settings`
- Make proper enums for the enum values (e.g. Chance)
- Write files

Smaller:
- Unknown footer for Steps
- Unknown footer attrs for Tracks
- Unknown footer for Projects
