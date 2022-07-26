use bevy::{prelude::*, window::WindowMode::*};

pub struct FullscreenTogglePlugin;

impl Plugin for FullscreenTogglePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(toggle_fullscreen)
            .init_resource::<FullscreenToggleButton>();
    }
}

pub struct FullscreenToggleButton(KeyCode);

impl Default for FullscreenToggleButton {
    fn default() -> Self {
        Self(KeyCode::F11)
    }
}

fn toggle_fullscreen(
    input: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
    code: Res<FullscreenToggleButton>,
) {
    let window = windows.primary_mut();
    if input.just_pressed(code.0) {
        window.set_mode(
            vec![BorderlessFullscreen, Windowed][(window.mode() == BorderlessFullscreen) as usize],
        );
    }
}
