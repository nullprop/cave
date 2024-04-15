mod player;
mod terrain;

use bevy::{prelude::*, window::CursorGrabMode};
use bevy_rapier3d::{
    plugin::{NoUserData, RapierPhysicsPlugin},
    render::{DebugRenderContext, RapierDebugRenderPlugin},
};
use player::plugin::PlayerPlugin;
use terrain::plugin::TerrainPlugin;

fn debug_input(kb_input: Res<ButtonInput<KeyCode>>, mut debug_render: ResMut<DebugRenderContext>) {
    if kb_input.just_pressed(KeyCode::F1) {
        debug_render.enabled = !debug_render.enabled;
    }
}

fn grab_mouse(
    mut windows: Query<&mut Window>,
    mouse: Res<ButtonInput<MouseButton>>,
    key: Res<ButtonInput<KeyCode>>,
) {
    let mut window = windows.single_mut();

    if mouse.just_pressed(MouseButton::Left) {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Locked;
    }

    if key.just_pressed(KeyCode::Escape) {
        window.cursor.visible = true;
        window.cursor.grab_mode = CursorGrabMode::None;
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 10.0,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Luola".into(),
                resolution: (1920.0, 1080.0).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin {
            enabled: false,
            ..default()
        })
        .add_plugins(TerrainPlugin { seed: 1337 })
        .add_plugins(PlayerPlugin {})
        .add_systems(Update, debug_input)
        .add_systems(Update, grab_mouse)
        .run();
}
