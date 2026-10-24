//! Derived library presentation. No preview writes to the park or design store.
use bevy::prelude::*;
use park_sim::{Park, TrackDesignError, TrackPieceSpec};

use crate::{ViewState, design_library::DesignLibrary, design_thumbnail, ui::designs::DesignImage};

pub(crate) const THUMBNAIL_WIDTH: u32 = 156;
pub(crate) const THUMBNAIL_HEIGHT: u32 = 70;
pub(crate) const PREVIEW_WIDTH: u32 = 320;
pub(crate) const PREVIEW_HEIGHT: u32 = 180;
pub(crate) const PAGE_SIZE: usize = 4;

pub(crate) fn page_start(library: &DesignLibrary) -> usize {
    library
        .selected
        .filter(|index| *index < library.entries.len())
        .unwrap_or(0)
        / PAGE_SIZE
        * PAGE_SIZE
}

/// The shape image is normalized; these figures describe the actual placement.
/// Quotes use current world/cash state and never replace the editor's draft.
pub(crate) fn status_text(park: &Park, library: &DesignLibrary) -> String {
    let Some(design) = library.selected_design() else {
        return "Save a draft to see its shape here.\nThumbnails are recreated from saved track.\nBrowsing never changes the park.".into();
    };
    let specs = match design.transformed(library.anchor, library.height, library.rotation) {
        Ok(specs) => specs,
        Err(error) => return match error {
            TrackDesignError::HeightOutOfBounds => "Height exceeds the track limits.\nAdjust entry height before loading.\nPreview leaves the draft unchanged.".into(),
            _ => "Design extends outside the map.\nAdjust position or turn before loading.\nPreview leaves the draft unchanged.".into(),
        },
    };
    match park.track_construction_quote_for(&specs) {
        Ok(quote) => {
            let readiness = if park.track_construction_operating_error().is_some() {
                "Close and empty the coaster to build."
            } else if !quote.affordable() {
                "Not enough cash to build here."
            } else {
                "Complete loop / ready to build."
            };
            let settlement = if quote.refund_cents > quote.construction_cents {
                format!(
                    "Receive {}",
                    crate::ui::money(quote.refund_cents - quote.construction_cents)
                )
            } else {
                format!("Pay {}", crate::ui::money(quote.required_cents()))
            };
            format!(
                "{readiness}\nBuy {} / Refund {}\n{settlement} / Cash {}\nLoad is free; Apply checks again.",
                crate::ui::money(quote.construction_cents),
                crate::ui::money(quote.refund_cents),
                crate::ui::money(quote.available_cents)
            )
        }
        Err(error) => {
            let reason = if let park_sim::TrackConstructionError::IncompleteLoop(pos) = error {
                format!(
                    "Join track at {}, {}.\nMatch the next entry arrow.",
                    pos.x + 1,
                    pos.y + 1
                )
            } else {
                crate::track_tools::error_message(park, error)
            };
            format!(
                "{reason}\nNo build quote at this placement.\nLoad to edit; Apply checks again."
            )
        }
    }
}

struct Thumbnail {
    pieces: Vec<TrackPieceSpec>,
    image: Handle<Image>,
}

#[derive(Default)]
pub(crate) struct PreviewImages {
    thumbnails: Vec<Thumbnail>,
    selected: Option<(Vec<TrackPieceSpec>, u8, Handle<Image>)>,
}

impl PreviewImages {
    fn sync(&mut self, library: &DesignLibrary, images: &mut Assets<Image>) {
        // Reconcile by geometry: rename and selection changes do not rerasterize
        // unchanged thumbnails. Removed/updated entries release their assets.
        if self.thumbnails.len() != library.entries.len()
            || self
                .thumbnails
                .iter()
                .zip(&library.entries)
                .any(|(cached, design)| cached.pieces != design.pieces())
        {
            let mut previous = std::mem::take(&mut self.thumbnails);
            for design in &library.entries {
                if let Some(index) = previous
                    .iter()
                    .position(|cached| cached.pieces == design.pieces())
                {
                    self.thumbnails.push(previous.swap_remove(index));
                } else {
                    self.thumbnails.push(Thumbnail {
                        pieces: design.pieces().to_vec(),
                        image: images.add(design_thumbnail::render(
                            design,
                            0,
                            THUMBNAIL_WIDTH,
                            THUMBNAIL_HEIGHT,
                        )),
                    });
                }
            }
            for removed in previous {
                images.remove(removed.image.id());
            }
        }
        if let Some(design) = library.selected_design() {
            let rotation = library.rotation % 4;
            if self
                .selected
                .as_ref()
                .is_none_or(|(pieces, turn, _)| *pieces != design.pieces() || *turn != rotation)
            {
                let rendered =
                    design_thumbnail::render(design, rotation, PREVIEW_WIDTH, PREVIEW_HEIGHT);
                let handle = match self.selected.take() {
                    Some((_, _, handle)) => {
                        if let Some(image) = images.get_mut(&handle) {
                            *image = rendered;
                            handle
                        } else {
                            images.add(rendered)
                        }
                    }
                    None => images.add(rendered),
                };
                self.selected = Some((design.pieces().to_vec(), rotation, handle));
            }
        } else if let Some((_, _, handle)) = self.selected.take() {
            images.remove(handle.id());
        }
    }
}

pub(crate) fn refresh_images(
    library: Res<DesignLibrary>,
    view: Res<ViewState>,
    mut images: ResMut<Assets<Image>>,
    mut nodes: Query<(&DesignImage, &mut ImageNode)>,
    mut cache: Local<PreviewImages>,
) {
    if !view.designs_open {
        return;
    }
    cache.sync(&library, &mut images);
    let start = page_start(&library);
    for (kind, mut node) in &mut nodes {
        let image = match kind {
            DesignImage::Preview => cache.selected.as_ref().map(|(_, _, image)| image),
            DesignImage::Thumbnail(slot) => cache
                .thumbnails
                .get(start + slot)
                .map(|cached| &cached.image),
        };
        let (handle, color) = image.map_or_else(
            || (Handle::default(), Color::NONE),
            |image| (image.clone(), Color::WHITE),
        );
        if node.image != handle || node.color != color {
            node.image = handle;
            node.color = color;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::{TilePos, TrackBuildKind, TrackDesign, TrackDirection};

    fn design(name: &str, kind: TrackBuildKind) -> TrackDesign {
        TrackDesign::from_draft(
            name,
            &[kind.spec(TilePos { x: 12, y: 12 }, 8, TrackDirection::East)],
        )
        .unwrap()
    }

    #[test]
    fn derived_images_reuse_geometry_and_release_deleted_or_updated_assets() {
        let mut library = DesignLibrary::default();
        library.entries = vec![
            design("First", TrackBuildKind::Station),
            design("Second", TrackBuildKind::Lift),
        ];
        library.selected = Some(0);
        let mut assets = Assets::<Image>::default();
        let mut cache = PreviewImages::default();
        cache.sync(&library, &mut assets);
        let first = cache.thumbnails[0].image.id();
        let preview = cache.selected.as_ref().unwrap().2.id();
        let initial_pixels = assets.get(preview).unwrap().data.clone();
        library.entries[0] = design("Renamed", TrackBuildKind::Station);
        library.anchor.x += 1;
        library.height += 1;
        cache.sync(&library, &mut assets);
        assert_eq!(cache.thumbnails[0].image.id(), first);
        assert_eq!(cache.selected.as_ref().unwrap().2.id(), preview);
        assert_eq!(assets.get(preview).unwrap().data, initial_pixels);
        library.rotation = 1;
        cache.sync(&library, &mut assets);
        assert_ne!(assets.get(preview).unwrap().data, initial_pixels);
        assert_eq!(cache.thumbnails[0].image.id(), first);
        for _ in 0..8 {
            library.entries[0] = design("Updated", TrackBuildKind::Brake);
            cache.sync(&library, &mut assets);
            library.entries[0] = design("Updated", TrackBuildKind::Drop);
            cache.sync(&library, &mut assets);
            assert_eq!(assets.len(), 3);
        }
        assert!(assets.get(first).is_none());
        library.entries.clear();
        library.selected = None;
        cache.sync(&library, &mut assets);
        assert!(assets.is_empty());
    }

    #[test]
    fn preview_and_thumbnail_browsing_preserve_park_draft_and_history() {
        let mut park = crate::demonstration_park();
        let mut view = ViewState::default();
        crate::track_tools::apply_command(
            &mut park,
            &mut view,
            crate::track_tools::TrackCommand::CopyInstalled,
        );
        let before = park.clone();
        let counts = (view.track_editor.undo_len(), view.track_editor.redo_len());
        let mut library = DesignLibrary::default();
        library.entries = vec![
            TrackDesign::from_draft("Installed", park.track_draft()).unwrap(),
            design("Unfinished", TrackBuildKind::Lift),
        ];
        let mut cache = PreviewImages::default();
        let mut assets = Assets::<Image>::default();
        for index in 0..2 {
            library.selected = Some(index);
            (library.anchor, library.height) = library.entries[index].anchor();
            for rotation in 0..4 {
                library.rotation = rotation;
                cache.sync(&library, &mut assets);
                assert!(!status_text(&park, &library).is_empty());
                assert_eq!(park, before);
                assert_eq!(park.path_cache_stats(), before.path_cache_stats());
                assert_eq!(
                    counts,
                    (view.track_editor.undo_len(), view.track_editor.redo_len())
                );
            }
        }
        library.anchor = TilePos { x: 0, y: 0 };
        library.selected = Some(0);
        assert!(status_text(&park, &library).contains("outside the map"));
        assert_eq!(park, before);
    }
}
