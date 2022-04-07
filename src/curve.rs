use bevy::math::*;
use bevy::prelude::{Mesh, Vec3};

use itertools::Itertools;

pub trait Curve {
    fn p(&self, u: f32) -> Vec3;
    fn dp(&self, u: f32) -> Vec3;
    fn d2p(&self, u: f32) -> Vec3;

    fn tangent(&self, u: f32) -> Vec3 {
        self.dp(u).normalize()
    }

    fn normal(&self, u: f32) -> Vec3 {
        let dpdu = self.dp(u);
        let d2pdu2 = self.d2p(u);

        dpdu.cross(d2pdu2.cross(dpdu)).normalize()
    }

    fn binormal(&self, u: f32) -> Vec3 {
        self.dp(u).cross(self.d2p(u)).normalize()
    }

    fn frame(&self, u: f32) -> Affine3A {
        Affine3A::from_mat3_translation(
            Mat3::from_cols(self.binormal(u), self.normal(u), self.tangent(u)),
            self.p(u),
        )
    }

    fn equidistant_resampling(&self, u_start: f32, u_stop: f32, ds: f32) -> Vec<f32> {
        let mut u = u_start;
        let mut us = Vec::<f32>::new();
        while u < u_stop {
            us.push(u);
            let dpdu = self.dp(u);
            // let d2pdu2 = self.d2p(u);
            // u += ds * dpdu.length_recip() - ds * ds / 4. * dpdu.dot(d2pdu2) / dpdu.length_squared();
            u += ds * dpdu.length_recip();
        }

        us
    }

    fn ribbon_mesh(&self, u_start: f32, u_end: f32, ds: f32, width: f32) -> Mesh {
        let us = self.equidistant_resampling(u_start, u_end, ds);
        // us.resize(2, 0.);
        let ps = us.iter().map(|&u| self.p(u));
        let xs = us.iter().map(|&u| self.binormal(u));

        let mut verts: Vec<[f32; 3]> = Vec::with_capacity(us.len() * 2);
        let mut normals = verts.clone();
        let uvs: Vec<[f32; 2]> = (0..verts.capacity())
            .map(|i| [(i % 2) as f32, ((i / 2) % 2) as f32])
            .collect();

        let w2 = width / 2.;
        for (p, x) in ps.zip(xs) {
            verts.push((p - w2 * x).to_array());
            verts.push((p + w2 * x).to_array());
        }

        for &u in us.iter() {
            let n = (-self.normal(u)).to_array();
            normals.push(n);
            normals.push(n);
        }

        println!("{:#?}", verts);

        let mut indices = (0..verts.len() as u32).collect::<Vec<_>>();
        let mut reversed: Vec<_> = indices.clone().into_iter().rev().skip(1).collect();
        indices.append(&mut reversed);

        println!("{:#?}", indices);

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
}

const fn catmull_rom_matrix(tau: f32) -> Mat4 {
    const_mat4!(
        // [0., -tau, 2. * tau, -tau],
        // [1., 0., tau - 3., 2. - tau],
        // [0., tau, 3. - 2. * tau, tau - 2.],
        // [0., 0., -tau, tau]
        [0., 1., 0., 0.],
        [-tau, 0., tau, 0.],
        [2. * tau, tau - 3., 3. - 2. * tau, -tau],
        [-tau, 2. - tau, tau - 2., tau]
    )
}

const TAU: f32 = 0.5;

pub struct CatmullRom3 {
    pub pts: Vec<Vec3>,
    pub coefs: Vec<Mat4>,
}

impl CatmullRom3 {
    pub fn new(pts: Vec<Vec3>) -> Self {
        let coefs = pts
            .iter()
            .map(|pt| pt.extend(0.))
            .tuple_windows()
            .map(|(p0, p1, p2, p3)| {
                Mat4::from_cols(p0, p1, p2, p3).mul_mat4(&catmull_rom_matrix(TAU))
            })
            .collect();

        Self { pts, coefs }
    }

    fn normalize(&self, mut u: f32) -> (f32, usize) {
        u *= self.coefs.len() as f32;
        let i = u.floor().clamp(0., self.coefs.len() as f32 - 1.);

        (u - i, i as usize)
    }
}

impl Curve for CatmullRom3 {
    fn p(&self, u: f32) -> Vec3 {
        let (u, i) = self.normalize(u);
        self.coefs[i]
            .mul_vec4(const_vec4!([1., u, u * u, u * u * u]))
            .truncate()
    }

    fn dp(&self, u: f32) -> Vec3 {
        let (u, i) = self.normalize(u);
        self.coefs[i]
            .mul_vec4(const_vec4!([0., 1., 2. * u, 3. * u * u]))
            .truncate()
            * self.coefs.len() as f32
    }

    fn d2p(&self, u: f32) -> Vec3 {
        let (u, i) = self.normalize(u);
        self.coefs[i]
            .mul_vec4(const_vec4!([0., 0., 2., 6. * u]))
            .truncate()
            * self.coefs.len().pow(2) as f32
    }
}
