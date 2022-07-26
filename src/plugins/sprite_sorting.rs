use bevy::prelude::*;
use bevy_inspector_egui::{Inspectable, RegisterInspectable};

pub struct SpriteSortingPlugin;

impl SpriteSortingPlugin {
    fn sort_sprites(
        mut sprites_query: Query<(&mut Transform, &SpriteSort)>,
        windows: Res<Windows>,
    ) {
        let window = windows.primary();
        let _screen_height = window.height();

        sprites_query.for_each_mut(|(mut transform, sort)| {
            transform.translation.z =
                sort.z_index as f32 * 10.0 - (transform.translation.y) / 1000.0;
        });
    }
}

impl Plugin for SpriteSortingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(Self::sort_sprites)
            .register_inspectable::<SpriteSort>();
    }
}

#[derive(Clone, Component, Inspectable)]
pub struct SpriteSort {
    pub layer: u8,
    pub z_index: i8,
    pub y_sort: bool,
}

impl SpriteSort {
    #[allow(dead_code)]
    fn new(layer: u8, z_index: i8, y_sort: bool) -> Self {
        Self {
            layer,
            z_index,
            y_sort,
        }
    }

    #[allow(dead_code)]
    fn layer(layer: u8) -> Self {
        Self {
            layer,
            ..Default::default()
        }
    }
}

impl Default for SpriteSort {
    fn default() -> Self {
        Self {
            layer: 0,
            z_index: 0,
            y_sort: true,
        }
    }
}
