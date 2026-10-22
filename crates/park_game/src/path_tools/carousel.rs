//! Deliberate placement validates the complete three-by-three footprint.
use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CarouselPreview {
    pub origin: TilePos,
    pub height: u8,
    pub valid: bool,
    pub demolish: bool,
}

pub(super) fn point_at(park: &mut Park, view: &mut ViewState, point: Vec2, click: bool) {
    let height = view.path_tool.height;
    let Some(origin) =
        world::pick_sloped_path_plane(point, park, view.rotation, height, PathSlope::Flat)
    else {
        view.path_tool.hint = "Outside the park at this height.".into();
        if click {
            view.path_notice = Some("Carousel not placed: outside the park.".into());
        }
        return;
    };
    let occluded = world::sloped_path_plane_occluded(
        point,
        park,
        view.rotation,
        height,
        PathSlope::Flat,
        view.view_mode,
    );
    let reason = park
        .validate_carousel(origin, height)
        .err()
        .map(|error| error.to_string())
        .or_else(|| {
            occluded.then(|| "Surface hidden here. Rotate or use underground view.".into())
        });
    if !occluded {
        view.path_tool.carousel_preview = Some(CarouselPreview {
            origin,
            height,
            valid: reason.is_none(),
            demolish: false,
        });
    }
    view.path_tool.hint = reason
        .clone()
        .unwrap_or_else(|| format!("3 x 3 / height {height}\nClick to build a closed carousel."));
    if !click {
        return;
    }
    if let Some(reason) = reason {
        view.path_notice = Some(format!("Carousel not placed: {reason}"));
        return;
    }
    match park.place_carousel(origin, height) {
        Ok(id) => {
            view.select_ride(id);
            let object = park.carousel(id).and_then(|c| c.objects.first()).copied();
            view.selected = object.map(SurfaceId::Object);
            view.hovered = view.selected;
            view.scene_dirty = true;
            view.path_tool.set_mode(PathTool::Inspect);
            view.path_notice =
                Some("Carousel built. Add its queue [J], entrance [I] and exit [K].".into());
        }
        Err(error) => view.path_notice = Some(format!("Carousel not placed: {error}")),
    }
}
