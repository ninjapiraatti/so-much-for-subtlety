use avian2d::{math::*, prelude::*};
use bevy::{ecs::query::Has, prelude::*};
use std::collections::HashMap;

#[derive(Component)]
pub struct Gun;

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub lifetime: f32, // Time before the projectile is destroyed
}