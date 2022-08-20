use bevy::{ecs::schedule::StateData, prelude::*};

use crate::{AppState, MenuWindow};

use super::UI;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub struct MenuUI {
    layout: Entity,
}

impl UI for MenuUI {
    fn bind<T: StateData>(state: T, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(state.to_owned()).with_system(Self::setup))
            .add_system_set(SystemSet::on_update(state.to_owned()).with_system(Self::update))
            .add_system_set(SystemSet::on_exit(state).with_system(Self::clean));
    }
}

// TODO: Make fn params queryable, like in systems. I have no idea how to do this 🗿
#[derive(Component)]
pub struct OnClick(Box<dyn FnMut(&mut ResMut<State<AppState>>) + Send + Sync>);

impl OnClick {
    pub fn new(on_click: Box<dyn FnMut(&mut ResMut<State<AppState>>) + Send + Sync>) -> Self {
        Self(on_click)
    }
}

impl MenuUI {
    fn create_button(
        text: &str,
        asset_server: &Res<AssetServer>,
        builder: &mut ChildBuilder,
        on_click: impl FnMut(&mut ResMut<State<AppState>>) + Send + Sync + 'static,
    ) -> Entity {
        builder
            .spawn_bundle(ButtonBundle {
                style: Style {
                    size: Size::new(Val::Auto, Val::Auto),
                    margin: Rect::all(Val::Auto),
                    padding: Rect {
                        left: Val::Px(20.0),
                        top: Val::Px(10.0),
                        right: Val::Px(20.0),
                        bottom: Val::Px(10.0),
                    },
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
                            text,
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
            .insert(OnClick::new(Box::new(on_click)))
            .id()
    }

    fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
        let layout = commands
            .spawn_bundle(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::ColumnReverse,
                    align_self: AlignSelf::Center,
                    size: Size::new(Val::Percent(100.0), Val::Px(150.0)),
                    overflow: Overflow::Hidden,
                    ..default()
                },
                color: Color::NONE.into(),
                ..default()
            })
            .with_children(|parent| {
                Self::create_button("Play", &asset_server, parent, |state| {
                    state.set(AppState::InGame).unwrap();
                });
                Self::create_button("Test Dialog", &asset_server, parent, |state| {
                    state.set(AppState::Menu(MenuWindow::DialogTest)).unwrap();
                });
            })
            .id();

        commands.insert_resource(Self { layout });
    }

    fn update(
        mut state: ResMut<State<AppState>>,
        mut interaction_query: Query<
            (&Interaction, &mut UiColor, &mut OnClick),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color, mut on_click) in interaction_query.iter_mut() {
            match *interaction {
                Interaction::Clicked => {
                    *color = PRESSED_BUTTON.into();
                    on_click.0(&mut state);
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

    fn clean(mut commands: Commands, menu: Res<Self>) {
        commands.entity(menu.layout).despawn_recursive();
    }
}
