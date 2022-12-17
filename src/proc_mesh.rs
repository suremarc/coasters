use bevy::prelude::Mesh;

use pythagorean_hodographs::{Curve, Frame};

pub fn resample(curve: &impl Curve, u_start: f32, u_stop: f32, ds: f32) -> Vec<f32> {
    let mut u = u_start;
    let mut us = Vec::<f32>::new();
    while u < u_stop {
        us.push(u);
        u += ds / curve.speed(u);
    }

    us
}

pub fn ribbon(curve: &(impl Curve + Frame), u_start: f32, u_end: f32, ds: f32, width: f32) -> Mesh {
    let us = resample(curve, u_start, u_end, ds);

    let mut verts: Vec<[f32; 3]> = Vec::with_capacity(us.len() * 2);
    let mut normals = verts.clone();
    let uvs: Vec<[f32; 2]> = (0..verts.capacity())
        .map(|i| [(i % 2) as f32, ((i / 2) % 2) as f32])
        .collect();

    let w2 = width / 2.;

    for frame in us.into_iter().map(|u| curve.frame(u)) {
        let p = frame.translation;
        let x = frame.z_axis;
        let n = frame.y_axis;

        verts.push((p - w2 * x).to_array());
        verts.push((p + w2 * x).to_array());

        normals.push(n.to_array());
        normals.push(n.to_array());
    }

    let mut indices = (0..verts.len() as u32).collect::<Vec<_>>();
    let mut reversed: Vec<_> = indices.clone().into_iter().rev().skip(1).collect();
    indices.append(&mut reversed);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleStrip);

    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh
}
