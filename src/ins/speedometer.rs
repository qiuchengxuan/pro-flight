use core::f32::consts::PI;

use nalgebra::Vector3;

use crate::{
    algorithm::ComplementaryFilter,
    config,
    protocol::serial::gnss::out::GNSS,
    types::measurement::{unit, Velocity, VelocityVector, ENU, GRAVITY, X, Y, Z},
};

pub struct Speedometer {
    interval: f32,
    filters: [ComplementaryFilter<f32>; 3],
    acceleration: Vector3<f32>,
    velocity_vector: VelocityVector<f32, unit::Ms, ENU>,
}

type Vario = Velocity<i32, unit::CMs>;
type VV = VelocityVector<f32, unit::Ms, ENU>;

impl Speedometer {
    pub fn new(sample_rate: usize) -> Self {
        let config = &config::get().ins.speedometer;
        Self {
            interval: 1.0 / sample_rate as f32,
            filters: [ComplementaryFilter::new(config.kp.into(), 1.0 / sample_rate as f32); 3],
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            velocity_vector: Default::default(),
        }
    }

    pub fn update(&mut self, accel: Vector3<f32>, vario: Option<Vario>, gnss: Option<GNSS>) -> VV {
        let mut a = accel * GRAVITY;
        a[2] += GRAVITY;
        let mut vv: nalgebra::Vector3<f32> = self.velocity_vector.into();
        if let Some(gnss) = gnss.map(|v| v.fixed.map(|f| f.velocity_vector).flatten()).flatten() {
            let gnss: nalgebra::Vector3<f32> = gnss.t(|v| v as f32).u(unit::Ms).into();
            vv[X] = self.filters[X].filter(gnss[X], a[X]);
            vv[Y] = self.filters[Y].filter(gnss[Y], a[Y]);
        } else if let Some(fixed) = gnss.map(|v| v.fixed).flatten() {
            let course = Into::<f32>::into(fixed.course.0) / PI / 2.0;
            let gs = fixed.ground_speed.t(|v| v as f32).u(unit::Ms).raw;
            let x = libm::sinf(course) * gs;
            let y = libm::cosf(course) * gs;
            vv[X] = self.filters[X].filter(x, a[X]);
            vv[Y] = self.filters[Y].filter(y, a[Y]);
        } else {
            vv[X] += (a[X] + self.acceleration[X]) / 2.0 * self.interval;
            vv[Y] += (a[Y] + self.acceleration[Y]) / 2.0 * self.interval;
        }
        if let Some(vario) = vario {
            let vs = vario.t(|v| v as f32).u(unit::Ms).raw;
            vv[Z] = self.filters[Z].filter(vs, a[Z]);
        } else {
            vv[Z] += (a[Z] + self.acceleration[Z]) / 2.0 * self.interval;
        }
        self.velocity_vector = VelocityVector::from(vv, unit::Ms, ENU);
        self.acceleration = a;
        self.velocity_vector
    }
}
