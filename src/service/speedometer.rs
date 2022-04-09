use nalgebra::Vector3;

use crate::{
    algorithm::ComplementaryFilter,
    config,
    service::info,
    types::measurement::{unit, Velocity, VelocityVector, GRAVITY},
};

pub struct Speedometer<A, GNSS> {
    altimeter: A,
    gnss: GNSS,
    gnss_aging: usize,
    interval: f32,
    filters: [ComplementaryFilter<f32>; 3],
    acceleration: Vector3<f32>,
    velocity: VelocityVector<f32, unit::MpS>,
}

impl<A, GNSS> Speedometer<A, GNSS>
where
    A: info::Reader<Velocity<i32, unit::CMpS>>,
    GNSS: info::AgingReader<VelocityVector<i32, unit::MMpS>>,
{
    pub fn new(altimeter: A, gnss: GNSS, sample_rate: usize, gnss_rate: usize) -> Self {
        let config = &config::get().ins.speedometer;
        Self {
            altimeter,
            gnss,
            gnss_aging: sample_rate / gnss_rate,
            interval: 1.0 / sample_rate as f32,
            filters: [ComplementaryFilter::new(config.kp.into(), 1.0 / sample_rate as f32); 3],
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            velocity: Default::default(),
        }
    }

    pub fn update(&mut self, acceleration: Vector3<f32>) -> VelocityVector<f32, unit::MpS> {
        let mut a = acceleration * GRAVITY;
        a[2] += GRAVITY;
        if let Some(velocity) = self.gnss.get_aging_last(self.gnss_aging) {
            let v = velocity.convert(|v| v as f32).to_unit(unit::MpS);
            self.velocity.x.value = self.filters[0].filter(v.x.value(), a[0]);
            self.velocity.y.value = self.filters[1].filter(v.y.value(), a[1]);
            self.velocity.z.value = self.filters[2].filter(v.z.value(), a[2]);
        } else if let Some(vertical_speed) = self.altimeter.get_last() {
            self.velocity.x.value += (a[0] + (a[0] - self.acceleration[0]) / 2.0) * self.interval;
            self.velocity.y.value += (a[1] + (a[1] - self.acceleration[1]) / 2.0) * self.interval;
            let vs = vertical_speed.convert(|v| v as f32).to_unit(unit::MpS).value();
            self.velocity.z.value = self.filters[2].filter(vs, a[2]);
        } else {
            let v = (a + (a - self.acceleration) / 2.0) * self.interval;
            self.velocity += VelocityVector::new(v[0], v[1], v[2], unit::MpS);
        }
        self.acceleration = a;
        self.velocity
    }
}
