use crate::types::{
    coordinate::Position,
    measurement::{
        unit::{CentiMeter, Meter, Ms},
        Altitude, Displacement, VelocityVector, ENU, Z,
    },
};

pub struct Positioning {
    interval: f32,
    velocity_vector: VelocityVector<f32, Ms, ENU>,
    initial: Position,
    displacement: Displacement<f32, Meter, ENU>, // relative to initial position
}

impl Positioning {
    pub fn new(update_rate: usize) -> Self {
        Self {
            interval: 1.0 / update_rate as f32,
            velocity_vector: Default::default(),
            initial: Default::default(),
            displacement: Default::default(),
        }
    }

    pub fn update(
        &mut self,
        v: VelocityVector<f32, Ms, ENU>,
        altitude: Option<Altitude>,
        gnss: Option<Position>,
    ) {
        if let Some(altitude) = altitude {
            if self.initial.altitude.is_zero() {
                self.initial.altitude = altitude;
            }
            let height = altitude - self.initial.altitude;
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
            self.displacement.raw[Z] = z;
        }
        let integral = (self.velocity_vector + v) / 2.0 * self.interval;
        self.displacement.raw += integral.u(Meter).raw;
        self.velocity_vector = v;
    }

    pub fn displacement(&self) -> Displacement<f32, Meter, ENU> {
        self.displacement
    }

    pub fn initial_position(&self) -> Position {
        self.initial
    }

    pub fn position(&self) -> Position {
        self.initial + self.displacement.u(CentiMeter).t(|v| v as i32)
    }
}
