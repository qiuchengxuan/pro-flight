use core::mem::MaybeUninit;

use ahrs::{Ahrs, Madgwick};
use ascii_osd_hud::telemetry as hud;
use integer_sqrt::IntegerSquareRoot;
use nalgebra::Vector3;

use crate::datastructures::measurement::{quaternion_to_euler, DEGREE_PER_DAG};
use crate::hal::sensors::{Acceleration, Gyro, Pressure};

pub struct IMU(Madgwick<f32>);

impl Default for IMU {
    fn default() -> Self {
        Self(Madgwick::new(0.001, 0.1))
    }
}

#[derive(Default, Value)]
pub struct TelemetryData {
    measurements: (Acceleration, Gyro),
    altitude: i16,
    vertical_speed: i16,
}

#[derive(Default)]
pub struct TelemetryUnit {
    imu: IMU,
    calibration_loop: u16,

    counter: usize,
    calibration: (Acceleration, Gyro),
    calibrated: bool,
    prev_altitude: i16,
    initial_altitude: i16,

    data: TelemetryData,
}

impl TelemetryUnit {
    fn on_accel_gyro_event(&mut self, event: (Acceleration, Gyro)) {
        let (mut acceleration, gyro) = event;
        self.counter += 1;
        if !self.calibrated {
            self.data.measurements = event;
            acceleration.z -= acceleration.sensitive as i16;
            self.calibration =
                (self.calibration.0.average(&acceleration), self.calibration.1.average(&gyro));
            self.calibrated = self.counter >= self.calibration_loop as usize;
        }
        self.data.measurements =
            (acceleration.calibrated(&self.calibration.0), gyro.calibrated(&self.calibration.1));
        let mut gyro: Vector3<f32> = self.data.measurements.1.into();
        gyro = gyro / DEGREE_PER_DAG;
        self.imu.0.update_imu(&gyro, &(acceleration.into())).ok();
    }

    fn on_barometer_event(&mut self, event: Pressure) {
        self.prev_altitude = self.data.altitude;
        if self.initial_altitude == 0 {
            self.initial_altitude = self.data.altitude;
        }
        self.data.altitude = event.to_sea_level_altitude().as_feet() as i16;
        self.data.vertical_speed =
            (self.data.vertical_speed + (self.data.altitude - self.prev_altitude) * 20 * 60) / 2;
    }

    pub fn g_force(&self) -> u8 {
        let acceleration = self.data.measurements.0;
        let (x, y, z) = (acceleration.x as i32, acceleration.y as i32, acceleration.z as i32);
        let g_force = ((x * x + y * y + z * z) as u32).integer_sqrt();
        let g_force = (g_force as f32 / acceleration.sensitive * 100.0) as u16;
        (g_force / 10 + ((g_force % 10) / 5)) as u8
    }
}

impl core::fmt::Display for TelemetryUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        sval::fmt::debug(f, &self.data)
    }
}

impl hud::TelemetrySource for TelemetryUnit {
    fn get_telemetry(&self) -> hud::Telemetry {
        let euler = quaternion_to_euler(self.imu.0.quat);
        let roll = -euler.theta as i16;
        let mut pitch = -euler.phi as i8;
        if pitch > 90 {
            pitch = 90
        } else if pitch < -90 {
            pitch = -90
        };
        let attitude = hud::Attitude { roll, pitch };
        let heading = ((-euler.psi as isize + 360) % 360) as u16;
        hud::Telemetry {
            altitude: self.data.altitude,
            attitude,
            heading,
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

pub fn init(gyro_accel_sample_rate: usize, calibration_loop: u16) -> &'static TelemetryUnit {
    let imu = IMU(Madgwick::new(1.0 / gyro_accel_sample_rate as f32, 0.1));
    unsafe {
        G_TELEMETRY_UNIT =
            MaybeUninit::new(TelemetryUnit { imu, calibration_loop, ..Default::default() });
        &*G_TELEMETRY_UNIT.as_ptr()
    }
}
