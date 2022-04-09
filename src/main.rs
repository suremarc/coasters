use bevy::{math::const_vec3, prelude::*};
use bevy_flycam::PlayerPlugin;
use coasters::curve::{Frame, GenericCurve, Resample};

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

fn as_float3(vals: &bevy::render::mesh::VertexAttributeValues) -> Option<&[[f32; 3]]> {
    match vals {
        bevy::render::mesh::VertexAttributeValues::Float32x3(values) => Some(values),
        _ => None,
    }
}

fn draw_spline(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // let spline = coasters::curve::CatmullRom3::new(vec![
    //     const_vec3!([6., 12., 0.]),
    //     const_vec3!([0., 8., 0.]),
    //     const_vec3!([3., 4., 0.]),
    //     const_vec3!([6., 0., 0.]),
    //     const_vec3!([8., 4., 0.]),
    //     const_vec3!([12., 2., 0.]),
    //     const_vec3!([11., 10., 0.]),
    //     const_vec3!([11., 19., 0.]),
    // ]);

    // const P0: Vec3 = const_vec3!([6., 12., 0.]);
    // const P1: Vec3 = const_vec3!([0., 8., 1.]);
    // const P2: Vec3 = const_vec3!([3., 4., 5.]);
    // const P3: Vec3 = const_vec3!([6., 0., 10.]);

    // let spline = coasters::curve::HermiteQuintic::new(P1, P2, 0.25 * (P2 - P0), 0.25 * (P3 - P1));

    const P0: Vec3 = const_vec3!([0., 0., 0.]);
    const P1: Vec3 = const_vec3!([8., 8., 8.]);
    const D0: Vec3 = const_vec3!([8., 0., 8.]);
    const D1: Vec3 = const_vec3!([0., 8., 8.]);

    let start = bevy::utils::Instant::now();
    let spline = coasters::curve::HelicalPHQuinticSplineSegment::new(P0, P1, D0, D1);
    // let curve = spline.curve();
    let duration = start.elapsed();
    println!("{}", duration.as_millis());

    let frame = spline.euler_rodrigues_frame();

    let m_start = bevy::utils::Instant::now();
    let mesh = coasters::curve::ribbon_mesh(&frame, 0., 1., 0.1, 1.);
    let m_duration = m_start.elapsed();
    println!("{}", m_duration.as_millis());

    let positions = mesh
        .attribute(Mesh::ATTRIBUTE_POSITION)
        .map(as_float3)
        .unwrap()
        .expect("`Mesh::ATTRIBUTE_POSITION` vertex attributes should be of type `float3`");
    // println!("{:#?}", positions);

    let normals = mesh
        .attribute(Mesh::ATTRIBUTE_NORMAL)
        .map(as_float3)
        .unwrap()
        .expect("`Mesh::ATTRIBUTE_NORMAL` vertex attributes should be of type `float3`");

    for vert in positions {
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                ..Default::default()
            })),
            material: materials.add(Color::GOLD.into()),
            transform: Transform::from_translation(Vec3::from(*vert)),
            ..Default::default()
        });
    }

    for (vert, normal) in positions.iter().zip(normals) {
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.1,
                ..Default::default()
            })),
            material: materials.add(Color::AZURE.into()),
            transform: Transform::from_translation(Vec3::from(*vert) + Vec3::from(*normal)),
            ..Default::default()
        });
    }

    // for p in [P0 - D0, P0, P1, P1 + D1] {
    //     commands.spawn_bundle(PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Icosphere {
    //             radius: 0.3,
    //             ..Default::default()
    //         })),
    //         material: materials.add(Color::SILVER.into()),
    //         transform: Transform::from_translation(p),
    //         ..Default::default()
    //     });
    // }

    // commands.spawn_bundle(PbrBundle {
    //     mesh: meshes.add(mesh),
    //     material: materials.add(Color::rgb(0.1, 0.4, 0.8).into()),
    //     ..Default::default()
    // });

    // for position in curve
    //     .resample(0., 1., 0.1)
    //     .into_iter()
    //     .map(|u| frame.frame(u).translation)
    // {
    //     commands.spawn_bundle(PbrBundle {
    //         mesh: meshes.add(Mesh::from(shape::Icosphere {
    //             radius: 0.05,
    //             ..Default::default()
    //         })),
    //         material: materials.add(Color::GOLD.into()),
    //         transform: Transform::from_translation(Vec3::from(position)),
    //         ..Default::default()
    //     });
    // }
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
