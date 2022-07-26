#![allow(clippy::forget_non_drop)]

mod wall;
pub use wall::*;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use heron::prelude::*;

use crate::systems::{MovementController, PlayerAnimationTimer};

use crate::plugins::SpriteSort;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: CollisionShape,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: RotationConstraints,
    pub physic_material: PhysicMaterial,
}

impl From<EntityInstance> for ColliderBundle {
    fn from(entity_instance: EntityInstance) -> ColliderBundle {
        let rotation_constraints = RotationConstraints::lock();

        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(6., 14., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                ..Default::default()
            },
            "Mob" => ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(5., 5., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::KinematicVelocityBased,
                rotation_constraints,
                ..Default::default()
            },
            "Chest" => ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(8., 8., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                physic_material: PhysicMaterial {
                    friction: 0.5,
                    density: 15.0,
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

impl From<EntityInstance> for SpriteSort {
    fn from(entity_instance: EntityInstance) -> Self {
        match entity_instance.identifier.as_ref() {
            "Player" => SpriteSort {
                layer: 0,
                z_index: 1,
                y_sort: true,
            },
            _ => SpriteSort::default(),
        }
    }
}

impl From<IntGridCell> for ColliderBundle {
    fn from(cell: IntGridCell) -> ColliderBundle {
        let rotation_constraints = RotationConstraints::lock();

        if cell.value == 2 {
            ColliderBundle {
                collider: CollisionShape::Cuboid {
                    half_extends: Vec3::new(8., 8., 0.),
                    border_radius: None,
                },
                rigid_body: RigidBody::Sensor,
                rotation_constraints,
                ..Default::default()
            }
        } else {
            ColliderBundle::default()
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet_bundle("images/player.png", 32.0, 32.0, 3, 4, 0.0, 1)]
    #[bundle]
    pub sprite_bundle: SpriteSheetBundle,

    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,

    pub player: Player,

    pub controller: MovementController,
    pub animator: PlayerAnimationTimer,

    #[from_entity_instance]
    pub sort: SpriteSort,

    #[worldly]
    pub worldly: Worldly,

    #[from_entity_instance]
    entity_instance: EntityInstance,
}
