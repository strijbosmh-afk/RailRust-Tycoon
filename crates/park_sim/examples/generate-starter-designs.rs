//! Regenerate original package content through the production design codec.
use park_sim::{Park, TilePos, TrackBuildKind, TrackDesign, TrackDirection, TrackPieceSpec};
use std::{error::Error, fs::OpenOptions, io::Write, path::PathBuf};

fn pieces(data: &[(u32, u32, TrackBuildKind, TrackDirection, u8)]) -> Vec<TrackPieceSpec> {
    data.iter()
        .map(|&(x, y, kind, direction, height)| kind.spec(TilePos { x, y }, height, direction))
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    use TrackBuildKind::*;
    use TrackDirection::*;
    let output = PathBuf::from(
        std::env::args_os()
            .nth(1)
            .ok_or("Provide a new absolute output directory")?,
    );
    if !output.is_absolute() || output.exists() {
        return Err("Use a new absolute directory; existing content is never overwritten".into());
    }
    let designs = [
        (
            "pocket-oval.parktrack.json",
            "Pocket Oval",
            pieces(&[
                (8, 10, Station, East, 8),
                (9, 10, Straight, East, 8),
                (10, 10, Straight, East, 8),
                (11, 10, CurveLeft, East, 8),
                (11, 9, Straight, North, 8),
                (11, 8, Straight, North, 8),
                (11, 7, Straight, North, 8),
                (11, 6, CurveLeft, North, 8),
                (10, 6, Straight, West, 8),
                (9, 6, Straight, West, 8),
                (8, 6, Straight, West, 8),
                (7, 6, CurveLeft, West, 8),
                (7, 7, Straight, South, 8),
                (7, 8, Straight, South, 8),
                (7, 9, Straight, South, 8),
                (7, 10, CurveLeft, South, 8),
            ]),
        ),
        (
            "meadow-winder.parktrack.json",
            "Meadow Winder",
            pieces(&[
                (8, 10, Station, East, 8),
                (9, 10, Straight, East, 8),
                (10, 10, CurveLeft, East, 8),
                (10, 9, Straight, North, 8),
                (10, 8, CurveRight, North, 8),
                (11, 8, Straight, East, 8),
                (12, 8, CurveLeft, East, 8),
                (12, 7, Straight, North, 8),
                (12, 6, CurveLeft, North, 8),
                (11, 6, Straight, West, 8),
                (10, 6, Straight, West, 8),
                (9, 6, Straight, West, 8),
                (8, 6, Straight, West, 8),
                (7, 6, CurveLeft, West, 8),
                (7, 7, Straight, South, 8),
                (7, 8, Straight, South, 8),
                (7, 9, Straight, South, 8),
                (7, 10, CurveLeft, South, 8),
            ]),
        ),
        (
            "meadow-climb.parktrack.json",
            "Meadow Climb",
            pieces(&[
                (8, 10, Station, East, 8),
                (9, 10, Lift, East, 8),
                (10, 10, Lift, East, 9),
                (11, 10, Lift, East, 10),
                (12, 10, Straight, East, 11),
                (13, 10, Straight, East, 11),
                (14, 10, CurveLeft, East, 11),
                (14, 9, Straight, North, 11),
                (14, 8, Straight, North, 11),
                (14, 7, Straight, North, 11),
                (14, 6, Straight, North, 11),
                (14, 5, CurveLeft, North, 11),
                (13, 5, SteepDrop, West, 11),
                (12, 5, Drop, West, 9),
                (11, 5, Straight, West, 8),
                (10, 5, Straight, West, 8),
                (9, 5, Straight, West, 8),
                (8, 5, Straight, West, 8),
                (7, 5, CurveLeft, West, 8),
                (7, 6, Straight, South, 8),
                (7, 7, Straight, South, 8),
                (7, 8, Straight, South, 8),
                (7, 9, Brake, South, 8),
                (7, 10, CurveLeft, South, 8),
            ]),
        ),
    ];
    // Prove every complete transformed loop before creating any output files.
    for (_, name, specs) in &designs {
        let design = TrackDesign::from_draft(name, specs)?;
        for turn in 0..4 {
            let mut park = Park::new(32, 32);
            park.replace_track_draft(&design.transformed(TilePos { x: 16, y: 16 }, 8, turn)?)?;
            park.apply_track_draft()?;
            park.validate_train_circuit()?;
        }
    }
    std::fs::create_dir(&output)?;
    for (file, name, specs) in designs {
        let design = TrackDesign::from_draft(name, &specs)?;
        let json = design.to_json()?;
        let mut park = Park::new(32, 32);
        park.replace_track_draft(&specs)?;
        println!(
            "{name}: {} pieces / {:?}",
            specs.len(),
            park.track_construction_quote()?
        );
        OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output.join(file))?
            .write_all(json.as_bytes())?;
    }
    Ok(())
}
