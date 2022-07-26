#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // prevent showing terminal on windows in release

#[macro_use]
extern crate rust_i18n;
i18n!("locales");

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum MenuWindow {
    Main,
    Settings,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Splash,
    Menu(MenuWindow),
    InGame,
    Paused,
}

// bevy_embasset::assets!(
//     pub enum UIAssets {
//         FiraSansFont = "fonts/FiraSans-Bold.ttf",
//         RobotoFont = "fonts/roboto.kayak_font",
//     },
//     pub struct UIAssetsIo {
//         root = "../assets/"
//     }
// );

// // TODO: autodefault
// #[bevy_main]
// fn main() {
//     use AppState::*;
//     use MenuWindow::*;

//     dotenv::dotenv().ok();

//     // TODO: Add i18n
//     App::new()
//         .insert_resource(WindowDescriptor {
//             width: 1270.0,
//             height: 720.0,
//             mode: if cfg!(debug_assertions) {
//                 WindowMode::Windowed
//             } else {
//                 WindowMode::BorderlessFullscreen
//             },
//             title: t!("title"),
//             ..default()
//         })
//         // TODO: Create own embedded resources registry
//         // .add_plugins_with(DefaultPlugins, |group| {
//         //     group.add_before::<AssetPlugin, _>(EmbassetPlugin::new(|io| {
//         //         io.add_handler(PlayerAssetsIo::new().into());
//         //         io.add_handler(DiscoAssetsIo::new().into());
//         //         io.add_handler(UIAssetsIo::new().into());
//         //         io.add_handler(MapAssetsIo::new().into());
//         //     }))
//         // })
//         .add_plugins(DefaultPlugins)
//         .add_plugin(WorldInspectorPlugin::new())
//         .insert_resource(WorldInspectorParams {
//             despawnable_entities: true,
//             ..Default::default()
//         })
//         // ldtk
//         .add_plugin(LdtkPlugin)
//         .add_plugin(PhysicsPlugin::default())
//         .insert_resource(LevelSelection::Uid(0))
//         .insert_resource(LdtkSettings {
//             level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
//                 load_level_neighbors: true,
//             },
//             set_clear_color: SetClearColor::FromLevelBackground,
//             ..Default::default()
//         })
//         .add_startup_system(systems::setup)
//         .add_system(systems::pause_physics_during_load)
//         .add_system(systems::spawn_wall_collision)
//         .add_system(systems::movement)
//         .add_system(systems::detect_climb_range)
//         .add_system(systems::ignore_gravity_if_climbing)
//         .add_system(systems::patrol)
//         .add_system(systems::camera_fit_inside_current_level)
//         .add_system(systems::update_level_selection)
//         .add_system(systems::dbg_player_items)
//         .add_system(systems::spawn_ground_sensor)
//         .add_system(systems::ground_detection)
//         .register_ldtk_int_cell::<components::WallBundle>(1)
//         // .register_ldtk_int_cell::<components::LadderBundle>(2)
//         // .register_ldtk_int_cell::<components::WallBundle>(3)
//         .register_ldtk_entity::<components::PlayerBundle>("Player")
//         .register_ldtk_entity::<components::MobBundle>("Mob")
//         .register_ldtk_entity::<components::ChestBundle>("Chest")
//         // discord presence
//         // .add_plugin(RPCPlugin(RPCConfig {
//         //     app_id: 996917115120521266,
//         //     show_time: true,
//         // }))
//         // game settings
//         // .add_state(Menu(Main)) // TODO: first state must be Splash
//         // .bind_ui::<MenuUI>(AppState::Menu(Main))
//         // .add_plugins(GamePlugins)
//         .run()
// }

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

trait BindUI {
    fn bind_ui<T: UI>(&mut self, state: impl StateData) -> &mut Self;
}

impl BindUI for App {
    fn bind_ui<T: UI>(&mut self, state: impl StateData) -> &mut Self {
        <T as UI>::bind(state, self);
        self
    }
}

trait UI {
    fn bind<T: StateData>(state: T, app: &mut App);
}

struct MenuUI {
    button_entity: Entity,
}

impl UI for MenuUI {
    fn bind<T: StateData>(state: T, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(state.to_owned()).with_system(Self::setup))
            .add_system_set(SystemSet::on_update(state.to_owned()).with_system(Self::update))
            .add_system_set(SystemSet::on_exit(state.to_owned()).with_system(Self::clean));
    }
}

impl MenuUI {
    fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        commands.spawn_bundle(UiCameraBundle::default());
        let button_entity = commands
            .spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Px(150.0), Val::Px(65.0)),
                    margin: Rect::all(Val::Auto),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: NORMAL_BUTTON.into(),
                ..default()
            })
            .insert(Name::new("Button"))
            .with_children(|parent| {
                parent
                    .spawn_bundle(TextBundle {
                        text: Text::with_section(
                            // TODO: i18n
                            "Play",
                            TextStyle {
                                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                                font_size: 40.0,
                                color: Color::rgb(0.9, 0.9, 0.9),
                            },
                            default(),
                        ),
                        ..default()
                    })
                    .insert(Name::new("Text"));
            })
            .id();
        commands.insert_resource(Self { button_entity });
    }

    fn update(
        mut state: ResMut<State<AppState>>,
        mut interaction_query: Query<
            (&Interaction, &mut UiColor),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color) in interaction_query.iter_mut() {
            match *interaction {
                Interaction::Clicked => {
                    *color = PRESSED_BUTTON.into();
                    state.set(AppState::InGame).unwrap();
                }
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                }
            }
        }
    }

    fn clean(mut commands: Commands, menu_data: Res<Self>) {
        commands.entity(menu_data.button_entity).despawn_recursive();
    }
}

mod plugins;

use bevy::{ecs::schedule::StateData, prelude::*, window::WindowMode};
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::{WorldInspectorParams, WorldInspectorPlugin};
use plugins::{FpsMeterPlugin, FullscreenTogglePlugin, SpriteSortingPlugin};

use heron::prelude::*;

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
        .bind_ui::<MenuUI>(AppState::Menu(MenuWindow::Main))
        .add_plugin(SpriteSortingPlugin)
        .add_system_set(
            SystemSet::on_enter(AppState::Menu(MenuWindow::Main)).with_system(systems::setup),
        )
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
        .run();
}
