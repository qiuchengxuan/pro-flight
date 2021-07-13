pub mod ascii_hud;
pub mod cli;
pub mod flight_data;
#[macro_use]
pub mod logger;
pub mod configuration;
pub mod imu;
pub mod mixer;
pub mod positioning;
pub mod speedometer;
pub mod variometer;

use imu::IMU;
use positioning::Positioning;
use speedometer::Speedometer;

use crate::datastructures::measurement::{Acceleration, Gyro};
use crate::sync::DataWriter;

pub fn imu_handler<'a>(hub: &'a flight_data::FlightDataHUB) -> impl FnMut(Acceleration, Gyro) + 'a {
    let reader = hub.reader();
    let (heading, course) = (reader.gnss_heading, reader.gnss_course);
    let mut imu = IMU::new(reader.magnetometer, heading, course, 1000, 1000 / 10);
    let mut speedometer = Speedometer::new(reader.vertical_speed, reader.gnss_velocity, 1000, 10);
    let mut positioning = Positioning::new(reader.altimeter, reader.gnss_position, 1000);
    let (accelerometer, gyroscope) = (&hub.accelerometer, &hub.gyroscope);
    let (quat, speed) = (&hub.imu, &hub.speedometer);
    let (position, displacement) = (&hub.positioning, &hub.displacement);
    move |accel, gyro| {
        accelerometer.write(accel);
        gyroscope.write(gyro);
        if imu.update_imu(&accel, &gyro) {
            quat.write(imu.quaternion());
            let v = speedometer.update(imu.acceleration());
            speed.write(v);
            let (p, d) = positioning.update(v);
            position.write(p);
            displacement.write(d)
        }
    }
}
