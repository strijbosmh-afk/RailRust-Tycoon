//! Original importable package content, using the existing version-1 codec.
use park_sim::{TrackDesign, TrackDesignError};

pub(crate) const FILES: [(&str, &str); 3] = [
    (
        "pocket-oval.parktrack.json",
        include_str!("../../../designs/pocket-oval.parktrack.json"),
    ),
    (
        "meadow-winder.parktrack.json",
        include_str!("../../../designs/meadow-winder.parktrack.json"),
    ),
    (
        "meadow-climb.parktrack.json",
        include_str!("../../../designs/meadow-climb.parktrack.json"),
    ),
];

pub(crate) fn designs() -> Result<Vec<TrackDesign>, TrackDesignError> {
    FILES
        .iter()
        .map(|(_, json)| TrackDesign::from_json(json))
        .collect()
}
