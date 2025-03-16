//! A basic implementation of a character controller for a dynamic rigid body.
//!
//! This showcases the following:
//!
//! - Basic directional movement and jumping
//! - Support for both keyboard and gamepad input
//! - A configurable maximum slope angle for jumping
//!
//! The character controller logic is contained within the `plugin` module.
//!
//! For a kinematic character controller, see the `kinematic_character_2d` example.

use avian2d::{math::*, prelude::*};
use bevy::prelude::*;

mod game;
mod input;
mod player;
mod weapons;
mod items;

use player::{
    CharacterControllerPlugin,
    PlayerAssignments,
};

use game::setup;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            // Add physics plugins and specify a units-per-meter scaling factor, 1 meter = 20 pixels.
            // The unit allows the engine to tune its parameters for the scale of the world, improving stability.
            PhysicsPlugins::default().with_length_unit(20.0),
            CharacterControllerPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(PlayerAssignments::default())
        .insert_resource(Gravity(Vector::NEG_Y * 1000.0))
        .add_systems(Startup, setup)
        //.add_systems(Update, gamepad_system)
        .run();
}
