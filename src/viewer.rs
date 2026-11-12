//! The 3D viewer: opens a window and shows the terrain with Bevy.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};

use crate::terrain::Terrain;

/// The settings the viewer needs. Stored as a Bevy "Resource" (global data).
#[derive(Resource)]
struct ViewerSettings {
    asc_path: String,
    step: usize,
    vertical_scale: f32,
}

/// Opens the 3D window. This function only returns when the window is closed.
pub fn run(asc_path: &str, step: usize, vertical_scale: f32) {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OptiFlux".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(PanOrbitCameraPlugin)
        .insert_resource(ClearColor(Color::srgb(0.53, 0.75, 0.92))) // sky blue
        .insert_resource(ViewerSettings {
            asc_path: asc_path.to_string(),
            step,
            vertical_scale,
        })
        .add_systems(Startup, setup)
        .run();
}

/// Runs once at startup: loads the terrain, creates the mesh, light and camera.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    settings: Res<ViewerSettings>,
) {
    // 1. Load the terrain (our own code!)
    let terrain = Terrain::from_asc(&settings.asc_path);
    let (positions, indices) = terrain.mesh_data(settings.step, settings.vertical_scale);
    println!(
        "Terrain mesh: {} points, {} triangles",
        positions.len(),
        indices.len() / 3
    );

    // 2. Turn our points and triangles into a Bevy mesh
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_indices(Indices::U32(indices));
    mesh.compute_smooth_normals(); // normals = which way each point faces, needed for lighting

    commands.spawn((
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.55, 0.30), // grass green
            perceptual_roughness: 0.9,                 // not shiny
            ..default()
        })),
    ));

    // 3. The sun: a light coming from far away, at an angle so the relief casts shadows
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(1.0, 2.0, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 4. The camera, looking at the center of the terrain
    let (size_x, size_z) = terrain.size_in_metres();
    let center = Vec3::new(size_x / 2.0, 0.0, size_z / 2.0);
    let distance = size_x.max(size_z);

    commands.spawn((
        Transform::from_translation(center + Vec3::new(0.0, distance * 0.6, distance * 0.8)),
        PanOrbitCamera {
            focus: center,
            ..default()
        },
    ));
}
