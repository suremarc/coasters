use bevy::prelude::Mesh;

use crate::curve::{Frame, Resample};

pub fn ribbon<T: Frame + Resample>(
    curve: &T,
    u_start: f32,
    u_end: f32,
    ds: f32,
    width: f32,
) -> Mesh {
    let us = curve.resample(u_start, u_end, ds);

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

    // println!("{:#?}", verts);

    let mut indices = (0..verts.len() as u32).collect::<Vec<_>>();
    let mut reversed: Vec<_> = indices.clone().into_iter().rev().skip(1).collect();
    indices.append(&mut reversed);

    // println!("{:#?}", indices);

    let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleStrip);
    mesh.set_attribute(
        Mesh::ATTRIBUTE_COLOR,
        verts
            .iter()
            .map(|_| [0.0, 0.0, 0.0, 1.0])
            .collect::<Vec<[f32; 4]>>(),
    );

    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, verts);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
    mesh
}
