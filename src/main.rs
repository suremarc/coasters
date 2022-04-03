use bevy::{math::const_vec3, prelude::*};
use coasters::{catmull_rom, curve::Curve};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Coasters!".to_string(),
            width: 1200.,
            height: 1200.,
            ..Default::default()
        })
        // .insert_resource(WgpuOptions {
        //     features: WgpuFeatures::POLYGON_MODE_LINE,
        //     ..Default::default()
        // })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_startup_system(draw_spline)
        .run();
}

fn piecewise_line_mesh(pts: Vec<Vec3>) -> Mesh {
    let vertices: Vec<[f32; 3]> = pts.iter().map(Vec3::to_array).collect();
    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::LineStrip);
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    mesh
}

fn draw_spline(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let spline = catmull_rom::CatmullRom3::new(vec![
        const_vec3!([12., 0., 6.]),
        const_vec3!([8., 0., 0.]),
        const_vec3!([4., 0., 3.]),
        const_vec3!([0., 0., 6.]),
        const_vec3!([4., 0., 8.]),
        const_vec3!([2., 0., 12.]),
        const_vec3!([10., 0., 11.]),
        const_vec3!([19., 0., 11.]),
    ]);

    let us = spline.equidistant_resampling(0., 1., 0.1);
    let pts: Vec<Vec3> = us.into_iter().map(|u| spline.p(u)).collect();

    let mesh = meshes.add(piecewise_line_mesh(pts));

    commands.spawn_bundle(PbrBundle {
        mesh,
        material: materials.add(Color::rgb(0.1, 0.4, 0.8).into()),
        ..Default::default()
    });
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    info!("Press 'm' to toggle MSAA");
    info!("Using 4x MSAA");

    // Plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 8.0 })),
        material: materials.add(Color::rgb(1., 0.9, 0.9).into()),
        transform: Transform::from_translation(Vec3::new(4., 0., 4.)),
        ..Default::default()
    });
    // Camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_matrix(Mat4::from_rotation_translation(
            Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
            Vec3::new(-7.0, 20.0, 4.0),
        )),
        ..Default::default()
    });
    // Light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}
