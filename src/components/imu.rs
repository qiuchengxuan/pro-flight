use core::fmt::Write;
use core::mem::MaybeUninit;

use ahrs::{Ahrs, Madgwick};
use nalgebra::Vector3;

use crate::components::logger::Logger;
use crate::hal::imu::{AccelGyroHandler, Attitude, IMU};
use crate::hal::sensors::{Acceleration, Gyro};

pub struct StubIMU(pub Madgwick<f32>);

impl IMU for StubIMU {
    fn get_attitude(&self) -> Attitude {
        let xyz = self.0.quat.imag() / (core::f32::consts::PI / 180.0);
        Attitude { pitch: xyz[0] as i8, roll: xyz[1] as i8, yaw: xyz[2] as u16 }
    }
}

static mut G_IMU: MaybeUninit<StubIMU> = MaybeUninit::uninit();
static mut G_ACCEL: Acceleration<f32> = (0.0, 0.0, 0.0);
static mut G_GYRO: Gyro<f32> = (0.0, 0.0, 0.0);
static mut G_COUNTER: usize = 0;

pub fn accel_gyro_handler(event: (Acceleration<f32>, Gyro<f32>)) {
    let (acceleration, gyro) = event;
    unsafe { G_ACCEL = acceleration };
    unsafe { G_GYRO = gyro };
    unsafe { G_COUNTER += 1 };
}

pub fn process_accel_gyro() {
    let counter = unsafe { G_COUNTER };
    if counter == 0 {
        return;
    } else if counter > 1000 {
        log!("Receive too slow");
        unsafe { G_COUNTER = 0 };
    } else {
        unsafe { G_COUNTER -= 1 };
    }
    let (a_x, a_y, a_z) = unsafe { G_ACCEL };
    let (g_x, g_y, g_z) = unsafe { G_GYRO };
    let accel = Vector3::new(a_x, a_y, a_z);
    let mut gyro = Vector3::new(g_x, g_y, g_z);
    gyro = gyro * (core::f32::consts::PI / 180.0);
    unsafe { &mut *G_IMU.as_mut_ptr() }.0.update_imu(&gyro, &accel).ok();
}

pub fn get_accel_gyro_handler() -> AccelGyroHandler {
    return accel_gyro_handler as AccelGyroHandler;
}

pub fn init() -> &'static StubIMU {
    let stub_imu = StubIMU(Madgwick::new(0.001, 0.1));
    unsafe {
        G_IMU = MaybeUninit::new(stub_imu);
        &*G_IMU.as_ptr()
    }
}
