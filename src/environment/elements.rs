//! Environment element definitions - ASCII art for ground and background
#![allow(dead_code)] // Some elements are reserved for future use

/// Ground decoration characters for Beach style
pub const BEACH_CHARS: &[char] = &['.', ',', '~', '.', ',', '@', 'o', '.', ',', '~'];

/// Ground decoration characters for Garden style
pub const GARDEN_CHARS: &[char] = &['.', ',', '\'', '`', '"', '.', ',', '*', '^', '.'];

/// Ground decoration characters for Rocky style
pub const ROCKY_CHARS: &[char] = &['.', 'o', '.', 'O', '.', '^', '.', 'o', '.', '.'];

/// Ground decoration characters for Minimal style
pub const MINIMAL_CHAR: char = '-';

/// Background element: Sun (for daytime)
pub const SUN: &[&str] = &[r"  \*/  ", r"-- O --", r"  /*\  "];

/// Small moon for smaller terminals
pub const MOON_SMALL: &[&str] = &[r"  ,-,", r" /.(", r" \ {", r"  `-`"];

/// Background element: Cloud (small)
pub const CLOUD_SMALL: &[&str] = &[r"  .--.  ", r" (    ) ", r"  `--'  "];

/// Background element: Cloud (large)  
pub const CLOUD_LARGE: &[&str] = &[
    r"   .---.   ",
    r"  (     )  ",
    r" (       ) ",
    r"  `-----'  ",
];

/// Star characters for nighttime
pub const STAR_CHARS: &[char] = &['*', '+', '.', '*', '.', '+', '*'];

/// Interactive object: Small rock
pub const ROCK_SMALL: &[&str] = &[r" /\ ", r"/  \"];

/// Interactive object: Large rock
pub const ROCK_LARGE: &[&str] = &[r"  /\  ", r" /  \ ", r"/____\"];

/// Interactive object: Plant/Seaweed
pub const PLANT: &[&str] = &[r" ) ", r"(  ", r" ) ", r"/| "];

/// Interactive object: Small plant
pub const PLANT_SMALL: &[&str] = &[r" \|/ ", r"  |  "];

/// Interactive object: Shell (beach theme)
pub const SHELL: &[&str] = &[r"@"];

/// Interactive object: Flower (garden theme)
pub const FLOWER: &[&str] = &[r"*", r"|"];
