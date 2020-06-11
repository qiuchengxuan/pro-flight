use core::mem::MaybeUninit;

use ahrs::{Ahrs, Mahony};
use ascii_osd_hud::telemetry as hud;
use integer_sqrt::IntegerSquareRoot;
use nalgebra::Vector3;

use crate::datastructures::measurement::{quaternion_to_euler, DEGREE_PER_DAG};
use crate::hal::sensors::{Acceleration, Gyro, Pressure};

pub struct IMU(Mahony<f32>);

impl Default for IMU {
    fn default() -> Self {
        Self(Mahony::new(0.001, 0.5, 0.0))
    }
}

#[derive(Default, Copy, Clone, Serialize, Deserialize)]
pub struct Attitude {
    roll: i16,
    pitch: i8,
}

impl Into<hud::Attitude> for Attitude {
    fn into(self) -> hud::Attitude {
        hud::Attitude { pitch: self.pitch, roll: self.roll }
    }
}

#[derive(Default, Serialize, Deserialize)]
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
    accel_calibration: Acceleration,
    gyro_calibration: Gyro,

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
            self.gyro_calibration = self.gyro_calibration.average(&gyro);
            self.calibrated = self.counter >= self.calibration_loop as usize;
        }
        self.data.acceleration = acceleration.calibrated(&self.accel_calibration);
        self.data.gyro = gyro.calibrated(&self.gyro_calibration);
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
        let vertical_speed = (self.data.altitude - self.prev_altitude) * 20 * 60 / 100 * 100;
        self.data.vertical_speed = (self.data.vertical_speed + vertical_speed) / 2;
    }

    pub fn g_force(&self) -> u8 {
        let acceleration = self.data.acceleration;
        let (x, y, z) = (acceleration.x as i32, acceleration.y as i32, acceleration.z as i32);
        let g_force = ((x * x + y * y + z * z) as u32).integer_sqrt();
        let g_force = (g_force as f32 / acceleration.sensitive * 100.0) as u16;
        ((g_force + 5) / 10) as u8
    }
}

impl core::fmt::Display for TelemetryUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match serde_json::to_string_pretty(&self.data) {
            Ok(string) => f.write_str(&string),
            Err(_) => f.write_str(""),
        }
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
    gyro_accel_sample_rate: usize,
    calibration_loop: u16,
    accel_calibration: Acceleration,
) -> &'static TelemetryUnit {
    let imu = IMU(Mahony::new(1.0 / gyro_accel_sample_rate as f32, 0.5, 0.0));
    unsafe {
        G_TELEMETRY_UNIT = MaybeUninit::new(TelemetryUnit {
            imu,
            calibration_loop,
            accel_calibration,
            ..Default::default()
        });
        &*G_TELEMETRY_UNIT.as_ptr()
    }
}
