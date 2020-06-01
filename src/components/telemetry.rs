use core::mem::MaybeUninit;

use ahrs::{Ahrs, Madgwick};
use ascii_osd_hud::telemetry::{Attitude, Telemetry, TelemetrySource};
use integer_sqrt::IntegerSquareRoot;
use nalgebra::Vector3;

use crate::hal::sensors::{Acceleration, Gyro};

pub struct TelemetryUnit {
    imu: Madgwick<f32>,
    refresh_counter: u16,

    counter: u16,
    g_force: u8,
}

impl TelemetryUnit {
    fn on_accel_gyro_event(&mut self, event: (Acceleration, Gyro)) {
        let (acceleration, gyro) = event;
        self.counter += 1;
        if self.counter == self.refresh_counter {
            let (x, y, z) = (acceleration.x as i32, acceleration.y as i32, acceleration.z as i32);
            let g_force = ((x * x + y * y + z * z) as u32).integer_sqrt();
            self.g_force = (g_force as f32 / acceleration.sensitive * 10.0) as u8;
            self.counter = 0;
        }
        let mut gyro: Vector3<f32> = gyro.into();
        gyro = gyro * (core::f32::consts::PI / 180.0);
        self.imu.update_imu(&gyro, &(acceleration.into())).ok();
    }
}

impl TelemetrySource for TelemetryUnit {
    fn get_telemetry(&self) -> Telemetry {
        let xyz = self.imu.quat.imag() / (core::f32::consts::PI / 180.0);
        let attitude = Attitude { pitch: xyz[0] as i8, roll: xyz[1] as i8, yaw: xyz[2] as u16 };
        Telemetry { attitude, g_force: self.g_force, ..Default::default() }
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

pub fn init(gyro_accel_rate: u16, refresh_rate: u16) -> &'static TelemetryUnit {
    let imu = Madgwick::new(1.0 / gyro_accel_rate as f32, 0.1);
    unsafe {
        G_TELEMETRY_UNIT = MaybeUninit::new(TelemetryUnit {
            imu,
            g_force: 0,
            refresh_counter: gyro_accel_rate / refresh_rate,
            counter: 0,
        });
        &*G_TELEMETRY_UNIT.as_ptr()
    }
}
