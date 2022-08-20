#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // prevent showing terminal on windows in release

// #![warn(missing_docs)]

// TODO: Load .dlg files by AssetServer
// TODO: Load characters from LDtk
// TODO: Make a StateStorage with states of world and characters
// TODO: Flags and values

#[macro_use]
extern crate rust_i18n;
i18n!("locales");

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MenuWindow {
    Main,
    Settings,
    DialogTest,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Splash,
    Menu(MenuWindow),
    InGame,
    Paused,
}

mod bevy_dlg;
mod plugins;
pub mod ui;

use bevy::{prelude::*, window::WindowMode};
use bevy_dlg::DialogAsset;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use plugins::{FpsMeterPlugin, FullscreenTogglePlugin, SpriteSortingPlugin};

use crate::ui::BindUI;
use heron::prelude::*;
use ui::{dialog::DialogUI, menu::MenuUI};

mod components;
mod systems;

fn main() {
    dotenv::dotenv().ok();

    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            mode: if cfg!(debug_assertions) {
                WindowMode::Windowed
            } else {
                WindowMode::BorderlessFullscreen
            },
            title: t!("title"),
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LdtkPlugin)
        .add_plugin(PhysicsPlugin::default())
        .add_plugin(FpsMeterPlugin)
        .add_plugin(FullscreenTogglePlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        // inspector
        .add_plugin(WorldInspectorPlugin::new())
        .insert_resource(WorldInspectorParams {
            despawnable_entities: true,
            highlight_changes: true,
            ..Default::default()
        })
        // ldtk
        // TODO: Раскидать системы по файлам, чтоб красиво было
        .insert_resource(LevelSelection::Uid(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .add_state(AppState::Menu(MenuWindow::Main))
        .bind_ui::<DialogUI>(AppState::Menu(MenuWindow::DialogTest))
        .bind_ui::<MenuUI>(AppState::Menu(MenuWindow::Main))
        .add_plugin(SpriteSortingPlugin)
        .add_system_set(
            SystemSet::on_enter(AppState::Menu(MenuWindow::Main)).with_system(systems::setup),
        )
        // .add_system_set(
        //     SystemSet::on_exit(AppState::Menu(MenuWindow::Main)).with_system(systems::clear),
        // )
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(systems::pause_physics_during_load)
                .with_system(systems::spawn_wall_collision)
                .with_system(systems::read_player_input)
                .with_system(systems::player_movement)
                .with_system(systems::player_animation)
                .with_system(systems::camera_fit_inside_current_level)
                .with_system(systems::update_level_selection),
        )
        .register_ldtk_int_cell::<components::WallBundle>(1)
        .register_ldtk_entity::<components::PlayerBundle>("Player")
        .register_default_ldtk_entity::<components::NonPlayerCharacter>()
        // dlg
        .add_asset::<DialogAsset>()
        .init_asset_loader::<DialogAsset>()
        .add_startup_system(load_dialog)
        .add_system(check_assets_ready)
        .run();
}

fn load_dialog(asset_server: Res<AssetServer>, mut commands: Commands) {
    let dialog_handle: Handle<DialogAsset> = asset_server.load("dialogues/ru/verres-home/test.dlg");

    commands.insert_resource(dialog_handle);
}

fn check_assets_ready(
    assets: Res<Assets<DialogAsset>>,
    dialog_handle: Res<Handle<DialogAsset>>,
    mut loaded: Local<bool>,
) {
    if *loaded {
        return;
    }

    if let Some(dialog) = assets.get(dialog_handle.as_ref()) {
        info!("{:#?}", dialog.0);
        *loaded = true;
    }
}
