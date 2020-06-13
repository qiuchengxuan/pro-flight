use core::mem::MaybeUninit;

use ahrs::{Ahrs, Mahony};
use ascii_osd_hud::telemetry as hud;
use integer_sqrt::IntegerSquareRoot;
use nalgebra::Vector3;

use crate::config::Accelerometer;
use crate::datastructures::measurement::{quaternion_to_euler, DEGREE_PER_DAG};
use crate::hal::sensors::{Acceleration, Axes, Gyro, Pressure};

pub struct IMU(Mahony<f32>);

impl Default for IMU {
    fn default() -> Self {
        Self(Mahony::new(0.001, 0.5, 0.0))
    }
}

#[derive(Default, Copy, Clone, Value)]
pub struct Attitude {
    roll: i16,
    pitch: i8,
}

impl Into<hud::Attitude> for Attitude {
    fn into(self) -> hud::Attitude {
        hud::Attitude { pitch: self.pitch, roll: self.roll }
    }
}

#[derive(Default, Value)]
pub struct TelemetryData {
    acceleration: Acceleration,
    gyro: Gyro,
    attitude: Attitude,
    altitude: i16,
    heading: u16,
    vertical_speed: i16,
}

#[derive(Default)]
pub struct TelemetryUnit {
    imu: IMU,
    calibration_loop: u16,
    accelerometer_bias: Axes,
    gyro_bias: Axes,

    counter: usize,
    calibrated: bool,
    prev_altitude: i16,
    initial_altitude: i16,

    data: TelemetryData,
}

impl TelemetryUnit {
    fn on_accel_gyro_event(&mut self, event: (Acceleration, Gyro)) {
        let (acceleration, gyro) = event;
        self.counter += 1;
        if !self.calibrated {
            self.data.gyro = gyro;
            self.gyro_bias = self.gyro_bias.average(&gyro.axes);
            self.calibrated = self.counter >= self.calibration_loop as usize;
        }

        self.data.acceleration = acceleration;
        self.data.gyro = gyro;
        self.data.acceleration.axes.calibrated(&self.accelerometer_bias);
        self.data.gyro.axes.calibrated(&self.gyro_bias);

        let mut gyro: Vector3<f32> = self.data.gyro.into();
        gyro = gyro / DEGREE_PER_DAG;
        match self.imu.0.update_imu(&gyro, &(acceleration.into())) {
            Ok(quat) => {
                let euler = quaternion_to_euler(quat);
                let roll = -euler.theta as i16;
                let mut pitch = -euler.phi as i8;
                if pitch > 90 {
                    pitch = 90
                } else if pitch < -90 {
                    pitch = -90
                };
                self.data.attitude = Attitude { roll, pitch };
                self.data.heading = ((-euler.psi as isize + 360) % 360) as u16;
            }
            Err(_) => (),
        }
    }

    fn on_barometer_event(&mut self, event: Pressure) {
        self.prev_altitude = self.data.altitude;
        if self.initial_altitude == 0 {
            self.initial_altitude = self.data.altitude;
        }
        let feet = event.to_sea_level_altitude().as_feet();
        self.data.altitude = (feet as i16 + 5) / 10 * 10;
        let vertical_speed = (self.data.altitude - self.prev_altitude) * 20 * 60;
        let delta = (vertical_speed - self.data.vertical_speed) / 10;
        self.data.vertical_speed = (self.data.vertical_speed + delta) / 100 * 100;
    }

    pub fn g_force(&self) -> u8 {
        let acceleration = self.data.acceleration;
        let axes = acceleration.axes;
        let (x, y, z) = (axes.x as i32, axes.y as i32, axes.z as i32);
        let g_force = (x * x + y * y + z * z).integer_sqrt();
        let g_force = (g_force / acceleration.sensitive * 100) as u16;
        ((g_force + 5) / 10) as u8
    }
}

impl core::fmt::Display for TelemetryUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval_json::to_fmt(f, &self.data).ok();
        Ok(())
    }
}

impl hud::TelemetrySource for TelemetryUnit {
    fn get_telemetry(&self) -> hud::Telemetry {
        hud::Telemetry {
            altitude: self.data.altitude,
            attitude: self.data.attitude.into(),
            heading: self.data.heading,
            g_force: self.g_force(),
            height: self.data.altitude - self.initial_altitude,
            vertical_speed: self.data.vertical_speed,
            ..Default::default()
        }
    }
}

static mut G_TELEMETRY_UNIT: MaybeUninit<TelemetryUnit> = MaybeUninit::uninit();

pub fn accel_gyro_handler(event: (Acceleration, Gyro)) {
    unsafe { &mut *G_TELEMETRY_UNIT.as_mut_ptr() }.on_accel_gyro_event(event);
}

pub fn barometer_handler(event: Pressure) {
    unsafe { &mut *G_TELEMETRY_UNIT.as_mut_ptr() }.on_barometer_event(event);
}

pub fn init(
    gyro_accel_sample_rate: u16,
    calibration_loop: u16,
    config: &Accelerometer,
) -> &'static TelemetryUnit {
    let imu = IMU(Mahony::new(1.0 / gyro_accel_sample_rate as f32, 0.5, 0.0));
    unsafe {
        G_TELEMETRY_UNIT = MaybeUninit::new(TelemetryUnit {
            imu,
            calibration_loop,
            accelerometer_bias: config.bias.into(),
            ..Default::default()
        });
        &*G_TELEMETRY_UNIT.as_ptr()
    }
}
