use std::ops::{Add, Mul};

use bevy::math::*;
use bevy::prelude::{Mesh, Vec3};

use itertools::Itertools;

pub trait Frame {
    fn frame(&self, u: f32) -> Affine3A;
}

pub trait Resample {
    fn resample(&self, u_start: f32, u_stop: f32, ds: f32) -> Vec<f32>;
}

pub trait GenericCurve {
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
}

impl<T: GenericCurve> Frame for T {
    fn frame(&self, u: f32) -> Affine3A {
        Affine3A::from_mat3_translation(
            Mat3::from_cols(self.binormal(u), self.normal(u), self.tangent(u)),
            self.p(u),
        )
    }
}

impl<T: GenericCurve> Resample for T {
    fn resample(&self, u_start: f32, u_stop: f32, ds: f32) -> Vec<f32> {
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

impl GenericCurve for CatmullRom3 {
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

#[derive(Copy, Clone, Debug)]
pub struct HelicalPHQuinticSplineSegment {
    c0: f32,
    c2: f32,
    a0: Quat,
    a2: Quat,
    pi: Vec3,
    pf: Vec3,
}

impl HelicalPHQuinticSplineSegment {
    pub fn new(pi: Vec3, pf: Vec3, di: Vec3, df: Vec3) -> HelicalPHQuinticSplineSegment {
        let (u, v) = quat_cross_term_basis(di, df);
        let h = 120. * (pf - pi) - 15. * (di + df);

        let d4 = h.cross(df).dot(u) * di.cross(h).dot(u) - di.cross(df).dot(h - 5. * u).powi(2);
        let d3 = -2. * h.cross(df).dot(u) * di.cross(h).dot(v)
            - 2. * di.cross(h).dot(u) * h.cross(df).dot(v)
            - 20. * di.cross(df).dot(v) * di.cross(df).dot(h - 5. * u);
        let d2 = -2. * h.cross(df).dot(u) * di.cross(h).dot(u)
            + 4. * h.cross(df).dot(v) * di.cross(h).dot(v)
            - 100. * di.cross(df).dot(v).powi(2)
            - 2. * di.cross(df).dot(h - 5. * u) * di.cross(df).dot(h + 5. * u);
        let d1 = 2. * h.cross(df).dot(v) * di.cross(h).dot(u)
            + 2. * di.cross(h).dot(v) * h.cross(df).dot(u)
            - 20. * di.cross(df).dot(v) * di.cross(df).dot(h + 5. * u);
        let d0 = h.cross(df).dot(u) * di.cross(h).dot(u) - di.cross(df).dot(h + 5. * u).powi(2);

        // USe f64 precision for best results.
        // f32 can sometimes results in NaNs.
        let roots =
            roots::find_roots_quartic(d4 as f64, d3 as f64, d2 as f64, d1 as f64, d0 as f64);

        roots
            .as_ref()
            .iter()
            .map(|&t| t as f32)
            .flat_map(|t| {
                let cos = (1. - t.powi(2)) / (1. + t.powi(2));
                let sin = 2. * t / (1. + t.powi(2));
                let n = u * cos + v * sin;

                let k0sq = 0.0625 * h.cross(df).dot(n) / di.cross(df).dot(n);
                let k2sq = 0.0625 * di.cross(h).dot(n) / di.cross(df).dot(n);

                if k0sq > 0. && k2sq > 0. {
                    Some((t, k0sq.sqrt(), k2sq.sqrt(), n))
                } else {
                    None
                }
            })
            .flat_map(|(t, k0, k2, n)| {
                let x = di.cross(df).dot(h) / di.cross(df).dot(n) + 5.;
                if x > 0. {
                    [(t, k0, k2), (t, -k0, -k2)]
                } else {
                    [(t, -k0, k2), (t, k0, -k2)]
                }
            })
            .map(|(t, k0, k2)| (2. * t.atan(), k0 - 0.75, k2 - 0.75))
            .map(|(phi, c0, c2)| {
                let phi0 = 0.57;
                let phi2 = phi0 + phi;

                let (x0, y0) = inverse_solve_quat(di);
                let (x2, y2) = inverse_solve_quat(df);

                let a0 = x0 * phi0.cos() + y0 * phi0.sin();
                let a2 = x2 * phi2.cos() + y2 * phi2.sin();

                ((c0, c2), (a0, a2))
            })
            .map(|((c0, c2), (a0, a2))| HelicalPHQuinticSplineSegment {
                c0,
                c2,
                a0,
                a2,
                pi,
                pf,
            })
            .min_by_key(|h| ordered_float::OrderedFloat(h.curve().elastic_bending_energy()))
            .unwrap()
    }

    pub fn euler_rodrigues_frame(&self) -> EulerRodriguesFrame {
        EulerRodriguesFrame {
            data: *self,
            curve: self.curve(),
        }
    }

    pub fn curve(&self) -> HermiteQuintic {
        let a0len2 = self.a0.length_squared();
        let a2len2 = self.a2.length_squared();
        let a0a2 = 2. * self.a0.dot(self.a2);

        let w0 = a0len2;
        let w1 = self.c0 * a0len2 + 0.5 * self.c2 * a0a2;
        let w2 = (1. / 6.)
            * (4. * self.c0.powi(2) * a0len2
                + (1. + 4. * self.c0 * self.c2) * a0a2
                + 4. * self.c2.powi(2) * a2len2);
        let w3 = 0.5 * a0a2 + self.c2 * a2len2;
        let w4 = a2len2;

        const I: Quat = const_quat!([1., 0., 0., 0.]);

        let a0ia0 = (self.a0 * I * self.a0.conjugate()).xyz();
        let a2ia2 = (self.a2 * I * self.a2.conjugate()).xyz();
        let a0ia2 = (self.a0 * I * self.a2.conjugate() + self.a2 * I * self.a0.conjugate()).xyz();

        let wt0 = a0ia0;
        let wt1 = self.c0 * a0ia0 + 0.5 * self.c2 * a0ia2;
        let wt2 = (1. / 6.)
            * (4. * self.c0.powi(2) * a0ia0
                + (1. + 4. * self.c0 * self.c2) * a0ia2
                + 4. * self.c2.powi(2) * a2ia2);
        let wt3 = 0.5 * self.c0 * a0ia2 + self.c2 * a2ia2;
        let wt4 = a2ia2;

        HermiteQuintic {
            pi: self.pi,
            weights: [w0, w1, w2, w3, w4],
            weighted_tangents: [wt0, wt1, wt2, wt3, wt4],
        }
    }
}

pub struct EulerRodriguesFrame {
    data: HelicalPHQuinticSplineSegment,
    curve: HermiteQuintic,
}

impl Resample for EulerRodriguesFrame {
    fn resample(&self, u_start: f32, u_stop: f32, ds: f32) -> Vec<f32> {
        self.curve.resample(u_start, u_stop, ds)
    }
}

impl Frame for EulerRodriguesFrame {
    fn frame(&self, u: f32) -> Affine3A {
        let a = self.data.a0 * (1. - u).powi(2)
            + (self.data.a0 * self.data.c0 + self.data.a2 * self.data.c2) * u * (1. - u)
            + self.data.a2 * u.powi(2);

        let ai = (a * Quat::from_xyzw(1., 0., 0., 0.) * a.conjugate()).xyz() / a.length_squared();
        let aj = (a * Quat::from_xyzw(0., 1., 0., 0.) * a.conjugate()).xyz() / a.length_squared();
        let ak = (a * Quat::from_xyzw(0., 0., 1., 0.) * a.conjugate()).xyz() / a.length_squared();

        Affine3A::from_mat3_translation(Mat3::from_cols(ai, aj, ak), self.curve.p(u))
    }
}

#[derive(Clone, Debug)]
pub struct HermiteQuintic {
    pi: Vec3,
    weights: [f32; 5],
    weighted_tangents: [Vec3; 5],
}

impl HermiteQuintic {
    pub fn tangent(&self, u: f32) -> Vec3 {
        hermite_quintic_polynomial(self.weighted_tangents, u)
            / hermite_quintic_polynomial(self.weights, u)
    }

    pub fn speed(&self, u: f32) -> f32 {
        hermite_quintic_polynomial(self.weights, u)
    }

    pub fn p(&self, u: f32) -> Vec3 {
        self.pi + hermite_quintic_polynomial_integral(self.weighted_tangents, u)
            - hermite_quintic_polynomial_integral(self.weighted_tangents, 0.)
    }

    pub fn dp(&self, u: f32) -> Vec3 {
        hermite_quintic_polynomial(self.weighted_tangents, u)
    }

    pub fn d2p(&self, u: f32) -> Vec3 {
        hermite_quintic_polynomial_derivative(self.weighted_tangents, u)
    }

    pub fn elastic_bending_energy(&self) -> f32 {
        (0..1000)
            .map(|x| x as f32 * 0.001)
            .map(|u| self.curvature_squared(u) * self.speed(u) * 0.001)
            .sum()
    }

    fn curvature_squared(&self, u: f32) -> f32 {
        self.dp(u).cross(self.d2p(u)).length_squared()
            / hermite_quintic_polynomial(self.weights, u).powi(6)
    }
}

impl Resample for HermiteQuintic {
    fn resample(&self, u_start: f32, u_stop: f32, ds: f32) -> Vec<f32> {
        let mut u = u_start;
        let mut us = Vec::<f32>::new();
        while u < u_stop {
            us.push(u);
            u += ds / self.speed(u);
        }

        us
    }
}

pub fn ribbon_mesh<T: Frame + Resample>(
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

// Generates the basis for the one-parameter family of solutions to AiA* = c.
// This function returns the quaternions A1, A2 such that any solution
// to AiA* = c can be generated by the solution A1 cos t + A2 sin t.
fn inverse_solve_quat(c: Vec3) -> (Quat, Quat) {
    let [lambda, mu, nu] = c.normalize().to_array();
    let r = (0.5 * (1. + lambda) * c.length()).sqrt();
    (
        quat(1., mu / (1. + lambda), nu / (1. + lambda), 0.) * r,
        quat(0., nu / (1. + lambda), -mu / (1. + lambda), -1.) * r,
    )
}

fn quat_cross_term_basis(c0: Vec3, c1: Vec3) -> (Vec3, Vec3) {
    let [l0, mu0, nu0] = c0.normalize().to_array();
    let [l1, mu1, nu1] = c1.normalize().to_array();

    let (l0p, l1p) = (1. + l0, 1. + l1);
    let r = (l0p * l1p).sqrt();
    let s = (c0.length() * c1.length()).sqrt();

    (
        vec3(
            l1p * l0p - (mu0 * mu1 + nu0 * nu1),
            l1p * mu0 + l0p * mu1,
            l1p * nu0 + l0p * nu1,
        ) / r
            * s,
        vec3(
            mu1 * nu0 - mu0 * nu1,
            l0p * nu1 - l1p * nu0,
            l1p * mu0 - l0p * mu1,
        ) / r
            * s,
    )
}

fn hermite_quintic_polynomial<T: Copy + Add<Output = T> + Mul<f32, Output = T>>(
    coefs: [T; 5],
    t: f32,
) -> T {
    coefs[0] * (1. - t).powi(4)
        + coefs[1] * (1. - t).powi(3) * t * 4.
        + coefs[2] * ((1. - t) * t).powi(2) * 6.
        + coefs[3] * (1. - t) * t.powi(3) * 4.
        + coefs[4] * t.powi(4)
}

fn hermite_quintic_polynomial_integral<T: Copy + Add<Output = T> + Mul<f32, Output = T>>(
    coefs: [T; 5],
    t: f32,
) -> T {
    coefs[0] * 0.2 * (t - 1.).powi(5)
        + coefs[1] * (-0.2 * t.powi(5) + 0.75 * t.powi(4) - t.powi(3) + 0.5 * t.powi(2)) * 4.
        + coefs[2] * (0.2 * t.powi(5) - 0.5 * t.powi(4) + t.powi(3) / 3.) * 6.
        + coefs[3] * (0.25 * t.powi(4) - 0.2 * t.powi(5)) * 4.
        + coefs[4] * 0.2 * t.powi(5)
}

fn hermite_quintic_polynomial_derivative<T: Copy + Add<Output = T> + Mul<f32, Output = T>>(
    coefs: [T; 5],
    t: f32,
) -> T {
    coefs[0] * -4. * (1. - t).powi(3)
        + coefs[1] * (1. - 4. * t) * (1. - t).powi(2) * 4.
        + coefs[2] * 2. * (t - 1.) * t * (2. * t - 1.) * 6.
        + coefs[3] * (3. - 4. * t) * t.powi(2) * 4.
        + coefs[4] * 4. * t.powi(3)
}
