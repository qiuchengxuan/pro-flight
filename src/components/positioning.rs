use crate::datastructures::{
    coordinate::{Displacement, Position},
    measurement::{
        displacement::DistanceVector,
        unit::{CentiMeter, Meter},
        Altitude, VelocityVector,
    },
};
use crate::sync::{AgingDataReader, DataReader};

pub struct Positioning<A, GNSS> {
    altimeter: A,
    gnss: GNSS,
    interval: f32,
    velocity: VelocityVector<f32, Meter>,
    current: Position,                        // updated from GNSS
    displacement: DistanceVector<f32, Meter>, // relative to current position
}

type Output = (Position, Displacement<CentiMeter>);

impl<A, GNSS> Positioning<A, GNSS>
where
    A: DataReader<Altitude>,
    GNSS: AgingDataReader<Position> + DataReader<Position>,
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

    pub fn update(&mut self, v: VelocityVector<f32, Meter>) -> Output {
        if let Some(position) = self.gnss.get() {
            self.current = position;
            self.displacement = DistanceVector::default();
        } else {
            if let Some(altitude) = self.altimeter.get() {
                self.current.altitude = altitude;
                self.displacement.z = Default::default();
            }
            self.displacement += (self.velocity + (v - self.velocity) / 2.0) * self.interval;
        }
        self.velocity = v;
        let displacement = self.displacement.to_unit(CentiMeter).convert(|v| v as i32);
        (self.current + displacement, displacement)
    }
}
