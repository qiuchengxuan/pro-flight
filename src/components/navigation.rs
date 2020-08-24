use alloc::boxed::Box;
use alloc::rc::Rc;

use nalgebra::UnitQuaternion;

use crate::algorithm::runge_kutta4;
use crate::alloc;
use crate::components::schedule::{Rate, Schedulable};
use crate::datastructures::coordinate::{Displacement, Position};
use crate::datastructures::data_source::singular::{SingularData, SingularDataSource};
use crate::datastructures::data_source::{AgingStaticData, DataWriter, OptionData, StaticData};
use crate::datastructures::measurement::unit::Meter;
use crate::datastructures::measurement::{Acceleration, Altitude, Velocity};
use crate::datastructures::waypoint::{Steerpoint, Waypoint};

const CURRENT: usize = 0;
const HOME: usize = 0;
const MAX_WAYPOINT: usize = 32;
const GRAVITY: f32 = 9.80665;

pub struct Navigation<IMU, A> {
    imu: IMU,
    accelerometer: A,
    gnss: Option<Box<dyn AgingStaticData<Position>>>,
    altimeter: Option<Box<dyn StaticData<(Altitude, Velocity<i16, Meter>)>>>,
    waypoints: [Waypoint; MAX_WAYPOINT],
    offset: (f32, f32, f32),
    displacements: [Displacement; MAX_WAYPOINT],
    output: Rc<SingularData<(Position, Steerpoint)>>,
    next_waypoint: u8,
    max_waypoint: u8,
    interval: f32,
    time: f32,
}

impl<IMU: OptionData<UnitQuaternion<f32>>, A: OptionData<Acceleration>> Navigation<IMU, A> {
    pub fn new(imu: IMU, accelerometer: A, interval: f32) -> Self {
        Self {
            imu,
            accelerometer,
            gnss: None,
            altimeter: None,
            waypoints: [Waypoint::default(); MAX_WAYPOINT],
            offset: (0.0, 0.0, 0.0),
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

    pub fn set_altimeter(
        &mut self,
        altimeter: Box<dyn StaticData<(Altitude, Velocity<i16, Meter>)>>,
    ) {
        self.altimeter = Some(altimeter)
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

impl<IMU, A> Schedulable for Navigation<IMU, A>
where
    IMU: OptionData<UnitQuaternion<f32>>,
    A: OptionData<Acceleration>,
{
    fn rate(&self) -> Rate {
        50
    }

    fn schedule(&mut self) -> bool {
        let rate = self.rate();
        let gnss = self.gnss.as_mut().map(|gnss| gnss.read(rate)).flatten();
        if let Some(position) = gnss {
            if self.waypoints[HOME].position.latitude == 0 {
                self.waypoints[HOME].position = position;
            }
            self.offset = (self.waypoints[HOME].position - position).into_f32();
            self.time = 0.0;
            return true;
        }

        while let Some(unit_quaternion) = self.imu.read() {
            let acceleration = self.accelerometer.read().unwrap();
            let vector = unit_quaternion.transform_vector(&acceleration.0.into()) * GRAVITY;
            let (ax, ay, az) = (vector[0], vector[1], vector[2] - GRAVITY); // z axis reverted
            let (t, dt) = (self.time, self.interval);
            let mut offset = self.offset;
            // TODO: add speed data source or runge-kutta won't properly work
            offset.0 = runge_kutta4(|_, dt| ax * dt, offset.0, t, dt);
            offset.1 = runge_kutta4(|_, dt| ay * dt, offset.1, t, dt);
            let altimeter = self.altimeter.as_mut().map(|a| a.read());
            if let Some((altitude, _)) = altimeter {
                if self.waypoints[HOME].position.altitude.is_zero() {
                    self.waypoints[HOME].position.altitude = altitude;
                }
                let height = altitude - self.waypoints[HOME].position.altitude;
                self.offset.2 = height.to_unit(Meter).value() as f32;
            } else {
                offset.2 = runge_kutta4(|_, dt| az * dt, offset.2, t, dt);
            }
            self.offset = offset;
            self.displacements[CURRENT] = offset.into();
            self.time += self.interval;
            let waypoint = self.waypoints[self.next_waypoint as usize];
            let steerpoint = Steerpoint { index: self.next_waypoint, waypoint };
            let position = self.waypoints[HOME].position + self.displacements[CURRENT];
            self.output.write((position, steerpoint));
        }
        true
    }
}
