use bevy::{input::mouse::MouseMotion, prelude::*};
use bevy_rapier3d::{
    control::{CharacterAutostep, CharacterLength, KinematicCharacterControllerOutput},
    dynamics::RigidBody,
    geometry::Collider,
    math::Vect,
    prelude::KinematicCharacterController,
};

pub struct PlayerPlugin {}

#[derive(Component, Default)]
struct CameraAngles {
    pitch: f32,
}

#[derive(Component, Default)]
struct Player {
    noclip: bool,
    yaw: f32,
    velocity: Vec3,
    accel: f32,
    air_accel: f32,
    max_vel_ground: f32,
    max_vel_air: f32,
    max_fall_vel: f32,
    jump_vel: f32,
    friction: f32,
    grounded: bool,
    wish_dir: Vec3,
    wish_jump: bool,
    wish_duck: bool,
    camera_height: f32,
}

#[derive(Bundle, Default)]
struct PlayerBundle {
    player: Player,
    tag: PlayerTag,
    transform: TransformBundle,
    controller: KinematicCharacterController,
    rigidbody: RigidBody,
    collider: Collider,
}

#[derive(Bundle, Default)]
struct PlayerCameraBundle {
    camera: Camera3dBundle,
    camera_angles: CameraAngles,
    tag: PlayerTag,
}

#[derive(Component, Default)]
struct PlayerTag {}

impl PlayerPlugin {
    fn spawn_player(mut commands: Commands) {
        let trans = Transform::from_xyz(0.0, 2.0, 5.0);
        commands
            .spawn(PlayerBundle {
                player: Player {
                    max_vel_ground: 5.0,
                    max_vel_air: 8.0,
                    max_fall_vel: 30.0,
                    jump_vel: 4.0,
                    accel: 32.0,
                    air_accel: 8.0,
                    friction: 16.0,
                    camera_height: 1.4,
                    ..default()
                },
                transform: TransformBundle {
                    local: trans,
                    ..default()
                },
                controller: KinematicCharacterController {
                    offset: CharacterLength::Absolute(0.01),
                    max_slope_climb_angle: 60.0_f32.to_radians(),
                    min_slope_slide_angle: 30.0_f32.to_radians(),
                    autostep: Some(CharacterAutostep {
                        max_height: CharacterLength::Absolute(0.5),
                        min_width: CharacterLength::Absolute(0.05),
                        include_dynamic_bodies: true,
                    }),
                    ..default()
                },
                rigidbody: RigidBody::KinematicPositionBased,
                collider: Collider::capsule(
                    Vect::ZERO,
                    Vect {
                        x: 0.0,
                        y: 1.6,
                        z: 0.0,
                    },
                    0.3,
                ),
                ..default()
            })
            .with_children(|parent| {
                parent.spawn(PlayerCameraBundle {
                    camera: Camera3dBundle {
                        projection: PerspectiveProjection {
                            fov: 60.0_f32.to_radians(),
                            ..default()
                        }
                        .into(),
                        ..default()
                    },
                    ..default()
                });
            });
        commands.spawn(PointLightBundle {
            transform: trans,
            ..Default::default()
        });
    }

    // todo this should only save the input values.
    // move the translation, etc. to update.
    fn player_input(
        mut mouse: EventReader<MouseMotion>,
        kb_input: Res<ButtonInput<KeyCode>>,
        mut q_parent: Query<(
            Entity,
            &mut Player,
            &mut KinematicCharacterController,
            &Children,
        )>,
        mut q_child: Query<(Entity, &mut CameraAngles)>,
        mut q_trans: Query<&mut Transform, With<PlayerTag>>,
        time: Res<Time>,
    ) {
        let mut mouse_move = Vec2 { x: 0.0, y: 0.0 };
        for ev in mouse.read() {
            mouse_move.x += ev.delta.x;
            mouse_move.y += ev.delta.y;
        }

        let sensitivity = 4.0;

        for (player_id, mut player, mut controller, camera_ids) in &mut q_parent {
            if f32::abs(mouse_move.x) > f32::EPSILON {
                player.yaw -= mouse_move.x * 0.022 * sensitivity;
            }
            if kb_input.just_pressed(KeyCode::KeyV) {
                player.noclip = !player.noclip;
            }

            let mut cam_fwd = -Vec3::Z;

            if let Ok(cam) = q_child.get_mut(camera_ids[0]) {
                let cam_id = cam.0;
                let mut cam_angles = cam.1;

                if f32::abs(mouse_move.y) > f32::EPSILON {
                    cam_angles.pitch -= mouse_move.y * 0.022 * sensitivity;
                    cam_angles.pitch = f32::clamp(cam_angles.pitch, -89.0, 89.0);
                }

                if let Ok(mut cam_trans) = q_trans.get_mut(cam_id) {
                    cam_trans.translation = Vec3 {
                        x: 0.0,
                        y: player.camera_height,
                        z: 0.0,
                    };
                    cam_trans.rotation = Quat::IDENTITY;
                    cam_trans.rotate_local_x(cam_angles.pitch.to_radians());
                    cam_fwd = cam_trans.forward().into();
                }
            }

            if let Ok(mut player_transform) = q_trans.get_mut(player_id) {
                player_transform.rotation = Quat::IDENTITY;
                player_transform.rotate_y(player.yaw.to_radians());

                let fwd: Vec3 = player_transform.forward().into();
                let right: Vec3 = player_transform.right().into();
                let up = Vec3::Y;
                cam_fwd = player_transform.rotation.mul_vec3(cam_fwd);

                if player.noclip {
                    let move_speed = 5.0;
                    let delta = move_speed * time.delta_seconds();

                    if kb_input.pressed(KeyCode::KeyE) {
                        player_transform.translation += cam_fwd * delta;
                    }
                    if kb_input.pressed(KeyCode::KeyD) {
                        player_transform.translation -= cam_fwd * delta;
                    }
                    if kb_input.pressed(KeyCode::KeyF) {
                        player_transform.translation += right * delta;
                    }
                    if kb_input.pressed(KeyCode::KeyS) {
                        player_transform.translation -= right * delta;
                    }
                    if kb_input.pressed(KeyCode::Space) {
                        player_transform.translation += up * delta;
                    }
                    if kb_input.pressed(KeyCode::ControlLeft) {
                        player_transform.translation -= up * delta;
                    }

                    player.velocity = Vect::ZERO;
                } else {
                    player.velocity.y -= 9.81 * time.delta_seconds();

                    let mut wish_dir = Vec3::ZERO;

                    if kb_input.pressed(KeyCode::KeyE) {
                        wish_dir += fwd;
                    }
                    if kb_input.pressed(KeyCode::KeyD) {
                        wish_dir -= fwd;
                    }
                    if kb_input.pressed(KeyCode::KeyF) {
                        wish_dir += right;
                    }
                    if kb_input.pressed(KeyCode::KeyS) {
                        wish_dir -= right;
                    }

                    if wish_dir.length() > f32::EPSILON {
                        wish_dir = wish_dir.normalize();
                    }
                    player.wish_dir = wish_dir;
                    player.wish_jump = kb_input.pressed(KeyCode::Space);
                    player.wish_duck = kb_input.pressed(KeyCode::ControlLeft);

                    let accel = if player.grounded {
                        player.accel
                    } else {
                        player.air_accel
                    };
                    let delta_v = player.wish_dir * accel * time.delta_seconds();
                    player.velocity += delta_v;

                    if player.grounded && player.wish_jump {
                        player.velocity.y = player.jump_vel;
                    }

                    let movement = player.velocity * time.delta_seconds();
                    controller.translation = Some(movement);
                }
            }
        }
    }

    fn player_update(
        mut players: Query<(&mut Player, &KinematicCharacterControllerOutput)>,
        time: Res<Time>,
    ) {
        for (mut player, output) in players.iter_mut() {
            player.grounded = output.grounded;
            if player.grounded {
                player.velocity.y = 0.0;

                let friction = player.friction * time.delta_seconds();

                if f32::abs(player.velocity.x) < friction {
                    player.velocity.x = 0.0;
                } else if player.velocity.x > friction {
                    player.velocity.x -= friction
                } else {
                    player.velocity.x += friction;
                }

                if f32::abs(player.velocity.z) < friction {
                    player.velocity.z = 0.0;
                } else if player.velocity.z > friction {
                    player.velocity.z -= friction
                } else {
                    player.velocity.z += friction;
                }

                player.velocity = player.velocity.clamp_length_max(player.max_vel_ground);
            } else {
                let vel_y = player
                    .velocity
                    .y
                    .clamp(-player.max_fall_vel, player.max_fall_vel);
                player.velocity.y = 0.0;
                player.velocity = player.velocity.clamp_length_max(player.max_vel_air);
                player.velocity.y = vel_y;
            }
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::spawn_player)
            .add_systems(Update, Self::player_update)
            .add_systems(Update, Self::player_input);
    }
}
