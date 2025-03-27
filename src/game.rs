use avian2d::{math::*, prelude::*};
use bevy::{
  prelude::*,
  render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
};

use crate::player::{
  CharacterControllerBundle,
  PlayerAssignments,
};

use crate::weapons::{ Gun, Projectile };

pub fn setup(
  mut commands: Commands,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  // A cube to move around (keep this)
  commands.spawn((
      Sprite {
          color: Color::srgb(0.0, 0.4, 0.7),
          custom_size: Some(Vec2::new(30.0, 30.0)),
          ..default()
      },
      Transform::from_xyz(50.0, -100.0, 0.0),
      RigidBody::Dynamic,
      Mass(5.0),
      Collider::rectangle(30.0, 30.0),
      //Friction::new(0.4).with_dynamic_coefficient(0.6).with_static_coefficient(0.6)
  ));

  // Planet surface (large circle)
  let planet_radius = 5000.0; // Large radius so only part is visible

  // Create a circle mesh with many vertices to make it smooth
  let segments = 256;
  let mut circle_mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

  // Vertices for the circle
  let mut positions = Vec::with_capacity(segments * 3);

  // Create a filled circle using triangles from center
  for i in 0..segments {
    let angle1 = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
    let angle2 = 2.0 * std::f32::consts::PI * ((i + 1) as f32) / (segments as f32);

    // Center point
    positions.push([0.0, 0.0, 0.0]);
    // First outer point
    positions.push([
        planet_radius * angle1.cos(), 
        planet_radius * angle1.sin(), 
        0.0
    ]);
    // Second outer point
    positions.push([
        planet_radius * angle2.cos(), 
        planet_radius * angle2.sin(), 
        0.0
    ]);
  }

  circle_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

  // Create a circle collider (assuming this exists in your physics system)
  let circle_collider = Collider::circle(planet_radius);
  let mut polygon_vertices = Vec::with_capacity(segments);
  for i in 0..segments {
    let angle = 2.0 * std::f32::consts::PI * (i as f32) / (segments as f32);
    polygon_vertices.push(Vec2::new(
      planet_radius * angle.cos(),
      planet_radius * angle.sin()
    ));
  }

  let polygon_collider = Collider::polyline(polygon_vertices, None);
  
  commands.spawn((
      Mesh2d(meshes.add(circle_mesh)),
      MeshMaterial2d(materials.add(Color::srgb(0.5, 0.8, 0.5))),
      // Position it so only the top part is visible (like a planet surface)
      Transform::from_xyz(0.0, -5200.0, 0.0),
      RigidBody::Kinematic,
      circle_collider,
      AngularVelocity(0.01),
      //Friction::new(0.4).with_dynamic_coefficient(0.6).with_static_coefficient(0.6)
  ));

  // Camera
  commands.spawn(Camera2d);
}

pub fn spawn_character(
  mut commands: Commands,
  mut assignments: ResMut<PlayerAssignments>,
  gamepads: Query<(Entity, &Gamepad)>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  for (entity, gamepad) in &gamepads {
      let start_button = gamepad.get(GamepadButton::South).unwrap_or(0.0);
      let gid = entity.index();
      if start_button > 0.1 && !assignments.players.contains_key(&gid) {
          let entity = commands
              .spawn((
                  Mesh2d(meshes.add(Capsule2d::new(12.5, 20.0))),
                  MeshMaterial2d(materials.add(Color::srgb(0.9, 0.1, 0.1))),
                  Transform::from_xyz(50.0, -100.0, 0.0),
                  CharacterControllerBundle::new(Collider::capsule(12.5, 20.0)).with_movement(
                      1250.0,
                      0.92,
                      800.0,
                      Quat::IDENTITY,
                      (30.0 as Scalar).to_radians(),
                      0.0,
                  ),
                  //Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
                  Friction::new(0.8).with_dynamic_coefficient(0.8).with_static_coefficient(0.8),
                  Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
                  ColliderDensity(2.0),
                  GravityScale(1.5),
              ))
              .with_children(|parent| {
                  parent.spawn((
                      Sprite {
                          color: Color::srgb(0.2, 0.2, 0.2),
                          custom_size: Some(Vec2::new(10.0, 40.0)),
                          anchor: bevy::sprite::Anchor::TopCenter,
                          ..default()
                      },
                      Transform::default(),
                      Gun,
                  ));
              })
              .id();
          assignments.players.insert(gid, entity);
      }
  }
}

pub fn move_objects(
  time: Res<Time>,
  mut commands: Commands,
  mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
) {
  for (entity, mut transform, mut projectile) in query.iter_mut() {
      // Update position based on velocity
      let delta_time = time.delta_secs_f64().adjust_precision();
      transform.translation += projectile.velocity.extend(0.0) * delta_time;

      if projectile.lifetime > 0.0 {
          projectile.lifetime -= delta_time;
      } else {
          // Remove the projectile after its lifetime expires
          commands.entity(entity).despawn();
      }
  }
}