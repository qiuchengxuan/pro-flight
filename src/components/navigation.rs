use alloc::boxed::Box;
use alloc::rc::Rc;

use nalgebra::Vector3;

use crate::algorithm::runge_kutta4;
use crate::alloc;
use crate::components::schedule::{Rate, Schedulable};
use crate::datastructures::coordinate::{Displacement, Position};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{AgingStaticData, DataWriter, OptionData};
use crate::datastructures::measurement::displacement::DistanceVector;
use crate::datastructures::measurement::unit::CentiMeter;
use crate::datastructures::measurement::{Altitude, GRAVITY};
use crate::datastructures::waypoint::{Steerpoint, Waypoint};

const CURRENT: usize = 0;
const HOME: usize = 0;
const MAX_WAYPOINT: usize = 32;

pub struct Navigation<A, ACCEL> {
    altimeter: A,
    accelerometer: ACCEL,
    gnss: Option<Box<dyn AgingStaticData<Position>>>,
    waypoints: [Waypoint; MAX_WAYPOINT],
    offset: DistanceVector<f32, CentiMeter>,
    displacements: [Displacement; MAX_WAYPOINT],
    output: Rc<SingularData<(Position, Steerpoint)>>,
    next_waypoint: u8,
    max_waypoint: u8,
    interval: f32,
    time: f32,
}

impl<A, ACCEL> Navigation<A, ACCEL> {
    pub fn new(altimeter: A, accelerometer: ACCEL, interval: f32) -> Self {
        Self {
            altimeter,
            accelerometer,
            gnss: None,
            waypoints: [Waypoint::default(); MAX_WAYPOINT],
            offset: Default::default(),
            displacements: [Displacement::default(); MAX_WAYPOINT],
            output: Rc::new(SingularData::default()),
            next_waypoint: HOME as u8,
            max_waypoint: 1,
            interval,
            time: 0.0,
        }
    }

    pub fn reader(&self) -> SingularDataSource<(Position, Steerpoint)> {
        SingularDataSource::new(&self.output)
    }

    pub fn set_gnss(&mut self, gnss: Box<dyn AgingStaticData<Position>>) {
        self.gnss = Some(gnss)
    }

    pub fn next_waypoint(&mut self) {
        self.next_waypoint = (self.next_waypoint + 1) % self.max_waypoint;
    }

    pub fn update_waypoint(&mut self, index: usize, waypoint: Waypoint) {
        if index >= MAX_WAYPOINT || index != self.max_waypoint as usize + 1 {
            return;
        }
        self.waypoints[index] = waypoint;
        self.max_waypoint += 1;
        if index != HOME {
            self.displacements[index] = waypoint.position - self.waypoints[HOME].position;
        }
    }
}

impl<A, ACCEL> Schedulable for Navigation<A, ACCEL>
where
    A: OptionData<Altitude>,
    ACCEL: OptionData<Vector3<f32>>,
{
    fn rate(&self) -> Rate {
        50
    }

    fn schedule(&mut self) -> bool {
        let rate = self.rate();
        // TODO: complementary filter
        if let Some(position) = self.gnss.as_mut().map(|gnss| gnss.read(rate)).flatten() {
            if self.waypoints[HOME].position.latitude.0 == 0 {
                self.waypoints[HOME].position = position;
            }
            self.offset = (self.waypoints[HOME].position - position).convert(|v| v as f32);
            self.time = 0.0;
            return true;
        }

        let altimeter = self.altimeter.read();

        while let Some(mut acceleration) = self.accelerometer.read() {
            acceleration *= GRAVITY;
            let (ax, ay, az) = (acceleration[0], acceleration[1], acceleration[2] + GRAVITY);
            let (t, dt) = (self.time, self.interval);
            let mut offset = self.offset;
            // TODO: complementary filter
            offset.x.value = runge_kutta4(|_, dt| ax * dt, offset.x.value(), t, dt);
            offset.y.value = runge_kutta4(|_, dt| ay * dt, offset.y.value(), t, dt);
            if let Some(altitude) = altimeter {
                if self.waypoints[HOME].position.altitude.is_zero() {
                    self.waypoints[HOME].position.altitude = altitude;
                }
                let height = altitude - self.waypoints[HOME].position.altitude;
                self.offset.z = height.convert(|v| v as f32);
            } else {
                offset.z.value = runge_kutta4(|_, dt| az * dt, offset.z.value(), t, dt);
            }
            self.offset = offset;
            self.displacements[CURRENT] = offset.convert(|v| v as i32);
            self.time += self.interval;
            let waypoint = self.waypoints[self.next_waypoint as usize];
            let steerpoint = Steerpoint { index: self.next_waypoint, waypoint };
            let position = self.waypoints[HOME].position + self.displacements[CURRENT];
            self.output.write((position, steerpoint));
        }
        true
    }
}
