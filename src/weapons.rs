use bevy::prelude::*;

#[derive(Component)]
pub struct Gun;

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub lifetime: f32, // Time before the projectile is destroyed
}