use bevy::math::*;
use bevy::prelude::Vec3;

use itertools::Itertools;

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

impl crate::curve::Curve for CatmullRom3 {
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
