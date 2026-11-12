use crate::{CarouselPhase, CarouselRide, PATH_UNITS, PathPoint, RideModel};

/// Integer circular samples shared by fixed-ride seating and presentation.
pub(crate) fn circle(progress: u32, duration: u32) -> (i32, i32) {
    const P: [(i32, i32); 16] = [
        (0, 1024),
        (392, 946),
        (724, 724),
        (946, 392),
        (1024, 0),
        (946, -392),
        (724, -724),
        (392, -946),
        (0, -1024),
        (-392, -946),
        (-724, -724),
        (-946, -392),
        (-1024, 0),
        (-946, 392),
        (-724, 724),
        (-392, 946),
    ];
    let scaled = u64::from(progress) * 16 * 1024 / u64::from(duration.max(1));
    let index = (scaled / 1024) as usize % 16;
    let t = (scaled % 1024) as i32;
    let a = P[index];
    let b = P[(index + 1) % 16];
    (a.0 + (b.0 - a.0) * t / 1024, a.1 + (b.1 - a.1) * t / 1024)
}
pub(crate) fn flat_seats(ride: &CarouselRide, cycle: u16) -> Option<Vec<PathPoint>> {
    let model = ride.ride.model?;

    let progress = if ride.phase == CarouselPhase::Running {
        u32::from(ride.progress)
    } else {
        0
    };
    let (sin, cos) = circle(progress, u32::from(cycle));
    let (width, depth) = model.dimensions();
    let cx = (width as i32 - 1) * PATH_UNITS / 2;
    let cy = (depth as i32 - 1) * PATH_UNITS / 2;
    let points = (0..model.flat_capacity())
        .map(|seat| {
            let seat = seat as i32;
            let (x, y, z) = match model {
                RideModel::Carousel => {
                    let (a, b) = circle(
                        progress + seat as u32 * u32::from(cycle) / 6,
                        u32::from(cycle),
                    );
                    (cx + a * PATH_UNITS / 1024, cy + b * PATH_UNITS / 1024, 35)
                }
                RideModel::Swing => {
                    let phase = progress * 64 / u32::from(cycle);
                    let travel = if phase < 16 {
                        phase as i32 * 3
                    } else if phase < 48 {
                        96 - phase as i32 * 3
                    } else {
                        phase as i32 * 3 - 192
                    };
                    (cx + travel, cy + seat * 17 - 43, 48 + travel.abs() / 3)
                }
                RideModel::DropTower => {
                    let phase = progress * 100 / u32::from(cycle);
                    let lift = if phase < 48 {
                        phase * 240 / 48
                    } else if phase < 58 {
                        240
                    } else if phase < 76 {
                        240 - (phase - 58) * 220 / 18
                    } else if phase < 90 {
                        20 - (phase - 76) * 20 / 14
                    } else {
                        0
                    };
                    (
                        cx + (seat / 3 * 2 - 1) * 9,
                        cy + (seat % 3 - 1) * 29,
                        35 + lift as i32,
                    )
                }

                RideModel::FerrisWheel => (
                    cx + sin * 78 / 1024 + (seat % 3 - 1) * 12,
                    cy + (seat / 3 * 2 - 1) * 12,
                    333 - cos * 312 / 1024,
                ),
                RideModel::TeaCups => {
                    let pod = seat / 2;
                    let (a, b) = circle(
                        progress + pod as u32 * u32::from(cycle) / 6,
                        u32::from(cycle),
                    );
                    let (spin_x, spin_y) = circle(
                        progress * 3 + seat as u32 * u32::from(cycle) / 2,
                        u32::from(cycle),
                    );
                    (
                        cx + a * 70 / 1024 + spin_x * 12 / 1024,
                        cy + b * 70 / 1024 + spin_y * 12 / 1024,
                        21,
                    )
                }
                RideModel::ObservationTower => {
                    let phase = progress * 100 / u32::from(cycle);
                    let rise = if phase < 35 {
                        phase * 510 / 35
                    } else if phase < 65 {
                        510
                    } else {
                        (100 - phase) * 510 / 35
                    };
                    let (a, b) = circle(seat as u32, 8);
                    (cx + a * 32 / 1024, cy + b * 32 / 1024, 21 + rise as i32)
                }
                RideModel::PirateShip => {
                    let (s, _) = circle(progress * 2, u32::from(cycle));
                    (
                        cx + (seat % 6 * 2 - 5) * 12,
                        cy + s * 45 / 1024 + (seat / 6 * 2 - 1) * 10,
                        21 + s.abs() * 150 / 1024,
                    )
                }
                _ => {
                    return PathPoint {
                        x: cx,
                        y: cy,
                        height: 21,
                    };
                }
            };
            let (x, y) = match ride.rotation % 4 {
                0 => (x, y),
                1 => ((depth as i32 - 1) * PATH_UNITS - y, x),
                2 => (
                    (width as i32 - 1) * PATH_UNITS - x,
                    (depth as i32 - 1) * PATH_UNITS - y,
                ),
                _ => (y, (width as i32 - 1) * PATH_UNITS - x),
            };
            PathPoint {
                x: ride.origin.x as i32 * PATH_UNITS + x,
                y: ride.origin.y as i32 * PATH_UNITS + y,
                height: i32::from(ride.height) * PATH_UNITS + z,
            }
        })
        .collect();
    Some(points)
}
