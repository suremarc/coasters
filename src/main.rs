use bevy::{math::vec3, prelude::*};
use bevy_flycam::PlayerPlugin;
use coasters::proc_mesh::resample;
use pythagorean_hodographs::{Frame, QuinticPHCurve, Spline};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                title: "Coasters!".to_string(),
                width: 1280.,
                height: 720.,
                ..Default::default()
            },
            ..Default::default()
        }))
        .add_plugin(PlayerPlugin)
        .insert_resource(Msaa { samples: 4 })
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
        vec3(6., 12., 1.),
        vec3(0., 8., 2.),
        vec3(3., 4., 7.),
        vec3(6., 0., 4.),
        vec3(8., 4., 5.),
        vec3(12., 2., 6.),
        vec3(11., 10., 7.),
        vec3(11., 19., 8.),
    ];

    let ds = 0.1;

    let start = bevy::utils::Instant::now();
    let spline = Spline::<QuinticPHCurve>::catmull_rom(pts.clone());
    let duration = start.elapsed();
    println!("{}", duration.as_micros());

    let m_start = bevy::utils::Instant::now();
    let mesh = coasters::proc_mesh::ribbon(&spline, 0., 1., ds, 2.);
    let m_duration = m_start.elapsed();
    println!("{}", m_duration.as_micros());

    for &pt in pts.iter() {
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.3,
                ..Default::default()
            })),
            material: materials.add(Color::RED.into()),
            transform: Transform::from_translation(pt),
            ..Default::default()
        });
    }

    for frame in resample(&spline, 0., 1., ds)
        .into_iter()
        .map(|u| spline.frame(u))
    {
        let position = frame.translation;
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                ..Default::default()
            })),
            material: materials.add(Color::GREEN.into()),
            transform: Transform::from_translation(Vec3::from(position)),
            ..Default::default()
        });
    }

    commands.spawn(PbrBundle {
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
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 8.0 })),
        material: materials.add(Color::rgb(1., 0.9, 0.9).into()),
        transform: Transform::from_translation(Vec3::new(4., -1., 4.)),
        ..Default::default()
    });
    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_matrix(Mat4::from_rotation_translation(
            Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
            Vec3::new(-7.0, 20.0, 4.0),
        )),
        ..Default::default()
    });
    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}
