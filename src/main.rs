use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*};

mod terrain;

use terrain::plugin::TerrainPlugin;

#[derive(Component, Default)]
struct CameraAngles {
    pitch: f32,
    yaw: f32,
    roll: f32,
}

fn spawn_debug_cam(mut commands: Commands) {
    let trans = Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        Camera3dBundle {
            projection: PerspectiveProjection {
                fov: 60.0_f32.to_radians(),
                ..default()
            }
            .into(),
            transform: trans,
            ..Default::default()
        },
        CameraAngles { ..default() },
    ));
    commands.spawn(PointLightBundle {
        transform: trans,
        ..Default::default()
    });
}

fn move_camera(
    mut mouse: EventReader<MouseMotion>,
    kb_input: Res<ButtonInput<KeyCode>>,
    mut cam_query: Query<&mut Transform, With<CameraAngles>>,
    mut angles_query: Query<&mut CameraAngles>,
    time: Res<Time>,
) {
    let mut camera = cam_query.single_mut();
    let mut angles = angles_query.single_mut();

    let sensitivity = 4.0;

    for ev in mouse.read() {
        if f32::abs(ev.delta.x) > f32::EPSILON {
            angles.yaw -= ev.delta.x * 0.022 * sensitivity;
        }
        if f32::abs(ev.delta.y) > f32::EPSILON {
            angles.pitch -= ev.delta.y * 0.022 * sensitivity;
            angles.pitch = f32::clamp(angles.pitch, -89.0, 89.0);
        }
    }

    camera.rotation = Quat::IDENTITY;
    camera.rotate_y(angles.yaw.to_radians());
    camera.rotate_local_x(angles.pitch.to_radians());

    let move_speed = 5.0;
    let delta = move_speed * time.delta_seconds();

    let fwd = camera.forward();
    let right = camera.right();
    let up = Vec3::Y;

    if kb_input.pressed(KeyCode::KeyE) {
        camera.translation += fwd * delta;
    }
    if kb_input.pressed(KeyCode::KeyD) {
        camera.translation -= fwd * delta;
    }
    if kb_input.pressed(KeyCode::KeyF) {
        camera.translation += right * delta;
    }
    if kb_input.pressed(KeyCode::KeyS) {
        camera.translation -= right * delta;
    }
    if kb_input.pressed(KeyCode::Space) {
        camera.translation += up * delta;
    }
    if kb_input.pressed(KeyCode::ControlLeft) {
        camera.translation -= up * delta;
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugins(DefaultPlugins)
        .add_plugins(TerrainPlugin { seed: 1337 })
        .add_systems(Startup, spawn_debug_cam)
        .add_systems(Update, move_camera)
        .run();
}
