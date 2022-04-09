use nalgebra::UnitQuaternion;

pub mod positioning;
pub mod speedometer;
pub mod variometer;

use positioning::Positioning;
use speedometer::Speedometer;

use crate::{
    algorithm::imu,
    config,
    service::{
        flight::data::FlightDataHUB,
        info::{
            bulletin::{Bulletin, BulletinReader},
            AgingReader, Reader, Writer,
        },
    },
    types::{
        coordinate::{Displacement, Position},
        measurement::{
            unit, Acceleration, Altitude, Course, Gyro, Heading, Magnetism, Velocity,
            VelocityVector,
        },
    },
};

type VelocityMeter<'a> = BulletinReader<'a, Velocity<i32, unit::CMpS>>;
type GNSSSpeedometer<'a> = BulletinReader<'a, VelocityVector<i32, unit::MMpS>>;
type Altimeter<'a> = BulletinReader<'a, Altitude>;
type GNSS<'a> = BulletinReader<'a, Position>;

pub struct INS<'a> {
    aging: usize,
    // input
    acceleration: BulletinReader<'a, Acceleration>,
    gyro: BulletinReader<'a, Gyro>,
    magnetometer: BulletinReader<'a, Magnetism>,
    heading: BulletinReader<'a, Heading>,
    course: BulletinReader<'a, Course>,
    // data process
    imu: imu::IMU,
    speedometer: Speedometer<VelocityMeter<'a>, GNSSSpeedometer<'a>>,
    positioning: Positioning<Altimeter<'a>, GNSS<'a>>,
    // output
    quaternion: &'a Bulletin<UnitQuaternion<f32>>,
    velocity: &'a Bulletin<VelocityVector<f32, unit::MpS>>,
    position: &'a Bulletin<Position>,
    displacement: &'a Bulletin<Displacement<unit::CentiMeter>>,
}

impl<'a> INS<'a> {
    pub fn new(sample_rate: usize, hub: &'a FlightDataHUB) -> Self {
        let config = config::get();
        let reader = hub.reader();
        let speedometer =
            Speedometer::new(reader.vertical_speed, reader.gnss_velocity, sample_rate, 10);
        let positioning = Positioning::new(reader.altimeter, reader.gnss_position, sample_rate);
        Self {
            aging: sample_rate / 10,
            acceleration: reader.accelerometer,
            gyro: reader.gyroscope,
            magnetometer: reader.magnetometer,
            heading: reader.gnss_heading,
            course: reader.gnss_course,
            imu: imu::IMU::new(sample_rate, &config.imu),
            speedometer,
            positioning,
            quaternion: &hub.imu,
            velocity: &hub.speedometer,
            position: &hub.positioning,
            displacement: &hub.displacement,
        }
    }

    pub fn invoke(&mut self) {
        let acceleration = self.acceleration.get_last().unwrap();
        let gyro = self.gyro.get_last().unwrap();
        let magnetism = self.magnetometer.get_last();
        let aging = self.aging;
        let heading = self.heading.get_aging_last(aging).and(self.course.get_aging_last(aging));
        if self.imu.update_imu(acceleration, gyro, magnetism, heading) {
            self.quaternion.write(self.imu.quaternion());
            let v = self.speedometer.update(self.imu.acceleration());
            self.velocity.write(v);
            let (p, d) = self.positioning.update(v);
            self.position.write(p);
            self.displacement.write(d)
        }
    }
}
