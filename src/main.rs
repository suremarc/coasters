use std::ops::Mul;

use bevy::{math::vec3, prelude::*, reflect::TypeUuid};
use bevy_flycam::{FlyCam, PlayerPlugin};
use coasters::proc_mesh::Resampler;
use iyes_loopless::prelude::*;
use pythagorean_hodographs::{Curve3, Frame, QuinticPHCurve, Spline};
use serde::Deserialize;

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
        .add_asset::<Coaster>()
        .add_startup_system(setup)
        .add_startup_system(init_coasters)
        .add_system(on_coaster_update)
        .add_system(bind_player_to_coaster)
        .add_system(advance_coaster_joint)
        .add_system(lock_camera_to_coaster_joint)
        .add_system(bevy::window::close_on_esc)
        .run();
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

    // Light
    commands.spawn(PointLightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });

    // Help
    commands.spawn(
        TextBundle::from_section(
            "Press Enter to ride the coaster; press Q to unbind from the track.",
            TextStyle {
                font: default(),
                font_size: 100.0,
                color: Color::WHITE,
            },
        )
        .with_text_alignment(TextAlignment::BOTTOM_RIGHT)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(5.0),
                right: Val::Px(15.0),
                ..default()
            },
            ..default()
        }),
    );
}

#[derive(Deref, DerefMut, Component)]
struct BevySpline(Box<dyn Frame + Send + Sync + 'static>);

#[derive(Debug, Deserialize, TypeUuid)]
#[uuid = "db5ca089-b785-4dfc-a407-f9ff009e8715"]
struct Coaster {
    pts: Vec<Vec3>,
}

#[derive(Component)]
struct CoasterJoint {
    pub coaster: Entity,
    pub state: CoasterJointState,
}

#[derive(Default)]
struct CoasterJointState {
    pub u: f32,
    pub dudt: f32,
}

impl CoasterJointState {
    const MU: f32 = 0.0; // rolling friction
    const G: f32 = 0.075; // gravity

    fn advance(&mut self, curve: &(impl Curve3 + ?Sized), dt: f32) {
        let dpdu = curve.dp(self.u);
        let d2pdu2 = curve.d2p(self.u);

        let d2udt2 = -(Self::G * dpdu.y + Self::MU * Self::G * dpdu.mul(vec3(1., 0., 1.)).length())
            * self.dudt
            + dpdu.dot(d2pdu2) / dpdu.length_squared() * self.dudt.powi(2);

        self.dudt += d2udt2 * dt;
        self.u += self.dudt * dt;
    }
}

fn init_coasters(mut coasters: ResMut<Assets<Coaster>>) {
    coasters.add(Coaster {
        pts: vec![
            vec3(6., 12., 1.),
            vec3(0., 8., 2.),
            vec3(3., 4., 7.),
            vec3(6., 0., 4.),
            vec3(8., 4., 5.),
            vec3(12., 2., 6.),
            vec3(11., 10., 7.),
            vec3(11., 19., 8.),
        ],
    });
}

fn on_coaster_update(
    mut commands: Commands,
    mut ev_asset: EventReader<AssetEvent<Coaster>>,
    assets: Res<Assets<Coaster>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cams: Query<Entity, With<FlyCam>>,
) {
    for ev in ev_asset.iter() {
        if let AssetEvent::Created { handle } = ev {
            let coaster = assets.get(handle).expect("asset should be loaded");
            let entity = spawn_coaster(&mut commands, handle, coaster, &mut meshes, &mut materials);
            // this is broken for now
        }
    }
}

fn bind_player_to_coaster(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    query_coasters: Query<(Entity, &BevySpline), With<Handle<Coaster>>>,
    mut query_player: Query<(Entity, &mut Transform), With<FlyCam>>,
) {
    let (entity, mut transform) = query_player.single_mut();
    if keys.just_pressed(KeyCode::Return) {
        let (coaster_entity, spline) = query_coasters.single();

        commands.get_entity(entity).unwrap().insert(CoasterJoint {
            coaster: coaster_entity,
            state: CoasterJointState { u: 0., dudt: 0.05 },
        });
        transform.rotation = spline.quat(0.);
    } else if keys.just_pressed(KeyCode::Q) {
        commands
            .get_entity(entity)
            .unwrap()
            .remove::<CoasterJoint>();
    }
    // let coaster =
}

fn spawn_coaster(
    commands: &mut Commands,
    handle: &Handle<Coaster>,
    coaster: &Coaster,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) -> Entity {
    let ds = 0.1;

    let start = bevy::utils::Instant::now();
    let spline = Spline::<QuinticPHCurve>::catmull_rom(&coaster.pts);
    let duration = start.elapsed();
    println!("{}", duration.as_micros());

    let m_start = bevy::utils::Instant::now();
    let mesh = coasters::proc_mesh::ribbon(&spline, 0., 1., ds, 2.);
    let m_duration = m_start.elapsed();
    println!("{}", m_duration.as_micros());

    commands
        .spawn((
            handle.clone(),
            BevySpline(Box::new(spline.clone())),
            PbrBundle {
                mesh: meshes.add(mesh),
                material: materials.add(Color::rgb(0.1, 0.4, 0.8).into()),
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            for &pt in coaster.pts.iter() {
                parent.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.3,
                        ..Default::default()
                    })),
                    material: materials.add(Color::RED.into()),
                    transform: Transform::from_translation(pt),
                    ..Default::default()
                });
            }

            for frame in Resampler::new(&spline, 0., 1., ds).map(|u| spline.frame(u)) {
                let position = frame.translation;
                parent.spawn(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Icosphere {
                        radius: 0.1,
                        ..Default::default()
                    })),
                    material: materials.add(Color::GREEN.into()),
                    transform: Transform::from_translation(Vec3::from(position)),
                    ..Default::default()
                });
            }
        })
        .id()
}

fn advance_coaster_joint(
    time: Res<Time>,
    mut query: Query<&mut CoasterJoint>,
    coaster_query: Query<&BevySpline>,
) {
    let dt = time.delta_seconds();
    for mut joint in query.iter_mut() {
        let coaster = coaster_query.get(joint.coaster).unwrap();
        joint.as_mut().state.advance(coaster.0.as_ref(), dt);
    }
}

fn lock_camera_to_coaster_joint(
    mut query: Query<(&mut Transform, &CoasterJoint)>,
    coaster_query: Query<&BevySpline>,
) {
    for (mut transform, joint) in query.iter_mut() {
        let coaster = coaster_query.get(joint.coaster).unwrap();
        let u = joint.state.u;
        transform.as_mut().translation = coaster.p(u) + Vec3::from(coaster.frame(u).y_axis);
        // transform.as_mut().rotation = coaster.quat(u);
    }
}
