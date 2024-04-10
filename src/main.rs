use std::f32::consts::PI;

use bevy::{input::mouse::MouseMotion, prelude::*};

mod terrain;

use terrain::plugin::TerrainPlugin;

fn spawn_debug_cam(mut commands: Commands) {
    let trans = Transform::from_xyz(0.0, 2.0, -2.0).looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn(Camera3dBundle {
        transform: trans,
        ..Default::default()
    });
    commands.spawn(PointLightBundle {
        transform: trans,
        ..Default::default()
    });
}

fn move_camera(
    mut mouse: EventReader<MouseMotion>,
    kb_input: Res<ButtonInput<KeyCode>>,
    mut cam_query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    let mut camera = cam_query.single_mut();

    for ev in mouse.read() {
        if f32::abs(ev.delta.x) > f32::EPSILON {
            camera.rotate_local_z(ev.delta.x * 0.022);
        }
        if f32::abs(ev.delta.y) > f32::EPSILON {
            camera.rotate_local_x(ev.delta.y * 0.022);
            let angle = camera.rotation.to_euler(EulerRot::XYZ).0;
            let max = 0.5 * PI;
            if angle < -max {
                camera.rotate_local_x(-max - angle);
            }
            if angle > max {
                camera.rotate_local_x(max - angle);
            }
        }
    }

    let move_speed = 5.0;
    let delta = move_speed * time.delta_seconds();

    let fwd = camera.forward();
    let right = camera.forward();
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

    info!(
        "{:?} {:?} {:?}",
        camera.translation.x, camera.translation.y, camera.translation.z,
    );
    info!("{:?}", camera.rotation.to_euler(EulerRot::XYZ),);
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
