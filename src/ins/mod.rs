pub mod out;
pub mod positioning;
pub mod speedometer;
pub mod variometer;

use fugit::NanosDurationU64 as Duration;

use positioning::Positioning;
use speedometer::Speedometer;
use variometer::Variometer;

use crate::{datastore, types::coordinate::Position};

pub struct INS {
    interval: Duration,
    variometer: Variometer,
    speedometer: Speedometer,
    positioning: Positioning,
    initial: Option<Position>,
}

impl INS {
    pub fn new(sample_rate: usize, variometer: Variometer) -> Self {
        let interval = Duration::micros(1000_000 / sample_rate as u64);
        let speedometer = Speedometer::new(sample_rate);
        let positioning = Positioning::new(sample_rate);
        Self { interval, variometer, speedometer, positioning, initial: None }
    }

    pub fn update(&mut self) {
        let ds = datastore::acquire();
        let gnss = ds.read_gnss_within(self.interval);
        let imu = ds.read_imu();

        let altitude = ds.read_altitude_within(self.interval);
        let vs = altitude.map(|alt| self.variometer.update(alt));
        let vv = self.speedometer.update(imu.acceleration.0.raw, vs, gnss);
        let gnss_position = gnss.map(|g| g.fixed.map(|f| f.position)).flatten();
        if self.initial.is_none() && gnss_position.is_some() {
            self.initial = gnss_position;
        }
        self.positioning.update(vv, altitude, gnss_position);
        let p = self.positioning.position();
        let d = self.positioning.displacement();
        let ins = out::INS { velocity_vector: vv, position: p, displacement: d };
        ds.write_ins(ins);
    }
}
