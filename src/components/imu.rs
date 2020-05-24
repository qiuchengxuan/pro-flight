use core::mem::MaybeUninit;

use dcmimu::DCMIMU;

use crate::hal::imu::{AccelGyroHandler, Attitude, IMU};
use crate::hal::sensors::{Acceleration, Gyro};

pub struct StubIMU(pub DCMIMU);

impl IMU for StubIMU {
    fn get_attitude(&self) -> Attitude {
        Attitude {
            pitch: self.0.pitch() as i8,
            roll: self.0.roll() as i8,
            yaw: self.0.yaw() as u16,
        }
    }
}

static mut G_IMU: MaybeUninit<StubIMU> = MaybeUninit::uninit();

static mut G_ACCELERATION: &[Acceleration<u32>] = &[];
static mut G_GYRO: &[Gyro<u32>] = &[];
pub fn accel_gyro_handler(event: (&'static [Acceleration<u32>], &'static [Gyro<u32>])) {
    let (acceleration, gyro) = event;
    unsafe {
        G_ACCELERATION = acceleration;
        G_GYRO = gyro
    }
}

pub fn get_handler() -> AccelGyroHandler {
    return accel_gyro_handler as AccelGyroHandler;
}

pub fn imu() -> &'static StubIMU {
    let stub_imu = StubIMU(DCMIMU::new());
    unsafe {
        G_IMU = MaybeUninit::new(stub_imu);
        &*G_IMU.as_ptr()
    }
}
