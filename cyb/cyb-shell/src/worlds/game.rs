use bevy::prelude::*;
use super::WorldState;

pub struct GameWorldPlugin;

#[derive(Resource, Default)]
struct GameTick(u64);

#[derive(Component)]
struct GameSceneMarker;

impl Plugin for GameWorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameTick>()
            .add_systems(OnEnter(WorldState::Game), show_game_scene)
            .add_systems(OnExit(WorldState::Game), hide_game_scene)
            .add_systems(
                Update,
                rotate_cube.run_if(in_state(WorldState::Game)),
            );
    }
}

fn show_game_scene(world: &mut World) {
    // Check if scene already exists
    let existing: Vec<Entity> = world
        .query_filtered::<Entity, With<GameSceneMarker>>()
        .iter(world)
        .collect();

    if !existing.is_empty() {
        // Already created — show everything
        for entity in &existing {
            if let Some(mut vis) = world.get_mut::<Visibility>(*entity) {
                *vis = Visibility::Visible;
            }
            if let Some(mut cam) = world.get_mut::<Camera>(*entity) {
                cam.is_active = true;
            }
        }
        info!("Game scene shown (persisted)");
        return;
    }

    // First time — spawn with world.spawn() for immediate materialization
    let mut meshes = world.resource_mut::<Assets<Mesh>>();
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    let mut materials = world.resource_mut::<Assets<StandardMaterial>>();
    let cube_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.8, 0.3),
        ..default()
    });

    world.spawn((
        Camera3d::default(),
        Camera {
            clear_color: ClearColorConfig::Custom(bevy::color::Color::BLACK),
            is_active: true,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        GameSceneMarker,
    ));

    world.spawn((
        Mesh3d(cube_mesh),
        MeshMaterial3d(cube_material),
        Transform::default(),
        GameSceneMarker,
    ));

    world.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.5, 0.5, 0.0)),
        GameSceneMarker,
    ));
}

fn hide_game_scene(world: &mut World) {
    let entities: Vec<Entity> = world
        .query_filtered::<Entity, With<GameSceneMarker>>()
        .iter(world)
        .collect();
    for entity in &entities {
        if let Some(mut vis) = world.get_mut::<Visibility>(*entity) {
            *vis = Visibility::Hidden;
        }
        if let Some(mut cam) = world.get_mut::<Camera>(*entity) {
            cam.is_active = false;
        }
    }
    info!("Game scene hidden (state persisted)");
}

fn rotate_cube(
    time: Res<Time>,
    mut query: Query<&mut Transform, (With<GameSceneMarker>, With<Mesh3d>)>,
    mut tick: ResMut<GameTick>,
) {
    tick.0 += 1;
    for mut transform in &mut query {
        transform.rotate_y(time.delta_secs());
    }
}
