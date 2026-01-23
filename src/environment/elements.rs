//! Environment element definitions - ASCII art for ground and background
#![allow(dead_code)] // Some elements are reserved for future use

/// Ground decoration chunks for Beach (coastal) style
pub const BEACH_CHUNKS: &[&str] = &[
    "....", "..", "..", "..", ".,", ",.", "::", "~~", "~~", "o.", ".o", "o", "@", "''",
];

/// Ground decoration chunks for Garden (forest floor) style
pub const GARDEN_CHUNKS: &[&str] = &[
    "....", "..", "..", "..", ".,", "`.", "''", "^^", "^^", "^^", "**", "++", "vv", "()",
];

/// Ground decoration chunks for Rocky (riverbed) style
pub const ROCKY_CHUNKS: &[&str] = &[
    "....", "..", "..", ".o", "o.", "oo", "O.", ".O", "O", "::", "~~", "~~",
];

/// Ground decoration chunks for Minimal (meadow) style
pub const MINIMAL_CHUNKS: &[&str] = &["....", "...", "..", "..", "..", "..", ".,", "^^", "''"];

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
