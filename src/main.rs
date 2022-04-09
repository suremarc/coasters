use bevy::{math::const_vec3, prelude::*};
use bevy_flycam::PlayerPlugin;
use coasters::curve::Curve;

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
    const P1: Vec3 = const_vec3!([1., 1., 1.]);
    const D0: Vec3 = const_vec3!([1., 0., 1.]);
    const D1: Vec3 = const_vec3!([0., 1., 1.]);

    let start = bevy::utils::Instant::now();
    let spline = coasters::curve::HelicalPHQuinticSpline::new(P0, P1, D0, D1);
    let curve = spline.curve();
    let duration = start.elapsed();
    println!("{}", duration.as_millis());

    // let mesh = spline.ribbon_mesh(0., 1., 0.5, 1.);

    // let positions = mesh
    //     .attribute(Mesh::ATTRIBUTE_POSITION)
    //     .map(as_float3)
    //     .unwrap()
    //     .expect("`Mesh::ATTRIBUTE_POSITION` vertex attributes should be of type `float3`");
    // println!("{:#?}", positions);

    // let normals = mesh
    //     .attribute(Mesh::ATTRIBUTE_NORMAL)
    //     .map(as_float3)
    //     .unwrap()
    //     .expect("`Mesh::ATTRIBUTE_NORMAL` vertex attributes should be of type `float3`");

    dbg!(P0, curve.p(0.));
    dbg!(P1, curve.p(1.));

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

    for position in curve
        .equidistant_resampling_ph(0., 1., 0.1)
        .into_iter()
        .map(|u| curve.p(u))
    {
        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Icosphere {
                radius: 0.05,
                ..Default::default()
            })),
            material: materials.add(Color::GOLD.into()),
            transform: Transform::from_translation(position),
            ..Default::default()
        });
    }
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
