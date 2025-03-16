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
  // A cube to move around
  commands.spawn((
      Sprite {
          color: Color::srgb(0.0, 0.4, 0.7),
          custom_size: Some(Vec2::new(30.0, 30.0)),
          ..default()
      },
      Transform::from_xyz(50.0, -100.0, 0.0),
      RigidBody::Dynamic,
      Collider::rectangle(30.0, 30.0),
  ));

  // Platforms
  commands.spawn((
      Sprite {
          color: Color::srgb(0.7, 0.7, 0.8),
          custom_size: Some(Vec2::new(1100.0, 50.0)),
          ..default()
      },
      Transform::from_xyz(0.0, -175.0, 0.0),
      RigidBody::Static,
      Collider::rectangle(1100.0, 50.0),
  ));
  commands.spawn((
      Sprite {
          color: Color::srgb(0.7, 0.7, 0.8),
          custom_size: Some(Vec2::new(300.0, 25.0)),
          ..default()
      },
      Transform::from_xyz(175.0, -35.0, 0.0),
      RigidBody::Static,
      Collider::rectangle(300.0, 25.0),
  ));
  commands.spawn((
      Sprite {
          color: Color::srgb(0.7, 0.7, 0.8),
          custom_size: Some(Vec2::new(300.0, 25.0)),
          ..default()
      },
      Transform::from_xyz(-175.0, 0.0, 0.0),
      RigidBody::Static,
      Collider::rectangle(300.0, 25.0),
  ));
  commands.spawn((
      Sprite {
          color: Color::srgb(0.7, 0.7, 0.8),
          custom_size: Some(Vec2::new(150.0, 80.0)),
          ..default()
      },
      Transform::from_xyz(475.0, -110.0, 0.0),
      RigidBody::Static,
      Collider::rectangle(150.0, 80.0),
  ));
  commands.spawn((
      Sprite {
          color: Color::srgb(0.7, 0.7, 0.8),
          custom_size: Some(Vec2::new(150.0, 80.0)),
          ..default()
      },
      Transform::from_xyz(-475.0, -110.0, 0.0),
      RigidBody::Static,
      Collider::rectangle(150.0, 80.0),
  ));

  // Ramps

  let mut ramp_mesh = Mesh::new(
      PrimitiveTopology::TriangleList,
      RenderAssetUsages::default(),
  );

  ramp_mesh.insert_attribute(
      Mesh::ATTRIBUTE_POSITION,
      vec![[-125.0, 80.0, 0.0], [-125.0, 0.0, 0.0], [125.0, 0.0, 0.0]],
  );

  let ramp_collider = Collider::triangle(
      Vector::new(-125.0, 80.0),
      Vector::NEG_X * 125.0,
      Vector::X * 125.0,
  );

  commands.spawn((
      Mesh2d(meshes.add(ramp_mesh)),
      MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 0.5))),
      Transform::from_xyz(-275.0, -150.0, 0.0),
      RigidBody::Static,
      ramp_collider,
  ));

  let mut ramp_mesh = Mesh::new(
      PrimitiveTopology::TriangleList,
      RenderAssetUsages::default(),
  );

  ramp_mesh.insert_attribute(
      Mesh::ATTRIBUTE_POSITION,
      vec![[20.0, -40.0, 0.0], [20.0, 40.0, 0.0], [-20.0, -40.0, 0.0]],
  );

  let ramp_collider = Collider::triangle(
      Vector::new(20.0, -40.0),
      Vector::new(20.0, 40.0),
      Vector::new(-20.0, -40.0),
  );

  commands.spawn((
      Mesh2d(meshes.add(ramp_mesh)),
      MeshMaterial2d(materials.add(Color::srgb(0.4, 0.4, 0.5))),
      Transform::from_xyz(380.0, -110.0, 0.0),
      RigidBody::Static,
      ramp_collider,
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
                      1200.0,
                      Quat::IDENTITY,
                      (30.0 as Scalar).to_radians(),
                      0.0,
                  ),
                  Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
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