use bevy::{input::mouse::MouseMotion, prelude::*};

pub struct PlayerPlugin {}

#[derive(Component, Default)]
struct CameraAngles {
    pitch: f32,
}

#[derive(Component, Default)]
struct Player {
    noclip: bool,
    yaw: f32,
}

#[derive(Bundle, Default)]
struct PlayerBundle {
    player: Player,
    tag: PlayerTag,
    transform: TransformBundle,
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
                transform: TransformBundle {
                    local: trans,
                    ..default()
                },
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

    fn update_player(
        mut mouse: EventReader<MouseMotion>,
        kb_input: Res<ButtonInput<KeyCode>>,
        mut q_parent: Query<(Entity, &mut Player, &Children)>,
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

        for (player_id, mut player, camera_ids) in &mut q_parent {
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
                    cam_trans.rotation = Quat::IDENTITY;
                    cam_trans.rotate_local_x(cam_angles.pitch.to_radians());
                    cam_fwd = cam_trans.forward().into();
                }
            }

            if let Ok(mut player_transform) = q_trans.get_mut(player_id) {
                player_transform.rotation = Quat::IDENTITY;
                player_transform.rotate_y(player.yaw.to_radians());

                let fwd = player_transform.forward();
                let right = player_transform.right();
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
                } else {
                }
            }
        }
    }
}

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, Self::spawn_player)
            .add_systems(Update, Self::update_player);
    }
}
