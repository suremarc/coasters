use bevy::math::*;
use bevy::prelude::{Mesh, Vec3};

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
        let mut us = self.equidistant_resampling(u_start, u_end, ds);
        // us.resize(2, 0.);
        let ps: Vec<Vec3> = us.iter().map(|&u| self.p(u)).collect();
        let xs: Vec<Vec3> = us.iter().map(|&u| self.binormal(u)).collect();

        let mut verts: Vec<[f32; 3]> = Vec::with_capacity(us.len() * 2);

        let w2 = width / 2.;
        for (p, x) in ps.iter().zip(xs) {
            verts.push((*p - w2 * x).to_array());
            verts.push((*p + w2 * x).to_array());
        }
        println!("{:#?}", verts);

        // let mut indices: Vec<u32> = Vec::with_capacity((us.len() - 1) * 2);
        #[allow(clippy::never_loop)]
        let indices = (0..verts.len() as u32).collect::<Vec<_>>();

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
        mesh.set_indices(Some(bevy::render::mesh::Indices::U32(indices)));
        mesh
    }
}
