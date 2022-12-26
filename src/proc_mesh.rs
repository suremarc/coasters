use bevy::prelude::Mesh;

use pythagorean_hodographs::{Curve3, Frame};

pub struct Resampler<'a, T: Curve3> {
    curve: &'a T,
    u: f32,
    ds: f32,
    limit: f32,
}

impl<'a, T: Curve3> Iterator for Resampler<'a, T> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.u >= self.limit {
            None
        } else {
            let old_u = self.u;
            self.u += self.ds / self.curve.speed(self.u);
            Some(old_u)
        }
    }
}

impl<'a, T: Curve3> Resampler<'a, T> {
    pub fn new(curve: &'a T, u_start: f32, u_stop: f32, ds: f32) -> Self {
        Resampler {
            curve,
            u: u_start,
            ds,
            limit: u_stop,
        }
    }
}

pub fn ribbon(
    curve: &(impl Curve3 + Frame),
    u_start: f32,
    u_end: f32,
    ds: f32,
    width: f32,
) -> Mesh {
    let estimated_capacity = curve.arc_length(1.0).ceil() as usize;

    let mut verts: Vec<[f32; 3]> = Vec::with_capacity(estimated_capacity * 2);
    let mut normals = verts.clone();

    let w2 = width / 2.;

    for frame in Resampler::new(curve, u_start, u_end, ds).map(|u| curve.frame(u)) {
        let p = frame.translation;
        let x = frame.z_axis;
        let n = frame.y_axis;

        verts.push((p - w2 * x).to_array());
        verts.push((p + w2 * x).to_array());

        normals.push(n.to_array());
        normals.push(n.to_array());
    }

    let uvs: Vec<[f32; 2]> = (0..verts.len())
        .map(|i| [(i % 2) as f32, ((i / 2) % 2) as f32])
        .collect();

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
