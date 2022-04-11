use bevy::{math::const_vec3, prelude::*};
use bevy_flycam::PlayerPlugin;
use coasters::curve::{Frame, Resample};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Coasters!".to_string(),
            width: 1280.,
            height: 720.,
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
    let pts = vec![
        const_vec3!([6., 12., 1.]),
        const_vec3!([0., 8., 2.]),
        const_vec3!([3., 4., 7.]),
        const_vec3!([6., 0., 4.]),
        const_vec3!([8., 4., 5.]),
        const_vec3!([12., 2., 6.]),
        const_vec3!([11., 10., 7.]),
        const_vec3!([11., 19., 8.]),
    ];

    let ds = 0.1;

    let start = bevy::utils::Instant::now();
    let spline =
        coasters::curve::Spline::<coasters::curve::EulerRodriguesFrame>::catmull_rom(pts.clone());
    let duration = start.elapsed();
    println!("{}", duration.as_micros());

    let m_start = bevy::utils::Instant::now();
    let mesh = coasters::proc_mesh::ribbon(&spline, 0., 1., ds, 2.);
    let m_duration = m_start.elapsed();
    println!("{}", m_duration.as_micros());

    for &pt in pts.iter() {
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.3,
                ..Default::default()
            })),
            material: materials.add(Color::RED.into()),
            transform: Transform::from_translation(pt),
            ..Default::default()
        });
    }

    for frame in spline
        .resample(0., 1., ds)
        .into_iter()
        .map(|u| spline.frame(u))
    {
        let position = frame.translation;
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                ..Default::default()
            })),
            material: materials.add(Color::GREEN.into()),
            transform: Transform::from_translation(Vec3::from(position)),
            ..Default::default()
        });
    }

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
        transform: Transform::from_translation(Vec3::new(4., -1., 4.)),
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
