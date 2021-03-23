use nalgebra::Vector3;

use crate::algorithm::ComplementaryFilter;
use crate::config;
use crate::datastructures::measurement::distance::Distance;
use crate::datastructures::measurement::unit::{Meter, MilliMeter};
use crate::datastructures::measurement::{Altitude, VelocityVector, GRAVITY};
use crate::sync::{AgingDataReader, DataReader};

pub struct Speedometer<A, GNSS> {
    altimeter: A,
    gnss: GNSS,
    gnss_aging: usize,
    interval: f32,
    altitude_delta: Distance<f32, Meter>,
    filters: [ComplementaryFilter<f32>; 3],
    acceleration: Vector3<f32>,
    altitude: Altitude,
    vector: (f32, f32, f32),
}

impl<A, GNSS> Speedometer<A, GNSS>
where
    A: DataReader<Altitude>,
    GNSS: AgingDataReader<VelocityVector<i32, MilliMeter>>,
{
    pub fn new(altimeter: A, gnss: GNSS, sample_rate: usize, gnss_rate: usize) -> Self {
        let config = &config::get().speedometer;
        Self {
            altimeter,
            gnss,
            gnss_aging: sample_rate / gnss_rate,
            interval: 1.0 / sample_rate as f32,
            acceleration: Vector3::new(0.0, 0.0, 0.0),
            altitude_delta: Default::default(),
            filters: [ComplementaryFilter::new(config.kp.into(), 1.0 / sample_rate as f32); 3],
            altitude: Altitude::default(),
            vector: (0.0, 0.0, 0.0),
        }
    }

    pub fn update(&mut self, a: Vector3<f32>) -> VelocityVector<f32, Meter> {
        if let Some(altitude) = self.altimeter.get() {
            self.altitude_delta = (altitude - self.altitude).convert(|v| v as f32).to_unit(Meter);
            self.altitude = altitude;
        }
        #[rustfmt::skip]
        let gnss = self.gnss.get_aging_last(self.gnss_aging)
            .map(|velocity| velocity.convert(|v| v as f32).to_unit(Meter));

        let a = a * GRAVITY;
        self.acceleration = a;
        if let Some(vector) = gnss {
            self.vector.0 = self.filters[0].filter(vector.x.value(), a[0]);
            self.vector.1 = self.filters[1].filter(vector.y.value(), a[1]);
        } else {
            self.vector.0 += (a[0] + (a[0] - self.acceleration[0]) / 2.0) * self.interval;
            self.vector.1 += (a[1] + (a[1] - self.acceleration[1]) / 2.0) * self.interval;
        }
        self.vector.2 = self.filters[2].filter(self.altitude_delta.value(), a[2] + GRAVITY);
        VelocityVector::new(self.vector.0, self.vector.1, self.vector.2, Meter)
    }
}
