use core::mem::MaybeUninit;

use ahrs::{Ahrs, Madgwick};
use ascii_osd_hud::telemetry::{Attitude, Telemetry, TelemetrySource};
use nalgebra::Vector3;

use crate::hal::sensors::{Acceleration, Gyro};

pub struct TelemetryUnit {
    imu: Madgwick<f32>,
}

impl TelemetryUnit {
    fn on_accel_gyro_event(&mut self, event: (Acceleration, Gyro)) {
        let (acceleration, gyro) = event;
        let mut gyro: Vector3<f32> = gyro.into();
        gyro = gyro * (core::f32::consts::PI / 180.0);
        self.imu.update_imu(&gyro, &(acceleration.into())).ok();
    }
}

impl TelemetrySource for TelemetryUnit {
    fn get_telemetry(&self) -> Telemetry {
        let xyz = self.imu.quat.imag() / (core::f32::consts::PI / 180.0);
        let attitude = Attitude { pitch: xyz[0] as i8, roll: xyz[1] as i8, yaw: xyz[2] as u16 };
        Telemetry { attitude: attitude, ..Default::default() }
    }
}

static mut G_TELEMETRY_UNIT: MaybeUninit<TelemetryUnit> = MaybeUninit::uninit();
static mut G_ACCEL_GYRO: (Acceleration, Gyro) =
    (Acceleration { x: 0, y: 0, z: 0, sensitive: 0.0 }, Gyro { x: 0, y: 0, z: 0, sensitive: 0.0 });
static mut G_DATA_VALID: bool = false;

pub fn accel_gyro_handler(event: (Acceleration, Gyro)) {
    unsafe { G_ACCEL_GYRO = event };
    unsafe { G_DATA_VALID = true };
}

pub fn process_accel_gyro() {
    if !unsafe { G_DATA_VALID } {
        return;
    }
    unsafe { G_DATA_VALID = false };
    unsafe { &mut *G_TELEMETRY_UNIT.as_mut_ptr() }.on_accel_gyro_event(unsafe { G_ACCEL_GYRO });
}

pub fn init() -> &'static TelemetryUnit {
    let imu = Madgwick::new(0.001, 0.1);
    unsafe {
        G_TELEMETRY_UNIT = MaybeUninit::new(TelemetryUnit { imu });
        &*G_TELEMETRY_UNIT.as_ptr()
    }
}
