use avian2d::{math::*, prelude::*};
use bevy::{ecs::query::Has, prelude::*};
use std::collections::HashMap;

pub struct CharacterControllerPlugin;
use crate::input::{gamepad_input, keyboard_input};
use crate::weapons::{Gun, Projectile};
use crate::game::{spawn_character, move_objects};

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerAction>().add_systems(
            Update,
            (
                keyboard_input,
                gamepad_input,
                update_grounded,
                apply_movement_damping,
                apply_aim_to_gun,
                move_objects,
                spawn_character,
                movement,
            )
                .chain(),
        );
    }
}

// An event sent for a movement input action.
#[derive(Event)]
pub enum PlayerAction {
    Move(Entity, Scalar),
    Jump(Entity),
    Aim(Entity, Scalar, Scalar),
    Fire(Entity),
}

#[derive(Resource, Default)]
pub struct PlayerAssignments {
    // Map each Gamepad to its spawned character
    pub players: HashMap<u32, Entity>,
}

// A marker component indicating that an entity is using a character controller.
#[derive(Component)]
pub struct CharacterController;

// A marker component indicating that an entity is on the ground.
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct Grounded;
// The acceleration used for character movement.
#[derive(Component)]
pub struct MovementAcceleration(Scalar);

// The damping factor used for slowing down movement.
#[derive(Component)]
pub struct MovementDampingFactor(Scalar);

// The strength of a jump.
#[derive(Component)]
pub struct JumpImpulse(Scalar);

#[derive(Component)]
pub struct FireImpulse(Scalar);

// The maximum angle a slope can have for a character controller
// to be able to climb and jump. If the slope is steeper than this angle,
// the character will slide down.

#[derive(Component)]
pub struct AimRotation(Quat);

#[derive(Component)]
pub struct MaxSlopeAngle(Scalar);

// A bundle that contains the components needed for a basic
// kinematic character controller.
#[derive(Bundle)]
pub struct CharacterControllerBundle {
    character_controller: CharacterController,
    rigid_body: RigidBody,
    collider: Collider,
    ground_caster: ShapeCaster,
    locked_axes: LockedAxes,
    movement: MovementBundle,
}

// A bundle that contains components for character movement.
#[derive(Bundle)]
pub struct MovementBundle {
    acceleration: MovementAcceleration,
    damping: MovementDampingFactor,
    jump_impulse: JumpImpulse,
    aiming: AimRotation,
    max_slope_angle: MaxSlopeAngle,
    fire_impulse: FireImpulse,
}

impl MovementBundle {
    pub const fn new(
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        aiming: Quat,
        max_slope_angle: Scalar,
        fire_impulse: Scalar,
    ) -> Self {
        Self {
            acceleration: MovementAcceleration(acceleration),
            damping: MovementDampingFactor(damping),
            jump_impulse: JumpImpulse(jump_impulse),
            aiming: AimRotation(aiming),
            max_slope_angle: MaxSlopeAngle(max_slope_angle),
            fire_impulse: FireImpulse(fire_impulse),
        }
    }
}

impl Default for MovementBundle {
    fn default() -> Self {
        Self::new(30.0, 0.9, 200.0, Quat::IDENTITY, PI * 0.45, 0.0)
    }
}

impl CharacterControllerBundle {
    pub fn new(collider: Collider) -> Self {
        // Create shape caster as a slightly smaller version of collider
        let mut caster_shape = collider.clone();
        caster_shape.set_scale(Vector::ONE * 0.99, 10);

        Self {
            character_controller: CharacterController,
            rigid_body: RigidBody::Dynamic,
            collider,
            ground_caster: ShapeCaster::new(caster_shape, Vector::ZERO, 0.0, Dir2::NEG_Y)
                .with_max_distance(10.0),
            locked_axes: LockedAxes::ROTATION_LOCKED,
            movement: MovementBundle::default(),
        }
    }

    pub fn with_movement(
        mut self,
        acceleration: Scalar,
        damping: Scalar,
        jump_impulse: Scalar,
        aiming: Quat,
        max_slope_angle: Scalar,
        fire_impulse: Scalar,
    ) -> Self {
        self.movement = MovementBundle::new(
            acceleration,
            damping,
            jump_impulse,
            aiming,
            max_slope_angle,
            fire_impulse,
        );
        self
    }
}

fn movement(
  time: Res<Time>,
  mut movement_event_reader: EventReader<PlayerAction>,
  mut controllers: Query<(
      Entity,
      &MovementAcceleration,
      &JumpImpulse,
      &mut AimRotation,
      &mut LinearVelocity,
      Has<Grounded>,
      &mut FireImpulse,
  )>,
) {
  // Precision is adjusted so that the example works with
  // both the `f32` and `f64` features. Otherwise you don't need this.
  let delta_time = time.delta_secs_f64().adjust_precision();
  for event in movement_event_reader.read() {
      match event {
          PlayerAction::Move(e, dir) => {
              if let Ok((_, accel, _, _, mut vel, _, _)) = controllers.get_mut(*e) {
                  vel.x += dir * accel.0 * delta_time;
              }
          }
          PlayerAction::Jump(e) => {
              if let Ok((_, _, jump, _, mut vel, grounded, _)) = controllers.get_mut(*e) {
                  if grounded {
                      vel.y = jump.0;
                  }
              }
          }
          PlayerAction::Aim(e, x, y) => {
              if let Ok((_, _, _, mut aim, _, _, _)) = controllers.get_mut(*e) {
                  let angle = y.atan2(*x) + std::f32::consts::PI / 2.0;
                  aim.0 = Quat::from_rotation_z(angle);
              }
          }
          PlayerAction::Fire(e) => {
              if let Ok((_, _, _, _, _, _, mut fire)) = controllers.get_mut(*e) {
                  fire.0 = 1.0;
              }
          }
      }
  }
}

fn apply_aim_to_gun(
  mut controllers: Query<(Entity, &AimRotation, &mut FireImpulse)>,
  mut guns: Query<(&Parent, &mut Transform), With<Gun>>,
  transforms: Query<&Transform, Without<Gun>>,
  mut commands: Commands,
) {
  for (parent, mut transform) in &mut guns {
      let bullet_transform = if let Ok(parent_transform) = transforms.get(parent.get()) {
          parent_transform.clone()
      } else {
          Transform::default()
      };
      if let Ok((_, aim, mut fire)) = controllers.get_mut(parent.get()) {
          transform.rotation = aim.0;
          if fire.0 > 0.0 {
              let adjusted_aim = aim.0 * Quat::from_rotation_z(-std::f32::consts::FRAC_PI_2); // Rotate by 90 degrees
              let velocity = (adjusted_aim * Vec3::new(500.0, 0.0, 0.0)).truncate();
              println!("Fire impulse: {:?}", fire.0);
              commands.spawn((
                  Projectile {
                      //velocity: aim.0 * Vec2::new(500.0, 0.0), // Set velocity based on the angle
                      //velocity: (aim.0 * Vec3::new(500.0, 0.0, 0.0)).truncate(), // Set velocity based on the angle
                      velocity: velocity,
                      lifetime: 2.0,
                  },
                  Sprite {
                      color: Color::WHITE,
                      custom_size: Some(Vec2::new(10.0, 10.0)),
                      ..default()
                  },
                  Transform {
                      translation: bullet_transform.translation, // Spawn at the gun's position
                      rotation: transform.rotation,
                      ..default()
                  },
                  RigidBody::Dynamic,
                  Collider::circle(5.0),
              ));
          }
          fire.0 = 0.0;
      }
  }
}

// Slows down movement in the X direction.
fn apply_movement_damping(mut query: Query<(&MovementDampingFactor, &mut LinearVelocity)>) {
  for (damping_factor, mut linear_velocity) in &mut query {
      // We could use `LinearDamping`, but we don't want to dampen movement along the Y axis
      linear_velocity.x *= damping_factor.0;
  }
}

fn update_grounded(
  mut commands: Commands,
  mut query: Query<
      (Entity, &ShapeHits, &Rotation, Option<&MaxSlopeAngle>),
      With<CharacterController>,
  >,
) {
  for (entity, hits, rotation, max_slope_angle) in &mut query {
      // The character is grounded if the shape caster has a hit with a normal
      // that isn't too steep.
      let is_grounded = hits.iter().any(|hit| {
          if let Some(angle) = max_slope_angle {
              (rotation * -hit.normal2).angle_to(Vector::Y).abs() <= angle.0
          } else {
              true
          }
      });

      if is_grounded {
          commands.entity(entity).insert(Grounded);
      } else {
          commands.entity(entity).remove::<Grounded>();
      }
  }
}