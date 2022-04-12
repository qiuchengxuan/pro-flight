use crate::types::{
    coordinate::Position,
    measurement::{
        unit::{CentiMeter, Meter, Ms},
        Altitude, Displacement, Distance, VelocityVector, ENU, Z,
    },
};

#[derive(PartialEq)]
enum AltitudeSource {
    Barometer,
    GNSS,
}

pub struct Positioning {
    interval: f32,
    velocity_vector: VelocityVector<f32, Ms, ENU>,
    initial: Position,
    displacement: Displacement<f32, Meter, ENU>, // relative to initial position
    altitude_source: AltitudeSource,
}

impl Positioning {
    pub fn new(update_rate: usize) -> Self {
        Self {
            interval: 1.0 / update_rate as f32,
            velocity_vector: Default::default(),
            initial: Default::default(),
            displacement: Default::default(),
            altitude_source: AltitudeSource::GNSS,
        }
    }

    pub fn update(
        &mut self,
        v: VelocityVector<f32, Ms, ENU>,
        altitude: Option<Altitude>,
        gnss: Option<Position>,
    ) {
        if let Some(mut altitude) = altitude {
            if self.altitude_source == AltitudeSource::GNSS {
                self.altitude_source = AltitudeSource::Barometer;
                if altitude.is_zero() {
                    altitude += Distance::new(1, CentiMeter);
                }
                self.initial.altitude = altitude;
            }
            let height = self.initial.altitude - altitude;
            self.displacement.raw[Z] = height.t(|v| v as f32).u(Meter).raw;
        }
        if let Some(position) = gnss {
            if self.initial.latitude.0 == 0 {
                let altitude = self.initial.altitude;
                self.initial = position;
                if !altitude.is_zero() {
                    self.initial.altitude = altitude;
                }
            }
            let displacement = position - self.initial;
            let z = self.displacement.raw[Z];
            self.displacement = displacement.t(|v| v as f32).u(Meter);
            if self.altitude_source == AltitudeSource::Barometer {
                self.displacement.raw[Z] = z;
            }
        }
        let integral = (self.velocity_vector + v) / 2.0 * self.interval;
        self.displacement.raw += integral.u(Meter).raw;
        self.velocity_vector = v;
    }

    pub fn displacement(&self) -> Displacement<f32, Meter, ENU> {
        self.displacement
    }

    pub fn position(&self) -> Position {
        self.initial + self.displacement.u(CentiMeter).t(|v| v as i32)
    }
}
