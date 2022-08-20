use super::UI;
use crate::{bevy_dlg::DialogAsset, AppState, MenuWindow};
use bevy::{ecs::schedule::StateData, prelude::*};
use bevy_egui::{egui::Window as EguiWindow, egui::*, EguiContext};
use dlg::{parser::Menu, prelude::*};

pub struct DialogUI;

impl UI for DialogUI {
    fn bind<T: StateData>(state: T, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(state.to_owned()).with_system(Self::setup))
            .add_system_set(SystemSet::on_update(state.to_owned()).with_system(Self::update))
            .add_system_set(SystemSet::on_exit(state).with_system(Self::clean));
    }
}

impl DialogUI {
    fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let width = 80.0;

        //         let layout = commands
        //             .spawn_bundle(NodeBundle {
        //                 style: Style {
        //                     align_content: AlignContent::Center,
        //                     justify_content: JustifyContent::Center,
        //                     flex_wrap: FlexWrap::Wrap,
        //                     position_type: PositionType::Absolute,
        //                     position: Rect {
        //                         left: Val::Percent((100.0 - width) / 2.0),
        //                         right: Val::Auto,
        //                         top: Val::Auto,
        //                         bottom: Val::Px(16.0),
        //                     },
        //                     size: Size::new(Val::Percent(width), Val::Percent(30.0)),
        //                     overflow: Overflow::Hidden,
        //                     ..default()
        //                 },
        //                 color: Color::rgb(0.15, 0.15, 0.15).into(),
        //                 ..default()
        //             })
        //             .with_children(|parent| {
        //                 parent.spawn_bundle(NodeBundle {
        //                     style: Style {
        //                         flex_direction: FlexDirection::ColumnReverse,
        //                         ..default()
        //                     },
        //                     color: Color::NONE.into(),
        //                     ..default()
        //                 }).with_children(|parent| {
        //                     parent.spawn_bundle(TextBundle {
        //                         text: Text::with_section(
        //                             // Accepts a `String` or any type that converts into a `String`, such as `&str`
        //                             "Narrator",
        //                             TextStyle {
        //                                 font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        //                                 font_size: 32.0,
        //                                 color: Color::WHITE,
        //                             },
        //                             TextAlignment {
        //                                 horizontal: HorizontalAlign::Left,
        //                                 vertical: VerticalAlign::Top,
        //                             },
        //                         ),
        //                         style: default(),
        //                         ..default()
        //                     });

        //                     parent.spawn_bundle(TextBundle {
        //                         text: Text::with_section(
        //                             "
        // Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
        // Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
        // ".trim(),
        //                             TextStyle {
        //                                 font: asset_server.load("fonts/FiraSans-Bold.ttf"),
        //                                 font_size: 24.0,
        //                                 color: Color::WHITE,
        //                             },
        //                             TextAlignment {
        //                                 vertical: VerticalAlign::Top,
        //                                 horizontal: HorizontalAlign::Left,
        //                             },
        //                         ),
        //                         style: Style {
        //                             flex_wrap: FlexWrap::Wrap,
        //                             max_size: Size {
        //                                 width: Val::Px(800.0),
        //                                 height: Val::Percent(100.0),
        //                             },
        //                             padding: Rect {
        //                                 left: Val::Px(10.0),
        //                                 top: Val::Px(10.0),
        //                                 right: Val::Px(10.0),
        //                                 bottom: Val::Px(10.0),
        //                             },
        //                             ..default()
        //                         },
        //                         ..default()
        //                     });
        //                 });
        //             })
        //             .id();

        //         commands.insert_resource(Self { layout });
    }

    fn update(
        mut egui_context: ResMut<EguiContext>,
        assets: Res<Assets<DialogAsset>>,
        dialog_handle: Res<Handle<DialogAsset>>,
        windows: Res<Windows>,
        mut cursor: Local<Cursor>,
        mut state: ResMut<bevy::ecs::schedule::State<AppState>>,
    ) {
        let dialog = assets.get(dialog_handle.as_ref());

        if let Some(DialogAsset(dialog)) = dialog {
            let line = dialog.get_line_by_cursor(&cursor);

            let window = windows.primary();

            let padding: f32 = 40.0;
            let (dialog_width, dialog_height) =
                (window.width() - padding * 2.0, window.height() / 3.0);

            let frame = Frame::window(&default()).shadow(epaint::Shadow {
                extrusion: 0.0,
                color: Color32::TRANSPARENT,
            });

            EguiWindow::new("dialog")
                .anchor(Align2::CENTER_BOTTOM, vec2(0.0, -(padding - 5.0))) // 5 is magic number
                .fixed_size(vec2(dialog_width, dialog_height))
                .resizable(false)
                .title_bar(false)
                .frame(frame)
                .show(egui_context.ctx_mut(), |ui| {
                    if let Some(line) = line {
                        match line {
                            Line::Phrase { speaker, lines } => {
                                let name = if let Speaker::Character(Alias(name), _) = speaker {
                                    name.to_string()
                                } else {
                                    "narrator".to_string()
                                };

                                ui.heading(name);
                                ui.vertical_centered(|ui| {
                                    ui.separator();
                                });
                                ui.label(lines.get(cursor.phrase_index()).unwrap());

                                ui.with_layout(
                                    Layout::right_to_left().with_cross_align(Align::Min),
                                    |ui| {
                                        if ui.button("Next").clicked() {
                                            if cursor.phrase_index() < lines.len() - 1 {
                                                cursor.next_phrase_index();
                                            } else {
                                                cursor.next_line_index();
                                            }
                                        }
                                    },
                                );
                            }
                            Line::Menu(Menu { title, options }) => {
                                if let Some(title) = title {
                                    ui.heading(title);
                                    ui.vertical_centered(|ui| {
                                        ui.separator();
                                    });
                                }

                                for opt in options {
                                    let title = opt.title.clone().unwrap();
                                    if ui.button(title).clicked() {
                                        let section = &opt.args[1..]; // FIXME: remove hash from link
                                        cursor.set_section(section.to_string());
                                    };
                                }
                            }
                        }
                    } else {
                        state.set(AppState::Menu(MenuWindow::Main)).unwrap();
                    }
                });
        }
    }

    fn clean() {}
}
