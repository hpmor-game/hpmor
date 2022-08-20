use bevy::{ecs::schedule::StateData, prelude::*};

pub mod dialog;
pub mod menu;

pub trait BindUI {
    fn bind_ui<T: UI>(&mut self, state: impl StateData) -> &mut Self;
}

impl BindUI for App {
    fn bind_ui<T: UI>(&mut self, state: impl StateData) -> &mut Self {
        <T as UI>::bind(state, self);
        self
    }
}

pub trait UI {
    fn bind<T: StateData>(state: T, app: &mut App);
}
