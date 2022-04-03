use bevy::math::*;
use bevy::prelude::Vec3;

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
}
