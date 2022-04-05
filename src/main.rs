use bevy::{math::const_vec3, prelude::*};
use bevy_flycam::PlayerPlugin;
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
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup)
        .add_startup_system(draw_spline)
        .run();
}

fn draw_spline(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let spline = catmull_rom::CatmullRom3::new(vec![
        const_vec3!([6., 12., 0.]),
        const_vec3!([0., 8., 0.]),
        const_vec3!([3., 4., 0.]),
        const_vec3!([6., 0., 0.]),
        const_vec3!([8., 4., 0.]),
        const_vec3!([12., 2., 0.]),
        const_vec3!([11., 10., 0.]),
        const_vec3!([11., 19., 0.]),
    ]);

    // let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);

    // mesh.set_attribute(
    //     Mesh::ATTRIBUTE_POSITION,
    //     vec![
    //         [0.0, 8.0, -0.1],
    //         [0.0, 8.0, 0.1],
    //         [-0.030238556, 7.9063673, -0.1],
    //         [-0.030238556, 7.9063673, 0.1],
    //     ],
    // );

    // mesh.set_indices(Some(bevy::render::mesh::Indices::U32(vec![
    //     0, 2, 1, 0, 3, 2,
    // ])));

    // mesh.set_attribute(Mesh::ATTRIBUTE_COLOR, vec![[0.0, 0.0, 0.0, 1.0]; 4]);

    let mesh = spline.ribbon_mesh(0., 1., 1., 2.);

    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(mesh),
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
