//! Embedded artwork and typography for the classic park presentation.
use bevy::{
    asset::RenderAssetUsages,
    image::{CompressedImageFormats, ImageSampler, ImageType},
    prelude::*,
};

pub(crate) const FOREST: Color = Color::srgb_u8(19, 62, 48);
pub(crate) const CREAM: Color = Color::srgb_u8(255, 244, 212);
pub(crate) const GOLD: Color = Color::srgb_u8(234, 183, 64);

#[derive(Resource)]
pub(crate) struct ClassicArt {
    pub background: Handle<Image>,
    pub oak: Handle<Image>,
    pub heading: Handle<Font>,
    pub body: Handle<Font>,
}

#[derive(Component)]
pub(crate) struct MenuButton {
    pub primary: bool,
    pub focused: bool,
}

#[derive(Resource)]
pub(crate) struct FoliageMaterial(pub Handle<ColorMaterial>);

#[derive(Component)]
pub(crate) struct TitleBackdrop;

pub(crate) fn fit_backdrop(
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut backgrounds: Query<&mut Node, With<TitleBackdrop>>,
) {
    let Ok(window) = windows.single() else { return };
    let ratio = window.width() / window.height().max(1.0);
    let art_ratio = 16.0 / 9.0;
    for mut node in &mut backgrounds {
        node.width = percent(100.0 * (art_ratio / ratio).max(1.0));
        node.height = percent(100.0 * (ratio / art_ratio).max(1.0));
    }
}

pub(crate) fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut fonts: ResMut<Assets<Font>>,
) {
    let background = Image::from_buffer(
        include_bytes!("../../../assets/art/title-gardens.png"),
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        true,
        ImageSampler::linear(),
        RenderAssetUsages::RENDER_WORLD,
    )
    .expect("the embedded title artwork is a valid PNG");
    let heading =
        Font::try_from_bytes(include_bytes!("../../../assets/fonts/Gloock-Regular.ttf").to_vec())
            .expect("embedded heading font");
    let body =
        Font::try_from_bytes(include_bytes!("../../../assets/fonts/Barlow-Medium.ttf").to_vec())
            .expect("embedded interface font");
    commands.insert_resource(ClassicArt {
        background: images.add(background),
        oak: images.add(
            Image::from_buffer(
                include_bytes!("../../../assets/art/oak.png"),
                ImageType::Extension("png"),
                CompressedImageFormats::NONE,
                true,
                ImageSampler::linear(),
                RenderAssetUsages::RENDER_WORLD,
            )
            .expect("embedded oak sprite"),
        ),
        heading: fonts.add(heading),
        body: fonts.add(body),
    });
}

/// A retained mesh holds foliage sprites with depth anchored at each trunk.
pub(crate) fn foliage(park: &park_sim::Park, rotation: u8, mode: crate::world::ViewMode) -> Mesh {
    use bevy::mesh::{Indices, PrimitiveTopology};
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();
    if mode == crate::world::ViewMode::Surface {
        for (pos, seed) in crate::world::scenery_trees(park) {
            if seed.is_multiple_of(3) {
                continue;
            }
            let Some(base) =
                crate::world::surface_center(park, park_sim::SurfaceId::Terrain(pos), rotation)
            else {
                continue;
            };
            let scale = 0.85 + (seed % 6) as f32 * 0.055;
            let depth = crate::world::tree_depth(park, pos, rotation);
            let start = positions.len() as u32;
            positions.extend([
                [base.x - 30.0 * scale, base.y, depth],
                [base.x + 30.0 * scale, base.y, depth],
                [base.x + 30.0 * scale, base.y + 68.0 * scale, depth],
                [base.x - 30.0 * scale, base.y + 68.0 * scale, depth],
            ]);
            uvs.extend([[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
            indices.extend([start, start + 1, start + 2, start, start + 2, start + 3]);
        }
    }
    let count = positions.len();
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, vec![[0.0, 0.0, 1.0]; count])
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, vec![[1.0, 1.0, 1.0, 1.0]; count])
    .with_inserted_indices(Indices::U32(indices))
}

pub(crate) fn fonts(art: Option<Res<ClassicArt>>, mut text: Query<&mut TextFont, Added<TextFont>>) {
    let Some(art) = art else { return };
    for mut font in &mut text {
        if font.font == Handle::default() {
            font.font = art.body.clone();
        }
    }
}

pub(crate) fn button_feedback(
    mut buttons: Query<(
        &Interaction,
        &MenuButton,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    for (interaction, button, mut background, mut border) in &mut buttons {
        background.0 = match (*interaction, button.primary) {
            (Interaction::Pressed, _) => Color::srgb_u8(151, 104, 27),
            (Interaction::Hovered, true) => Color::srgb_u8(255, 210, 98),
            (Interaction::Hovered, _) => Color::srgb_u8(56, 110, 77),
            (_, true) => GOLD,
            _ => FOREST,
        };
        *border = BorderColor::all(if button.focused || *interaction != Interaction::None {
            CREAM
        } else {
            GOLD
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use park_sim::{Park, SceneryKind, TilePos};

    #[test]
    fn foliage_follows_saved_tree_positions_and_cannot_leak_into_cutaway() {
        let mut park = Park::new(12, 12);
        park.build_scenery(TilePos { x: 6, y: 6 }, SceneryKind::Tree)
            .unwrap();
        park.build_scenery(TilePos { x: 7, y: 6 }, SceneryKind::Tree)
            .unwrap();
        let saved = park.to_save_bytes().unwrap();
        for rotation in 0..4 {
            let mesh = foliage(&park, rotation, crate::world::ViewMode::Surface);
            let positions = mesh
                .attribute(Mesh::ATTRIBUTE_POSITION)
                .unwrap()
                .as_float3()
                .unwrap();
            assert!(!positions.is_empty());
            assert!(positions.iter().flatten().all(|p| p.is_finite()));
            assert_eq!(positions.len() % 4, 0);
            assert_eq!(
                foliage(&park, rotation, crate::world::ViewMode::Underground).count_vertices(),
                0
            );
        }
        assert_eq!(saved, park.to_save_bytes().unwrap());
    }
}
