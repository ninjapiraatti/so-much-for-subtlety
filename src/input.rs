use avian2d::{math::*, prelude::*};
use bevy::prelude::*;

use crate::weapons::Gun;
use crate::player::{
  CharacterControllerBundle,
  PlayerAssignments,
  PlayerAction,
};

pub fn gamepad_input(
  mut movement_event_writer: EventWriter<PlayerAction>,
  assignments: Res<PlayerAssignments>,
  gamepads: Query<(Entity, &Gamepad)>,
) {
  for (entity, gamepad) in &gamepads {
      let gid = entity.index();
      if let Some(entity) = assignments.players.get(&gid) {
          // Movement
          let x = gamepad.get(GamepadAxis::LeftStickX).unwrap_or(0.0);
          if x.abs() > 0.01 {
              movement_event_writer.send(PlayerAction::Move(*entity, x.into()));
          }
          let jump = gamepad.get(GamepadButton::South).unwrap_or(0.0);
          if jump > 0.1 {
              movement_event_writer.send(PlayerAction::Jump(*entity));
          }
          // Aiming
          let rx = gamepad.get(GamepadAxis::RightStickX).unwrap_or(0.0);
          let ry = gamepad.get(GamepadAxis::RightStickY).unwrap_or(0.0);
          if rx.abs() > 0.01 || ry.abs() > 0.01 {
              movement_event_writer.send(PlayerAction::Aim(*entity, rx, ry));
          }
          let fire = gamepad.get(GamepadButton::RightTrigger).unwrap_or(0.0);
          if fire > 0.1 {
              movement_event_writer.send(PlayerAction::Fire(*entity));
          }
      }
  }
}

pub fn keyboard_input(
  mut commands: Commands,
  mut movement_event_writer: EventWriter<PlayerAction>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
  mut assignments: ResMut<PlayerAssignments>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  let left = keyboard_input.any_pressed([KeyCode::KeyA, KeyCode::ArrowLeft]);
  let right = keyboard_input.any_pressed([KeyCode::KeyD, KeyCode::ArrowRight]);

  let horizontal = right as i8 - left as i8;
  let direction = horizontal as Scalar;

  if direction != 0.0 {
      // Assuming the player entity is the first one in the assignments
      if let Some(entity) = assignments.players.values().next() {
          movement_event_writer.send(PlayerAction::Move(*entity, direction));
      }
  }

  if keyboard_input.just_pressed(KeyCode::Space) {
      if let Some(entity) = assignments.players.values().next() {
          movement_event_writer.send(PlayerAction::Jump(*entity));
      }
  }

  if keyboard_input.just_pressed(KeyCode::KeyF) {
      if let Some(entity) = assignments.players.values().next() {
          movement_event_writer.send(PlayerAction::Aim(*entity, 0.5, 0.5));
          movement_event_writer.send(PlayerAction::Fire(*entity));
      }
  }

  if keyboard_input.just_pressed(KeyCode::Enter) {
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
              Friction::new(0.4).with_dynamic_coefficient(0.6).with_static_coefficient(0.6),
              //Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
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
      assignments.players.insert(5, entity);
  }
}