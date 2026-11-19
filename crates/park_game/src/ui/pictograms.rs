//! Original, antialiased line artwork shared by every menu button.
use super::Action;
use crate::{path_tools::PathTool, track_tools::TrackCommand};
use bevy::{
    asset::RenderAssetUsages,
    image::ImageSampler,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
use park_sim::{SceneryKind, TrackBuildKind};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub(super) enum Glyph {
    Menu,
    Path,
    Queue,
    Gate,
    Exit,
    Search,
    Hammer,
    Raise,
    Lower,
    Water,
    Grass,
    Sand,
    Tree,
    Flower,
    Shop,
    Ride,
    Track,
    Save,
    Load,
    Back,
    Forward,
    Close,
    Play,
    Pause,
    Fast,
    Fastest,
    Plus,
    Minus,
    RotateLeft,
    RotateRight,
    Grid,
    Terrain,
    Underground,
    Map,
    Finance,
    Guests,
    Chart,
    Settings,
    Help,
    Sound,
    Undo,
    Redo,
    Bin,
    Bench,
    Flag,
    Fence,
    Brush,
    Land,
    Tool,
    Clock,
    Palette,
    Pen,
    Check,
    Copy,
    Home,
    Star,
    Lock,
    File,
    Eye,
    Filter,
    Research,
    Sun,
    Moon,
    Rain,
    New,
    Power,
    Flat,
    SlopeUp,
    SlopeDown,
    CurveLeft,
    CurveRight,
    Station,
    Loop,
    Brake,
    Lift,
    Sign,
    Patrol,
    BrushSmaller,
    BrushLarger,
}

impl Glyph {
    pub(super) const ALL: &[Self] = &[
        Self::Menu,
        Self::Path,
        Self::Queue,
        Self::Gate,
        Self::Exit,
        Self::Search,
        Self::Hammer,
        Self::Raise,
        Self::Lower,
        Self::Water,
        Self::Grass,
        Self::Sand,
        Self::Tree,
        Self::Flower,
        Self::Shop,
        Self::Ride,
        Self::Track,
        Self::Save,
        Self::Load,
        Self::Back,
        Self::Forward,
        Self::Close,
        Self::Play,
        Self::Pause,
        Self::Fast,
        Self::Fastest,
        Self::Plus,
        Self::Minus,
        Self::RotateLeft,
        Self::RotateRight,
        Self::Grid,
        Self::Terrain,
        Self::Underground,
        Self::Map,
        Self::Finance,
        Self::Guests,
        Self::Chart,
        Self::Settings,
        Self::Help,
        Self::Sound,
        Self::Undo,
        Self::Redo,
        Self::Bin,
        Self::Bench,
        Self::Flag,
        Self::Fence,
        Self::Brush,
        Self::Land,
        Self::Tool,
        Self::Clock,
        Self::Palette,
        Self::Pen,
        Self::Check,
        Self::Copy,
        Self::Home,
        Self::Star,
        Self::Lock,
        Self::File,
        Self::Eye,
        Self::Filter,
        Self::Research,
        Self::Sun,
        Self::Moon,
        Self::Rain,
        Self::New,
        Self::Power,
        Self::Flat,
        Self::SlopeUp,
        Self::SlopeDown,
        Self::CurveLeft,
        Self::CurveRight,
        Self::Station,
        Self::Loop,
        Self::Brake,
        Self::Lift,
        Self::Sign,
        Self::Patrol,
        Self::BrushSmaller,
        Self::BrushLarger,
    ];

    pub(super) fn for_button(action: Option<&Action>, caption: &str) -> Self {
        match action {
            Some(Action::Menu) => Self::Menu,
            Some(Action::FocusTrain | Action::FindGuest) => Self::Search,
            Some(Action::SetPathTool(tool)) => match tool {
                PathTool::Inspect => Self::Search,
                PathTool::Build => Self::Path,
                PathTool::BuildQueue => Self::Queue,
                PathTool::Entrance => Self::Gate,
                PathTool::Exit => Self::Exit,
                PathTool::Service(_) => Self::Shop,
                PathTool::Carousel => Self::Ride,
                PathTool::Scenery(SceneryKind::Tree) => Self::Tree,
                PathTool::Scenery(SceneryKind::FlowerBed) => Self::Flower,
                PathTool::Demolish => Self::Hammer,
                PathTool::RaiseTerrain => Self::Raise,
                PathTool::LowerTerrain => Self::Lower,
                PathTool::PaintWater => Self::Water,
                PathTool::PaintGrass => Self::Grass,
                PathTool::PaintSand => Self::Sand,
                PathTool::BuyLand => Self::Land,
            },
            Some(Action::Track(TrackCommand::Piece(piece))) => match piece {
                TrackBuildKind::Station => Self::Station,
                TrackBuildKind::Straight => Self::Track,
                TrackBuildKind::CurveLeft | TrackBuildKind::BankedLeft => Self::CurveLeft,
                TrackBuildKind::CurveRight | TrackBuildKind::BankedRight => Self::CurveRight,
                TrackBuildKind::Lift => Self::Lift,
                TrackBuildKind::Drop
                | TrackBuildKind::SteepDrop
                | TrackBuildKind::TransitionDown => Self::SlopeDown,
                TrackBuildKind::TransitionUp => Self::SlopeUp,
                TrackBuildKind::Brake => Self::Brake,
                TrackBuildKind::Launch => Self::Fast,
                TrackBuildKind::Camelback | TrackBuildKind::HeartlineRoll => Self::Loop,
            },
            Some(Action::RotateLeft) => Self::RotateLeft,
            Some(Action::RotateRight) => Self::RotateRight,
            Some(Action::ZoomIn) => Self::Plus,
            Some(Action::ZoomOut) => Self::Minus,
            Some(Action::TerrainBrushRadius(delta)) => {
                if *delta < 0 {
                    Self::BrushSmaller
                } else {
                    Self::BrushLarger
                }
            }
            Some(Action::CyclePathShape) => {
                if caption.starts_with("Flat") {
                    Self::Flat
                } else if caption.starts_with("Ground") {
                    Self::Terrain
                } else if caption.starts_with("Brush") {
                    Self::Brush
                } else {
                    Self::SlopeUp
                }
            }
            Some(
                Action::TicketPrice(delta)
                | Action::ServicePrice(delta)
                | Action::PathHeight(delta),
            ) => {
                if *delta < 0 {
                    Self::Minus
                } else {
                    Self::Plus
                }
            }
            Some(Action::Catalogue(super::catalogue::Command::Ride(_))) => Self::Ride,
            Some(Action::Catalogue(super::catalogue::Command::Facility(_))) => Self::Shop,
            Some(Action::Grid) => Self::Grid,
            Some(Action::Terrain) => Self::Terrain,
            Some(Action::Underground) => Self::Underground,
            Some(Action::Home) => Self::Map,
            Some(Action::Save) => Self::Save,
            Some(Action::Load | Action::LoadBackup) => Self::Load,
            Some(Action::UndoPark) => Self::Undo,
            Some(Action::RedoPark) => Self::Redo,
            Some(Action::ToggleBin | Action::ToggleEmptyBins) => Self::Bin,
            Some(Action::ToggleBench) => Self::Bench,
            Some(Action::ToggleSign) => Self::Sign,
            Some(Action::TogglePatrol) => Self::Patrol,
            Some(Action::ToggleFence) => Self::Fence,
            Some(Action::CycleGuestFilter) => Self::Filter,
            Some(Action::SelectGuest(_)) => Self::Guests,
            Some(Action::RepairGuestPath) => Self::Tool,
            _ => Self::for_caption(caption),
        }
    }

    fn for_caption(caption: &str) -> Self {
        let text = caption.trim().to_lowercase();
        if text.starts_with('-')
            || text.starts_with('−')
            || text.contains("smaller")
            || text.contains("volume down")
        {
            return Self::Minus;
        }
        if text.starts_with('+') || text.contains("larger") || text.contains("volume up") {
            return Self::Plus;
        }
        if text.starts_with('<') || text.starts_with("previous") {
            return Self::Back;
        }
        if text.starts_with('>') || text.starts_with("next") {
            return Self::Forward;
        }
        let families: &[(Self, &[&str])] = &[
            (Self::Close, &["close", "cancel", "done"]),
            (Self::Back, &["back", "return"]),
            (Self::Copy, &["save as", "copy", "duplicate"]),
            (
                Self::New,
                &["new park", "create park", "new coaster", "new scenario"],
            ),
            (Self::Save, &["save", "export", "backup"]),
            (Self::Load, &["load", "import", "library"]),
            (Self::Exit, &["exit", "quit"]),
            (
                Self::Play,
                &[
                    "resume",
                    "continue",
                    "enter parkbound",
                    "play ",
                    "open ride",
                ],
            ),
            (Self::Pause, &["pause", "hold"]),
            (Self::Fastest, &["4x"]),
            (Self::Fast, &["2x", "launch"]),
            (Self::Play, &["1x"]),
            (Self::Search, &["find", "inspect", "follow", "focus"]),
            (Self::Gate, &["gate", "entrance", "access"]),
            (
                Self::Settings,
                &[
                    "settings",
                    "controls",
                    "management",
                    "policy",
                    "mode",
                    "display",
                    "scale",
                ],
            ),
            (Self::Help, &["help", "guide", "credits", "support"]),
            (
                Self::Sound,
                &[
                    "sound",
                    "audio",
                    "music",
                    "volume",
                    "ambience",
                    "effects",
                    "reactions",
                ],
            ),
            (Self::Undo, &["undo"]),
            (Self::Redo, &["redo", "restore", "reset"]),
            (Self::Hammer, &["demolish", "remove", "erase", "clear"]),
            (Self::Check, &["apply", "confirm", "accept", "test"]),
            (Self::Lock, &["locked", "unlock"]),
            (Self::Research, &["research", "fund"]),
            (
                Self::Palette,
                &[
                    "color",
                    "colour",
                    "paint",
                    "palette",
                    "structure",
                    "supports",
                    "vehicles",
                ],
            ),
            (Self::Pen, &["name", "rename", "edit"]),
            (Self::Clock, &["wait", "duration", "dispatch", "time"]),
            (
                Self::Tool,
                &["repair", "mechanic", "maintenance", "service ride"],
            ),
            (
                Self::Finance,
                &[
                    "finance", "loan", "budget", "ticket", "price", "cost", "charging", "repay",
                ],
            ),
            (
                Self::Chart,
                &["stats", "statistics", "ledger", "report", "economy"],
            ),
            (Self::Guests, &["guest", "visitor", "staff", "handyman"]),
            (
                Self::Flag,
                &[
                    "sign",
                    "patrol",
                    "marketing",
                    "ads",
                    "campaign",
                    "challenge",
                    "scenario",
                ],
            ),
            (
                Self::Shop,
                &[
                    "service",
                    "facilit",
                    "food",
                    "drink",
                    "toilet",
                    "rest",
                    "ice cream",
                    "souvenir",
                    "first aid",
                    "information",
                ],
            ),
            (Self::Tree, &["tree", "woodland"]),
            (Self::Flower, &["flower"]),
            (Self::Water, &["water"]),
            (Self::Grass, &["grass"]),
            (Self::Sand, &["sand"]),
            (Self::Raise, &["raise"]),
            (Self::Lower, &["lower"]),
            (Self::Bin, &["bin", "clean"]),
            (Self::Bench, &["bench"]),
            (Self::Fence, &["fence"]),
            (Self::Brush, &["brush"]),
            (Self::Land, &["buy land", "ownership"]),
            (Self::Flat, &["flat"]),
            (Self::Terrain, &["terrain", "landscape"]),
            (Self::Map, &["map", "explore", "fit", "sandbox"]),
            (Self::Queue, &["queue"]),
            (Self::Path, &["path"]),
            (Self::SlopeUp, &["climb", "slope up", "uphill"]),
            (Self::SlopeDown, &["drop", "slope down", "downhill"]),
            (Self::Track, &["track", "coaster", "transport", "train"]),
            (
                Self::Ride,
                &["ride", "carousel", "gentle", "thrill", "catalogue", "build"],
            ),
            (Self::Sun, &["lighting", "day", "sunset"]),
            (Self::Moon, &["night"]),
            (Self::Rain, &["weather", "rain"]),
            (Self::Eye, &["view", "visible", "contrast", "motion"]),
            (Self::Star, &["rating", "goal", "objective"]),
        ];
        families
            .iter()
            .find(|(_, words)| words.iter().any(|word| text.contains(word)))
            .map_or(Self::File, |(glyph, _)| *glyph)
    }

    pub(super) fn image(self) -> Image {
        let mut canvas = Canvas {
            pixels: vec![0; 64 * 64 * 4],
            ink: CREAM,
        };
        self.draw(&mut canvas);
        let mut image = Image::new(
            Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            canvas.pixels,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::linear();
        image
    }

    fn draw(self, c: &mut Canvas) {
        use Glyph::*;
        match self {
            Menu => {
                for y in [8., 16., 24.] {
                    c.line([5., y], [27., y]);
                }
            }
            Path => {
                c.color(WOOD);
                c.fill(&[[6., 29.], [26., 29.], [22., 5.], [10., 5.]]);
                c.color(CREAM);
                c.fill(&[[8., 27.], [24., 27.], [21., 4.], [11., 4.]]);
                c.color(WOOD);
                for y in [10., 17., 24.] {
                    c.line([10., y], [23., y]);
                }
                c.line([16., 5.], [16., 26.]);
            }
            Queue => {
                c.color(BLUE);
                c.fill(&[[5., 28.], [27., 28.], [25., 5.], [7., 5.]]);
                c.color(GOLD);
                c.poly(&[
                    [8., 25.],
                    [24., 25.],
                    [24., 17.],
                    [9., 17.],
                    [9., 9.],
                    [23., 9.],
                ]);
                c.color(CREAM);
                for p in [
                    [8., 25.],
                    [24., 25.],
                    [24., 17.],
                    [9., 17.],
                    [9., 9.],
                    [23., 9.],
                ] {
                    c.line(p, [p[0], p[1] - 3.]);
                    c.disc([p[0], p[1] - 3.], 1.1);
                }
            }
            Gate => {
                c.color(WOOD);
                c.block(5., 12., 10., 29.);
                c.block(22., 12., 27., 29.);
                c.color(GOLD);
                c.fill(&[[3., 12.], [16., 3.], [29., 12.]]);
                c.color(CREAM);
                c.line([5., 13.], [27., 13.]);
                c.line([8., 22.], [24., 22.]);
                c.color(RED);
                c.block(12., 20., 15., 24.);
                c.block(18., 20., 21., 24.);
            }
            Exit => {
                c.color(CREAM);
                c.poly(&[[14., 4.], [5., 4.], [5., 28.], [14., 28.]]);
                c.color(GREEN);
                c.fill(&[
                    [12., 13.],
                    [21., 13.],
                    [21., 8.],
                    [29., 16.],
                    [21., 24.],
                    [21., 19.],
                    [12., 19.],
                ]);
            }
            Search => {
                c.color(BLUE);
                c.disc([13., 13.], 7.);
                c.color(CREAM);
                c.circle([13., 13.], 8.);
                c.line([9., 10.], [12., 8.]);
                c.color(GOLD);
                c.fill(&[[18., 21.], [21., 18.], [29., 26.], [26., 29.]]);
            }
            Hammer => {
                c.color(WOOD);
                c.fill(&[[13., 17.], [17., 13.], [29., 25.], [25., 29.]]);
                c.color(CREAM);
                c.fill(&[[4., 10.], [9., 4.], [21., 12.], [16., 19.]]);
                c.color(BLUE);
                c.line([8., 10.], [16., 15.]);
            }
            Raise | Lower => {
                c.color(WOOD);
                c.fill(&[
                    [3., 23.],
                    [16., 29.],
                    [29., 23.],
                    [29., 27.],
                    [16., 32.],
                    [3., 27.],
                ]);
                c.color(GREEN);
                c.fill(&[[3., 23.], [16., 17.], [29., 23.], [16., 29.]]);
                c.color(if self == Raise { GOLD } else { BLUE });
                if self == Raise {
                    c.fill(&[
                        [12., 17.],
                        [20., 17.],
                        [20., 9.],
                        [25., 9.],
                        [16., 1.],
                        [7., 9.],
                        [12., 9.],
                    ]);
                } else {
                    c.fill(&[
                        [12., 2.],
                        [20., 2.],
                        [20., 10.],
                        [25., 10.],
                        [16., 18.],
                        [7., 10.],
                        [12., 10.],
                    ]);
                }
            }
            Water => {
                c.color(BLUE);
                c.fill(&[
                    [3., 24.],
                    [6., 10.],
                    [16., 4.],
                    [27., 10.],
                    [29., 24.],
                    [16., 30.],
                ]);
                c.color(CREAM);
                for y in [14., 21.] {
                    c.poly(&[
                        [6., y],
                        [10., y - 2.],
                        [16., y + 1.],
                        [22., y - 2.],
                        [27., y],
                    ]);
                }
            }
            Grass => {
                c.color(WOOD);
                c.fill(&[
                    [3., 21.],
                    [16., 27.],
                    [29., 21.],
                    [29., 25.],
                    [16., 31.],
                    [3., 25.],
                ]);
                c.color(GREEN);
                c.fill(&[[3., 21.], [16., 15.], [29., 21.], [16., 27.]]);
                c.color(CREAM);
                c.poly(&[
                    [10., 21.],
                    [8., 12.],
                    [15., 18.],
                    [17., 5.],
                    [20., 20.],
                    [25., 13.],
                    [24., 23.],
                ]);
            }
            Sand => {
                c.color(WOOD);
                c.fill(&[
                    [3., 18.],
                    [16., 25.],
                    [29., 18.],
                    [29., 24.],
                    [16., 31.],
                    [3., 24.],
                ]);
                c.color(GOLD);
                c.fill(&[[3., 18.], [16., 11.], [29., 18.], [16., 25.]]);
                c.color(CREAM);
                c.poly(&[[8., 17.], [14., 15.], [21., 19.], [25., 18.]]);
                for p in [[9., 21.], [18., 21.], [18., 14.], [23., 16.]] {
                    c.disc(p, 0.8);
                }
            }
            Tree => {
                c.color(WOOD);
                c.block(14., 15., 19., 30.);
                c.color(GREEN);
                c.disc([9., 15.], 6.);
                c.disc([22., 15.], 7.);
                c.disc([16., 8.], 7.);
                c.color(CREAM);
                c.poly(&[[10., 11.], [11., 7.], [15., 5.]]);
                c.color(GOLD);
                c.poly(&[[16., 26.], [16., 18.], [22., 14.]]);
            }
            Flower => {
                c.color(GREEN);
                c.line([16., 14.], [16., 30.]);
                c.fill(&[[16., 25.], [8., 20.], [8., 26.], [16., 28.]]);
                c.fill(&[[16., 23.], [24., 18.], [24., 24.], [16., 26.]]);
                c.color(RED);
                for p in [[16., 5.], [23., 10.], [20., 17.], [12., 17.], [9., 10.]] {
                    c.disc(p, 4.);
                }
                c.color(GOLD);
                c.disc([16., 11.], 4.);
                c.color(CREAM);
                c.disc([15., 10.], 1.);
            }
            Shop => {
                c.color(CREAM);
                c.block(6., 13., 26., 29.);
                c.color(BLUE);
                c.fill(&[[3., 14.], [7., 4.], [25., 4.], [29., 14.]]);
                c.color(GOLD);
                c.fill(&[[8., 14.], [11., 4.], [15., 4.], [14., 14.]]);
                c.fill(&[[20., 14.], [19., 4.], [23., 4.], [25., 14.]]);
                c.color(WOOD);
                c.block(11., 20., 21., 29.);
                c.color(GOLD);
                c.line([4., 15.], [28., 15.]);
            }
            Ride => {
                c.color(GOLD);
                c.poly(&[[6., 29.], [16., 13.], [26., 29.]]);
                c.color(BLUE);
                c.circle([16., 13.], 10.);
                c.color(CREAM);
                for a in 0..6 {
                    let angle = a as f32 * std::f32::consts::TAU / 6.;
                    let point = [16. + angle.cos() * 10., 13. + angle.sin() * 10.];
                    c.line([16., 13.], point);
                    c.color(RED);
                    c.block(point[0] - 2., point[1], point[0] + 2., point[1] + 3.);
                    c.color(CREAM);
                }
                c.disc([16., 13.], 2.);
                c.line([4., 30.], [28., 30.]);
            }
            Flat => {
                c.color(WOOD);
                c.fill(&[
                    [3., 19.],
                    [16., 26.],
                    [29., 19.],
                    [29., 23.],
                    [16., 30.],
                    [3., 23.],
                ]);
                c.color(CREAM);
                c.fill(&[[3., 19.], [16., 12.], [29., 19.], [16., 26.]]);
                c.color(GOLD);
                c.line([8., 19.], [24., 19.]);
            }
            Track | Lift | SlopeUp | SlopeDown | Brake => {
                let (a, b) = match self {
                    SlopeUp | Lift => ([4., 25.], [28., 7.]),
                    SlopeDown => ([4., 7.], [28., 25.]),
                    _ => ([4., 11.], [28., 11.]),
                };
                c.line(a, b);
                c.line([a[0], a[1] + 8.], [b[0], b[1] + 8.]);
                for t in [0.1, 0.35, 0.6, 0.85] {
                    let x = a[0] + (b[0] - a[0]) * t;
                    let y = a[1] + (b[1] - a[1]) * t;
                    c.line([x, y], [x, y + 8.]);
                }
                if self == Brake {
                    c.line([11., 6.], [11., 24.]);
                    c.line([21., 6.], [21., 24.]);
                }
                if self == Lift {
                    c.poly(&[[20., 4.], [28., 4.], [28., 11.]]);
                }
            }
            Station => {
                c.poly(&[[3., 11.], [16., 3.], [29., 11.]]);
                c.rect(5., 11., 27., 28.);
                c.rect(10., 17., 22., 28.);
                c.line([3., 28.], [29., 28.]);
            }
            CurveLeft | CurveRight => {
                let right = self == CurveRight;
                for x in [6., 14.] {
                    let points = [[x, 28.], [x, 17.], [x + 3., 10.], [x + 10., 6.], [28., 6.]];
                    let mapped = points.map(|p| if right { [32. - p[0], p[1]] } else { p });
                    c.poly(&mapped);
                }
            }
            Loop => {
                c.poly(&[
                    [3., 27.],
                    [12., 27.],
                    [23., 21.],
                    [26., 12.],
                    [23., 5.],
                    [16., 3.],
                    [9., 7.],
                    [8., 16.],
                    [13., 23.],
                    [22., 27.],
                    [29., 27.],
                ]);
            }
            Save => {
                c.poly(&[
                    [5., 4.],
                    [24., 4.],
                    [28., 8.],
                    [28., 28.],
                    [5., 28.],
                    [5., 4.],
                ]);
                c.rect(10., 4., 21., 12.);
                c.rect(10., 20., 23., 28.);
                c.line([18., 7.], [18., 10.]);
            }
            Load => {
                c.poly(&[
                    [3., 26.],
                    [3., 9.],
                    [12., 9.],
                    [15., 12.],
                    [28., 12.],
                    [25., 26.],
                    [3., 26.],
                ]);
                c.line([16., 3.], [16., 20.]);
                c.poly(&[[11., 15.], [16., 20.], [21., 15.]]);
            }
            Back | Forward | Undo | Redo => {
                let right = matches!(self, Forward | Redo);
                let points = if matches!(self, Undo | Redo) {
                    vec![[5., 11.], [18., 11.], [25., 16.], [25., 25.]]
                } else {
                    vec![[5., 16.], [27., 16.]]
                };
                let points: Vec<_> = points
                    .into_iter()
                    .map(|p| if right { [32. - p[0], p[1]] } else { p })
                    .collect();
                c.poly(&points);
                let y = if matches!(self, Undo | Redo) {
                    11.
                } else {
                    16.
                };
                let x = if right { 27. } else { 5. };
                let tip = if right { 21. } else { 11. };
                c.poly(&[[tip, y - 6.], [x, y], [tip, y + 6.]]);
            }
            Close => {
                c.line([7., 7.], [25., 25.]);
                c.line([7., 25.], [25., 7.]);
            }
            Play => c.poly(&[[10., 5.], [27., 16.], [10., 27.], [10., 5.]]),
            Pause => {
                c.rect(7., 5., 12., 27.);
                c.rect(20., 5., 25., 27.);
            }
            Fast | Fastest => {
                for x in [3., 15.] {
                    c.poly(&[[x, 7.], [x + 11., 16.], [x, 25.], [x, 7.]]);
                }
                if self == Fastest {
                    c.line([29., 7.], [29., 25.]);
                }
            }
            Plus | Minus => {
                c.line([5., 16.], [27., 16.]);
                if self == Plus {
                    c.line([16., 5.], [16., 27.]);
                }
            }
            RotateLeft | RotateRight => {
                let right = self == RotateRight;
                let points = [
                    [6., 12.],
                    [9., 6.],
                    [18., 4.],
                    [26., 10.],
                    [26., 21.],
                    [18., 27.],
                    [9., 25.],
                ];
                c.poly(&points.map(|p| if right { [32. - p[0], p[1]] } else { p }));
                let points = [[3., 5.], [6., 12.], [13., 10.]];
                c.poly(&points.map(|p| if right { [32. - p[0], p[1]] } else { p }));
            }
            Grid => {
                c.rect(4., 4., 28., 28.);
                for n in [12., 20.] {
                    c.line([n, 4.], [n, 28.]);
                    c.line([4., n], [28., n]);
                }
            }
            Terrain => c.poly(&[
                [3., 25.],
                [12., 7.],
                [18., 17.],
                [23., 11.],
                [29., 25.],
                [3., 25.],
            ]),
            Underground => {
                c.line([3., 9.], [29., 9.]);
                c.poly(&[
                    [5., 27.],
                    [5., 21.],
                    [9., 14.],
                    [23., 14.],
                    [27., 21.],
                    [27., 27.],
                ]);
                c.line([12., 27.], [12., 23.]);
                c.line([20., 27.], [20., 23.]);
            }
            Map => {
                c.poly(&[
                    [3., 7.],
                    [11., 4.],
                    [21., 8.],
                    [29., 5.],
                    [29., 25.],
                    [21., 28.],
                    [11., 24.],
                    [3., 27.],
                    [3., 7.],
                ]);
                c.line([11., 4.], [11., 24.]);
                c.line([21., 8.], [21., 28.]);
            }
            Finance => {
                c.circle([16., 16.], 12.);
                c.line([16., 6.], [16., 26.]);
                c.poly(&[
                    [22., 10.],
                    [12., 10.],
                    [9., 13.],
                    [12., 16.],
                    [20., 16.],
                    [23., 20.],
                    [20., 23.],
                    [9., 23.],
                ]);
            }
            Guests => {
                c.circle([16., 9.], 4.);
                c.poly(&[
                    [8., 27.],
                    [8., 21.],
                    [12., 17.],
                    [20., 17.],
                    [24., 21.],
                    [24., 27.],
                ]);
                c.circle([5., 12.], 3.);
                c.circle([27., 12.], 3.);
            }
            Chart => {
                c.poly(&[[4., 4.], [4., 28.], [29., 28.]]);
                c.line([10., 23.], [10., 18.]);
                c.line([17., 23.], [17., 11.]);
                c.line([24., 23.], [24., 6.]);
            }
            Settings => {
                c.circle([16., 16.], 8.);
                c.circle([16., 16.], 3.);
                for a in 0..8 {
                    let a = a as f32 * std::f32::consts::FRAC_PI_4;
                    c.line(
                        [16. + a.cos() * 9., 16. + a.sin() * 9.],
                        [16. + a.cos() * 13., 16. + a.sin() * 13.],
                    );
                }
            }
            Help => {
                c.circle([16., 16.], 12.);
                c.poly(&[
                    [11., 11.],
                    [13., 8.],
                    [19., 8.],
                    [22., 12.],
                    [20., 16.],
                    [16., 18.],
                    [16., 20.],
                ]);
                c.disc([16., 25.], 1.2);
            }
            Sound => {
                c.poly(&[
                    [4., 12.],
                    [10., 12.],
                    [18., 5.],
                    [18., 27.],
                    [10., 20.],
                    [4., 20.],
                    [4., 12.],
                ]);
                c.poly(&[[23., 9.], [26., 13.], [26., 19.], [23., 23.]]);
                c.poly(&[[27., 5.], [30., 12.], [30., 20.], [27., 27.]]);
            }
            Bin => {
                c.color(BLUE);
                c.fill(&[[7., 9.], [9., 28.], [23., 28.], [25., 9.]]);
                c.color(CREAM);
                c.line([4., 9.], [28., 9.]);
                c.poly(&[[11., 8.], [11., 4.], [21., 4.], [21., 8.]]);
                c.line([13., 14.], [13., 23.]);
                c.line([19., 14.], [19., 23.]);
            }
            Bench => {
                c.color(WOOD);
                c.block(4., 7., 28., 16.);
                c.block(3., 19., 29., 23.);
                c.color(CREAM);
                c.line([5., 11.], [27., 11.]);
                c.line([7., 16.], [7., 28.]);
                c.line([25., 16.], [25., 28.]);
            }
            Sign => {
                c.color(WOOD);
                c.block(14., 5., 18., 30.);
                c.color(GOLD);
                c.fill(&[[4., 5.], [23., 5.], [29., 10.], [23., 15.], [4., 15.]]);
                c.color(CREAM);
                c.poly(&[[19., 8.], [22., 10.], [19., 12.]]);
            }
            Patrol => {
                c.color(GOLD);
                c.poly(&[[4., 27.], [4., 20.], [10., 20.], [10., 10.], [27., 10.]]);
                c.color(BLUE);
                c.fill(&[
                    [11., 5.],
                    [15., 2.],
                    [19., 5.],
                    [19., 11.],
                    [15., 18.],
                    [11., 11.],
                ]);
                c.color(CREAM);
                c.disc([15., 7.], 2.);
                c.color(GREEN);
                c.block(20., 22., 28., 27.);
                c.disc([26., 20.], 2.);
            }
            Flag => {
                c.line([7., 3.], [7., 29.]);
                c.poly(&[[7., 4.], [26., 4.], [22., 10.], [26., 16.], [7., 16.]]);
            }
            Fence => {
                c.color(WOOD);
                c.block(3., 13., 29., 17.);
                c.block(3., 22., 29., 26.);
                c.color(CREAM);
                for x in [6., 16., 26.] {
                    c.fill(&[
                        [x - 2., 28.],
                        [x - 2., 7.],
                        [x, 4.],
                        [x + 2., 7.],
                        [x + 2., 28.],
                    ]);
                }
            }
            Brush | BrushSmaller | BrushLarger => {
                c.color(WOOD);
                c.fill(&[[7., 20.], [20., 4.], [25., 8.], [12., 24.]]);
                c.color(CREAM);
                c.fill(&[[7., 20.], [4., 23.], [3., 29.], [10., 28.], [12., 24.]]);
                c.color(BLUE);
                c.line([7., 21.], [11., 24.]);
                if self != Brush {
                    c.color(GOLD);
                    c.line([20., 23.], [29., 23.]);
                    if self == BrushLarger {
                        c.line([24.5, 18.5], [24.5, 27.5]);
                    }
                }
            }
            Land => {
                c.color(GREEN);
                c.fill(&[[3., 20.], [16., 13.], [29., 20.], [16., 28.]]);
                c.color(WOOD);
                c.poly(&[[3., 23.], [16., 31.], [29., 23.]]);
                c.color(GOLD);
                c.disc([16., 8.], 7.);
                c.color(CREAM);
                c.line([16., 4.], [16., 12.]);
                c.line([13., 8.], [19., 8.]);
            }
            Tool => {
                c.poly(&[
                    [21., 4.],
                    [18., 11.],
                    [23., 15.],
                    [29., 10.],
                    [28., 17.],
                    [22., 20.],
                    [18., 17.],
                    [7., 28.],
                    [3., 24.],
                    [14., 13.],
                    [14., 7.],
                    [21., 4.],
                ]);
            }
            Clock => {
                c.circle([16., 16.], 12.);
                c.poly(&[[16., 7.], [16., 16.], [22., 20.]]);
            }
            Palette => {
                c.poly(&[
                    [28., 16.],
                    [27., 8.],
                    [19., 4.],
                    [10., 5.],
                    [4., 12.],
                    [4., 21.],
                    [10., 28.],
                    [17., 28.],
                    [18., 23.],
                    [14., 19.],
                    [19., 16.],
                    [28., 16.],
                ]);
                for p in [[11., 11.], [20., 10.], [8., 20.]] {
                    c.disc(p, 2.);
                }
            }
            Pen => {
                c.poly(&[
                    [4., 28.],
                    [6., 20.],
                    [23., 3.],
                    [29., 9.],
                    [12., 26.],
                    [4., 28.],
                ]);
                c.line([19., 7.], [25., 13.]);
            }
            Check => c.poly(&[[5., 17.], [12., 24.], [27., 7.]]),
            Copy => {
                c.rect(10., 10., 28., 28.);
                c.poly(&[[22., 6.], [22., 4.], [4., 4.], [4., 22.], [6., 22.]]);
            }
            Home => {
                c.poly(&[[3., 14.], [16., 3.], [29., 14.]]);
                c.poly(&[[7., 12.], [7., 28.], [25., 28.], [25., 12.]]);
                c.rect(13., 20., 19., 28.);
            }
            Star => c.poly(&[
                [16., 3.],
                [20., 11.],
                [29., 12.],
                [23., 19.],
                [24., 28.],
                [16., 24.],
                [8., 28.],
                [9., 19.],
                [3., 12.],
                [12., 11.],
                [16., 3.],
            ]),
            Lock => {
                c.rect(6., 14., 26., 28.);
                c.poly(&[
                    [10., 14.],
                    [10., 8.],
                    [13., 4.],
                    [19., 4.],
                    [22., 8.],
                    [22., 14.],
                ]);
                c.line([16., 20.], [16., 24.]);
            }
            File | New => {
                c.poly(&[
                    [6., 3.],
                    [21., 3.],
                    [27., 9.],
                    [27., 29.],
                    [6., 29.],
                    [6., 3.],
                ]);
                c.poly(&[[21., 3.], [21., 9.], [27., 9.]]);
                c.line([11., 18.], [22., 18.]);
                if self == New {
                    c.line([16., 13.], [16., 24.]);
                } else {
                    c.line([11., 24.], [22., 24.]);
                }
            }
            Eye => {
                c.poly(&[
                    [3., 16.],
                    [9., 9.],
                    [16., 6.],
                    [23., 9.],
                    [29., 16.],
                    [23., 23.],
                    [16., 26.],
                    [9., 23.],
                    [3., 16.],
                ]);
                c.circle([16., 16.], 5.);
            }
            Filter => c.poly(&[
                [4., 5.],
                [28., 5.],
                [19., 16.],
                [19., 26.],
                [13., 29.],
                [13., 16.],
                [4., 5.],
            ]),
            Research => {
                c.poly(&[
                    [12., 3.],
                    [20., 3.],
                    [20., 13.],
                    [28., 26.],
                    [26., 29.],
                    [6., 29.],
                    [4., 26.],
                    [12., 13.],
                    [12., 3.],
                ]);
                c.line([9., 21.], [23., 21.]);
            }
            Sun => {
                c.circle([16., 16.], 6.);
                for a in 0..8 {
                    let a = a as f32 * std::f32::consts::FRAC_PI_4;
                    c.line(
                        [16. + a.cos() * 10., 16. + a.sin() * 10.],
                        [16. + a.cos() * 13., 16. + a.sin() * 13.],
                    );
                }
            }
            Moon => c.poly(&[
                [19., 3.],
                [10., 6.],
                [5., 14.],
                [7., 23.],
                [15., 28.],
                [23., 26.],
                [28., 19.],
                [19., 20.],
                [13., 14.],
                [14., 7.],
                [19., 3.],
            ]),
            Rain => {
                c.poly(&[
                    [4., 17.],
                    [3., 12.],
                    [6., 8.],
                    [11., 8.],
                    [15., 4.],
                    [22., 6.],
                    [24., 11.],
                    [28., 11.],
                    [29., 17.],
                    [4., 17.],
                ]);
                for x in [8., 16., 24.] {
                    c.line([x, 22.], [x - 2., 28.]);
                }
            }
            Power => {
                c.poly(&[
                    [9., 6.],
                    [4., 13.],
                    [5., 22.],
                    [12., 28.],
                    [21., 27.],
                    [28., 20.],
                    [27., 11.],
                    [23., 6.],
                ]);
                c.line([16., 3.], [16., 16.]);
            }
        }
    }
}

const CREAM: [u8; 3] = [255, 244, 212];
const GOLD: [u8; 3] = [239, 184, 70];
const BLUE: [u8; 3] = [85, 186, 219];
const GREEN: [u8; 3] = [114, 186, 109];
const RED: [u8; 3] = [231, 126, 111];
const WOOD: [u8; 3] = [185, 130, 75];

struct Canvas {
    pixels: Vec<u8>,
    ink: [u8; 3],
}
impl Canvas {
    fn color(&mut self, color: [u8; 3]) {
        self.ink = color;
    }

    fn paint(&mut self, pixel: usize, coverage: f32) {
        if coverage <= 0.0 {
            return;
        }
        let previous = f32::from(self.pixels[pixel + 3]) / 255.0;
        let alpha = coverage + previous * (1.0 - coverage);
        for channel in 0..3 {
            self.pixels[pixel + channel] = ((f32::from(self.ink[channel]) * coverage
                + f32::from(self.pixels[pixel + channel]) * previous * (1.0 - coverage))
                / alpha)
                .round() as u8;
        }
        self.pixels[pixel + 3] = (alpha * 255.0).round() as u8;
    }

    fn fill(&mut self, points: &[[f32; 2]]) {
        for y in 0..64 {
            for x in 0..64 {
                let mut covered = 0;
                for offset in [[0.25, 0.25], [0.75, 0.25], [0.25, 0.75], [0.75, 0.75]] {
                    let p = Vec2::new((x as f32 + offset[0]) / 2.0, (y as f32 + offset[1]) / 2.0);
                    let mut inside = false;
                    for i in 0..points.len() {
                        let a = Vec2::from(points[i]);
                        let b = Vec2::from(points[(i + 1) % points.len()]);
                        if (a.y > p.y) != (b.y > p.y)
                            && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x
                        {
                            inside = !inside;
                        }
                    }
                    covered += u8::from(inside);
                }
                self.paint((y * 64 + x) * 4, f32::from(covered) / 4.0);
            }
        }
    }

    fn block(&mut self, x: f32, y: f32, right: f32, bottom: f32) {
        self.fill(&[[x, y], [right, y], [right, bottom], [x, bottom]]);
    }
    fn line(&mut self, a: [f32; 2], b: [f32; 2]) {
        let a = Vec2::from(a);
        let b = Vec2::from(b);
        let ab = b - a;
        self.stroke(|p| {
            let t = ((p - a).dot(ab) / ab.length_squared().max(0.001)).clamp(0., 1.);
            (p - a - ab * t).length()
        });
    }
    fn poly(&mut self, points: &[[f32; 2]]) {
        for pair in points.windows(2) {
            self.line(pair[0], pair[1]);
        }
    }
    fn rect(&mut self, x: f32, y: f32, right: f32, bottom: f32) {
        self.poly(&[[x, y], [right, y], [right, bottom], [x, bottom], [x, y]]);
    }
    fn circle(&mut self, center: [f32; 2], radius: f32) {
        self.stroke(|p| (p.distance(Vec2::from(center)) - radius).abs());
    }
    fn disc(&mut self, center: [f32; 2], radius: f32) {
        self.stroke(|p| (p.distance(Vec2::from(center)) - radius + 0.8).max(0.));
    }
    fn stroke(&mut self, distance: impl Fn(Vec2) -> f32) {
        for y in 0..64 {
            for x in 0..64 {
                let coverage = ((1.15
                    - distance(Vec2::new((x as f32 + 0.5) / 2., (y as f32 + 0.5) / 2.)))
                    * 2.)
                    .clamp(0., 1.);
                let pixel = (y * 64 + x) * 4;
                self.paint(pixel, coverage);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_pictogram_has_visible_antialiased_artwork() {
        for (index, glyph) in Glyph::ALL.iter().enumerate() {
            assert_eq!(*glyph as usize, index);
            let image = glyph.image();
            let pixels = image.data.unwrap();
            assert!(
                pixels.chunks_exact(4).filter(|p| p[3] > 128).count() > 100,
                "{glyph:?}"
            );
            assert!(
                pixels.chunks_exact(4).any(|p| p[3] > 0 && p[3] < 255),
                "{glyph:?}"
            );
        }
    }
    #[test]
    fn construction_and_ride_controls_have_distinct_icons() {
        for (tool, glyph) in [
            (PathTool::Build, Glyph::Path),
            (PathTool::BuildQueue, Glyph::Queue),
            (PathTool::Entrance, Glyph::Gate),
            (PathTool::Exit, Glyph::Exit),
            (PathTool::RaiseTerrain, Glyph::Raise),
            (PathTool::LowerTerrain, Glyph::Lower),
        ] {
            assert_eq!(
                Glyph::for_button(Some(&Action::SetPathTool(tool)), ""),
                glyph
            );
        }
        assert_eq!(
            Glyph::for_button(Some(&Action::FocusTrain), "Find [F]"),
            Glyph::Search
        );
        assert_eq!(Glyph::for_button(None, "Save As New Park"), Glyph::Copy);
        assert_eq!(
            Glyph::for_button(Some(&Action::ToggleSign), "Sign"),
            Glyph::Sign
        );
        assert_eq!(
            Glyph::for_button(Some(&Action::TogglePatrol), "Patrol"),
            Glyph::Patrol
        );
        assert_ne!(Glyph::Path.image().data, Glyph::Queue.image().data);
    }
}
