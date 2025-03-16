use avian2d::{math::*, prelude::*};
use bevy::{ecs::query::Has, prelude::*};
use std::collections::HashMap;

pub struct CharacterControllerPlugin;

impl Plugin for CharacterControllerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerAction>().add_systems(
            Update,
            (
                keyboard_input,
                gamepad_input,
                update_grounded,
                movement,
                apply_movement_damping,
                spawn_character,
                apply_aim_to_gun,
                move_objects,
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

#[derive(Component)]
struct Gun;

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub lifetime: f32, // Time before the projectile is destroyed
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

fn spawn_character(
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

fn gamepad_input(
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

// Sends [`PlayerAction`] events based on keyboard input.
fn keyboard_input(
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
        assignments.players.insert(5, entity);
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

// Responds to [`PlayerAction`] events and moves character controllers accordingly.
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

fn move_objects(
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
