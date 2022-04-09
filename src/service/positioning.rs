use crate::{
    service::info,
    types::{
        coordinate::{Displacement, Position},
        measurement::{displacement::DistanceVector, unit, Altitude, VelocityVector},
    },
};

pub struct Positioning<A, GNSS> {
    altimeter: A,
    gnss: GNSS,
    interval: f32,
    velocity: VelocityVector<f32, unit::MpS>,
    current: Position,                              // updated from GNSS
    displacement: DistanceVector<f32, unit::Meter>, // relative to current position
}

type Output = (Position, Displacement<unit::CentiMeter>);

impl<A, GNSS> Positioning<A, GNSS>
where
    A: info::Reader<Altitude>,
    GNSS: info::AgingReader<Position> + info::Reader<Position>,
{
    pub fn new(altimeter: A, gnss: GNSS, update_rate: usize) -> Self {
        Self {
            altimeter,
            gnss,
            interval: 1.0 / update_rate as f32,
            velocity: Default::default(),
            current: Default::default(),
            displacement: Default::default(),
        }
    }

    pub fn update(&mut self, v: VelocityVector<f32, unit::MpS>) -> Output {
        if let Some(position) = self.gnss.get() {
            self.current = position;
            self.displacement = DistanceVector::default();
        } else {
            if let Some(altitude) = self.altimeter.get() {
                self.current.altitude = altitude;
                self.displacement.z = Default::default();
            }
            let integral = (self.velocity + (v - self.velocity) / 2.0) * self.interval;
            self.displacement += integral.to_unit(unit::Meter);
        }
        self.velocity = v;
        let displacement = self.displacement.to_unit(unit::CentiMeter).convert(|v| v as i32);
        (self.current + displacement, displacement)
    }
}
