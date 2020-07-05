use nalgebra::{Quaternion, UnitQuaternion, Vector3};

use crate::datastructures::coordinate::Position;
use crate::hal::sensors::Acceleration;
use crate::math::runge_kutta4;

const HOME: usize = 0;

pub struct Navigation {
    waypoints: [Position; 32],
    current: (f32, f32, f32), // ref to home, unit meter
    next_waypoint: usize,
    interval: f32,
    time: f32,
}

impl Navigation {
    pub fn new(interval: f32) -> Self {
        Self {
            waypoints: [Position::default(); 32],
            current: (0.0, 0.0, 0.0),
            next_waypoint: HOME,
            interval,
            time: 0.0,
        }
    }

    pub fn set_waypoint(&mut self, index: usize) {
        self.next_waypoint = index;
    }

    pub fn update_waypoint(&mut self, index: usize, position: Position) {
        self.waypoints[index] = position;
    }

    pub fn update_from_imu(&mut self, acceleration: Acceleration, quaternion: Quaternion<f32>) {
        let vector: Vector3<f32> = acceleration.into();
        let unit = UnitQuaternion::new_normalize(quaternion);
        let vector = unit.inverse_transform_vector(&vector) * 9.8;
        let (ax, ay, az) = (vector[0], vector[1], vector[2]);
        let (x, y, z) = self.current;
        let x = runge_kutta4(|_, dt| ax * dt, x, self.time, self.interval);
        let y = runge_kutta4(|_, dt| ay * dt, y, self.time, self.interval);
        let z = runge_kutta4(|_, dt| az * dt, z, self.time, self.interval);
        self.time += self.interval;
        self.current = (x, y, z);
    }

    pub fn update_from_gnss(&mut self, position: Position) {
        let delta = self.waypoints[HOME] - position;
        self.current = delta.into();
        self.time = 0.0;
    }
}
