use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    input::mouse::MouseMotion,
    prelude::*,
};
use bevy_rapier3d::{
    control::{CharacterAutostep, CharacterLength, KinematicCharacterControllerOutput},
    dynamics::RigidBody,
    geometry::{Collider, TOIStatus},
    math::Vect,
    prelude::KinematicCharacterController,
};

use crate::terrain::plugin::TerrainCellEvent;

pub struct PlayerPlugin {}

#[derive(Component, Default)]
struct CameraAngles {
    pitch: f32,
}

#[derive(Component, Default)]
pub struct Player {
    noclip: bool,
    noclip_speed: f32,
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
    transform: SpatialBundle,
    controller: KinematicCharacterController,
    rigidbody: RigidBody,
    collider: Collider,
    tag: PlayerTag,
}

#[derive(Bundle, Default)]
struct PlayerCameraBundle {
    camera: Camera3dBundle,
    camera_angles: CameraAngles,
    tag: PlayerTag,
    bloom: BloomSettings,
    fog: FogSettings,
}

#[derive(Bundle, Default)]
struct PlayerLightBundle {
    light: SpotLightBundle,
    tag: PlayerLightTag,
    tag1: PlayerTag,
}

#[derive(Component, Default)]
struct PlayerTag {}

#[derive(Component, Default)]
struct PlayerLightTag {}

impl PlayerPlugin {
    fn spawn_player(mut commands: Commands) {
        let trans = Transform::from_xyz(0.0, 4.0, 5.0);
        commands
            .spawn(PlayerBundle {
                player: Player {
                    noclip_speed: 20.0,
                    max_vel_ground: 5.0,
                    max_vel_air: 8.0,
                    max_fall_vel: 30.0,
                    jump_vel: 4.0,
                    accel: 32.0,
                    air_accel: 8.0,
                    friction: 8.0,
                    camera_height: 1.4,
                    ..default()
                },
                transform: SpatialBundle {
                    transform: trans,
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
                        camera: Camera {
                            hdr: true,
                            ..default()
                        },
                        projection: PerspectiveProjection {
                            fov: 60.0_f32.to_radians(),
                            far: 60.0,
                            ..default()
                        }
                        .into(),
                        tonemapping: Tonemapping::TonyMcMapface,
                        ..default()
                    },
                    bloom: BloomSettings::NATURAL,
                    fog: FogSettings {
                        color: Color::rgba(0.25, 0.25, 0.25, 1.0),
                        falloff: FogFalloff::Linear {
                            start: 50.0,
                            end: 60.0,
                        },
                        ..default()
                    },
                    ..default()
                });
                parent.spawn(PlayerLightBundle {
                    light: SpotLightBundle {
                        spot_light: SpotLight {
                            color: Color::rgb(1.0, 1.0, 1.0),
                            intensity: 1_000_000.0,
                            range: 50.0,
                            radius: 0.0,
                            shadows_enabled: true,
                            inner_angle: 0.0,
                            outer_angle: 40.0_f32.to_radians(),
                            ..default()
                        },
                        ..default()
                    },
                    ..default()
                });
            });
    }

    // todo this should only save the input values.
    // move the translation, etc. to update.
    fn player_input(
        mut mouse: EventReader<MouseMotion>,
        mouse_buttons: Res<ButtonInput<MouseButton>>,
        kb_input: Res<ButtonInput<KeyCode>>,
        mut q_parent: Query<(
            Entity,
            &mut Player,
            &mut KinematicCharacterController,
            &Children,
        )>,
        mut q_cam: Query<(Entity, &mut CameraAngles)>,
        q_light: Query<Entity, With<PlayerLightTag>>,
        mut q_trans: Query<&mut Transform, With<PlayerTag>>,
        time: Res<Time>,
        mut events: EventWriter<TerrainCellEvent>,
    ) {
        let mut mouse_move = Vec2 { x: 0.0, y: 0.0 };
        for ev in mouse.read() {
            mouse_move.x += ev.delta.x;
            mouse_move.y += ev.delta.y;
        }

        let sensitivity = 4.0;

        for (player_id, mut player, mut controller, children) in &mut q_parent {
            if f32::abs(mouse_move.x) > f32::EPSILON {
                player.yaw -= mouse_move.x * 0.022 * sensitivity;
            }
            if kb_input.just_pressed(KeyCode::KeyV) {
                player.noclip = !player.noclip;
            }

            let mut cam_fwd = -Vec3::Z;
            let mut cam_rot = Quat::IDENTITY;

            for child in children.iter() {
                if let Ok(cam) = q_cam.get_mut(*child) {
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
                        cam_rot = cam_trans.rotation;
                        cam_fwd = cam_trans.forward().into();
                    }
                }
            }

            for child in children.iter() {
                if let Ok(light) = q_light.get(*child) {
                    if let Ok(mut light_trans) = q_trans.get_mut(light) {
                        light_trans.translation = Vec3 {
                            x: 0.0,
                            y: player.camera_height,
                            z: 0.0,
                        };
                        light_trans.rotation = cam_rot;
                    }
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
                    let delta = player.noclip_speed * time.delta_seconds();

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

                if mouse_buttons.just_pressed(MouseButton::Left) {
                    events.send(TerrainCellEvent {
                        origin: player_transform.translation,
                        dir: cam_fwd,
                        value: 1.0,
                        radius: 1.5,
                    });
                }
                if mouse_buttons.just_pressed(MouseButton::Right) {
                    events.send(TerrainCellEvent {
                        origin: player_transform.translation,
                        dir: cam_fwd,
                        value: 0.0,
                        radius: 1.5,
                    });
                }
            }
        }
    }

    fn clip_velocity(velocity: Vec3, normal: Vec3, bounce: f32) -> Vec3 {
        let mut backoff = velocity.dot(normal);
        if backoff < bounce {
            backoff *= bounce;
        } else {
            backoff /= bounce;
        }
        return velocity - normal * backoff;
    }

    fn player_update(
        mut players: Query<(&mut Player, &KinematicCharacterControllerOutput)>,
        time: Res<Time>,
    ) {
        for (mut player, output) in players.iter_mut() {
            for col in &output.collisions {
                if col.toi.status == TOIStatus::Converged {
                    if let Some(details) = col.toi.details {
                        player.velocity =
                            Self::clip_velocity(player.velocity, details.normal1, 1.0);
                    }
                }
            }

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
