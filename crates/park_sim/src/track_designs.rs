//! Portable named track geometry, independent of parks, ride identities and money.

use crate::{
    MAX_MAP_SIZE, MAX_TRACK_DRAFT_PIECES, TilePos, TrackDirection, TrackKind, TrackPieceSpec,
};

pub const TRACK_DESIGN_VERSION: u32 = 1;
pub const MAX_TRACK_DESIGN_NAME_CHARS: usize = 32;
pub const MAX_TRACK_DESIGN_BYTES: usize = 128 * 1024;
const FORMAT: &str = "parkbound-track-design";

/// Fields remain private so all designs pass the same bounded codec and geometry rules.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrackDesign {
    name: String,
    pieces: Vec<TrackPieceSpec>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TrackDesignError {
    EmptyName,
    NameTooLong,
    NameContainsControl,
    EmptyDesign,
    TooLarge { actual: usize, max: usize },
    Json(String),
    InvalidFormat,
    UnsupportedVersion(u32),
    Geometry(crate::TrackConstructionError),
    PositionOutOfBounds,
    HeightOutOfBounds,
}

impl std::fmt::Display for TrackDesignError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyName => f.write_str("enter a name for this design"),
            Self::NameTooLong => write!(
                f,
                "design names support at most {MAX_TRACK_DESIGN_NAME_CHARS} characters"
            ),
            Self::NameContainsControl => {
                f.write_str("design names cannot contain control characters")
            }
            Self::EmptyDesign => f.write_str("add track pieces before saving a design"),
            Self::TooLarge { actual, max } => {
                write!(f, "track design has {actual} bytes; maximum is {max}")
            }
            Self::Json(message) => write!(f, "invalid track design JSON: {message}"),
            Self::InvalidFormat => f.write_str("this file is not a Parkbound track design"),
            Self::UnsupportedVersion(version) => {
                write!(f, "unsupported track design version {version}")
            }
            Self::Geometry(error) => error.fmt(f),
            Self::PositionOutOfBounds => {
                f.write_str("the design extends beyond the supported map bounds")
            }
            Self::HeightOutOfBounds => {
                f.write_str("the design extends beyond supported track heights")
            }
        }
    }
}
impl std::error::Error for TrackDesignError {}
impl From<crate::TrackConstructionError> for TrackDesignError {
    fn from(error: crate::TrackConstructionError) -> Self {
        Self::Geometry(error)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DesignEnvelope {
    format: String,
    version: u32,
    name: String,
    #[serde(deserialize_with = "crate::saves::bounded_objects::<_, _, MAX_TRACK_DRAFT_PIECES>")]
    pieces: Vec<TrackPieceSpec>,
}

fn json_error(error: serde_json::Error) -> TrackDesignError {
    TrackDesignError::Json(error.to_string().chars().take(512).collect())
}

impl TrackDesign {
    /// Keep exact piece order and grades. Completion and clearance belong to Apply.
    pub fn from_draft(name: &str, pieces: &[TrackPieceSpec]) -> Result<Self, TrackDesignError> {
        if name.chars().any(char::is_control) {
            return Err(TrackDesignError::NameContainsControl);
        }
        let name = name.trim();
        if name.is_empty() {
            return Err(TrackDesignError::EmptyName);
        }
        if name.chars().count() > MAX_TRACK_DESIGN_NAME_CHARS {
            return Err(TrackDesignError::NameTooLong);
        }
        if pieces.is_empty() {
            return Err(TrackDesignError::EmptyDesign);
        }
        crate::track_construction::validate_draft_structure_in_bounds(
            MAX_MAP_SIZE,
            MAX_MAP_SIZE,
            pieces,
        )?;
        Ok(Self {
            name: name.to_owned(),
            pieces: pieces.to_vec(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn pieces(&self) -> &[TrackPieceSpec] {
        &self.pieces
    }

    /// Placement is relative to the first station, or first piece of an unfinished design.
    pub fn anchor(&self) -> (TilePos, u8) {
        let piece = self
            .pieces
            .iter()
            .find(|piece| piece.kind == TrackKind::Station)
            .unwrap_or(&self.pieces[0]);
        (piece.pos, piece.entry.height)
    }

    pub fn to_json(&self) -> Result<String, TrackDesignError> {
        let json = serde_json::to_string_pretty(&DesignEnvelope {
            format: FORMAT.to_owned(),
            version: TRACK_DESIGN_VERSION,
            name: self.name.clone(),
            pieces: self.pieces.clone(),
        })
        .map_err(json_error)?;
        check_size(json.len())?;
        Ok(json)
    }

    pub fn from_json(json: &str) -> Result<Self, TrackDesignError> {
        check_size(json.len())?;
        let mut deserializer = serde_json::Deserializer::from_str(json);
        let envelope: DesignEnvelope =
            crate::saves::strict_object(&mut deserializer).map_err(json_error)?;
        deserializer.end().map_err(json_error)?;
        if envelope.format != FORMAT {
            return Err(TrackDesignError::InvalidFormat);
        }
        if envelope.version != TRACK_DESIGN_VERSION {
            return Err(TrackDesignError::UnsupportedVersion(envelope.version));
        }
        Self::from_draft(&envelope.name, &envelope.pieces)
    }

    /// Rotate clockwise in world tile coordinates, then translate horizontally and vertically.
    /// No clamping occurs: any out-of-bounds endpoint rejects the complete placement.
    /// The receiving park checks its own (possibly smaller) dimensions on draft replacement.
    pub fn transformed(
        &self,
        anchor: TilePos,
        height: u8,
        quarter_turns: u8,
    ) -> Result<Vec<TrackPieceSpec>, TrackDesignError> {
        let (source, source_height) = self.anchor();
        let turns = quarter_turns % 4;
        let height_delta = i16::from(height) - i16::from(source_height);
        let translate_height = |old: u8| {
            let height = i16::from(old) + height_delta;
            if !(0..=i16::from(crate::MAX_TERRAIN_HEIGHT - 2)).contains(&height) {
                return Err(TrackDesignError::HeightOutOfBounds);
            }
            Ok(height as u8)
        };
        self.pieces
            .iter()
            .map(|piece| {
                let dx = i64::from(piece.pos.x) - i64::from(source.x);
                let dy = i64::from(piece.pos.y) - i64::from(source.y);
                let (dx, dy) = match turns {
                    0 => (dx, dy),
                    1 => (-dy, dx),
                    2 => (-dx, -dy),
                    _ => (dy, -dx),
                };
                let x = i64::from(anchor.x) + dx;
                let y = i64::from(anchor.y) + dy;
                if !(0..i64::from(MAX_MAP_SIZE)).contains(&x)
                    || !(0..i64::from(MAX_MAP_SIZE)).contains(&y)
                {
                    return Err(TrackDesignError::PositionOutOfBounds);
                }
                Ok(TrackPieceSpec {
                    pos: TilePos {
                        x: x as u32,
                        y: y as u32,
                    },
                    kind: piece.kind,
                    entry: crate::TrackPort {
                        edge: rotate_edge(piece.entry.edge, turns),
                        height: translate_height(piece.entry.height)?,
                    },
                    exit: crate::TrackPort {
                        edge: rotate_edge(piece.exit.edge, turns),
                        height: translate_height(piece.exit.height)?,
                    },
                })
            })
            .collect()
    }
}

fn check_size(actual: usize) -> Result<(), TrackDesignError> {
    if actual > MAX_TRACK_DESIGN_BYTES {
        return Err(TrackDesignError::TooLarge {
            actual,
            max: MAX_TRACK_DESIGN_BYTES,
        });
    }
    Ok(())
}

fn rotate_edge(edge: TrackDirection, turns: u8) -> TrackDirection {
    use TrackDirection::{East, North, South, West};
    let index = match edge {
        North => 0,
        East => 1,
        South => 2,
        West => 3,
    };
    [North, East, South, West][(index + usize::from(turns)) % 4]
}

#[cfg(test)]
mod tests;
