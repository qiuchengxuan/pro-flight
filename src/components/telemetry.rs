use core::mem::MaybeUninit;

use ahrs::{Ahrs, Madgwick};
use ascii_osd_hud::telemetry as hud;
use integer_sqrt::IntegerSquareRoot;
use nalgebra::Vector3;

use crate::datastructures::measurement::{quaternion_to_euler, DEGREE_PER_DAG};
use crate::hal::sensors::{Acceleration, Gyro};

pub struct TelemetryUnit {
    imu: Madgwick<f32>,
    calibration_loop: u16,

    counter: usize,
    measurements: (Acceleration, Gyro),
    calibration: (Acceleration, Gyro),
    calibrated: bool,
}

impl TelemetryUnit {
    fn on_accel_gyro_event(&mut self, event: (Acceleration, Gyro)) {
        let (mut acceleration, gyro) = event;
        self.counter += 1;
        if !self.calibrated {
            self.measurements = event;
            acceleration.z -= acceleration.sensitive as i16;
            self.calibration =
                (self.calibration.0.average(&acceleration), self.calibration.1.average(&gyro));
            self.calibrated = self.counter >= self.calibration_loop as usize;
        }
        self.measurements =
            (acceleration.calibrated(&self.calibration.0), gyro.calibrated(&self.calibration.1));
        let mut gyro: Vector3<f32> = self.measurements.1.into();
        gyro = gyro / DEGREE_PER_DAG;
        self.imu.update_imu(&gyro, &(acceleration.into())).ok();
    }

    pub fn g_force(&self) -> u8 {
        let acceleration = self.measurements.0;
        let (x, y, z) = (acceleration.x as i32, acceleration.y as i32, acceleration.z as i32);
        let g_force = ((x * x + y * y + z * z) as u32).integer_sqrt();
        let g_force = (g_force as f32 / acceleration.sensitive * 100.0) as u16;
        (g_force / 10 + ((g_force % 10) / 5)) as u8
    }
}

impl core::fmt::Display for TelemetryUnit {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(
            f,
            "{{\"acceleration\":{},\"gyro\":{},\"euler\":{},\"g-force\":{},\
               \"calibration\":{{\"acceleration\":{},\"gyro\":{}}}}}",
            self.measurements.0,
            self.measurements.1,
            quaternion_to_euler(self.imu.quat),
            self.g_force(),
            self.calibration.0,
            self.calibration.1,
        )
    }
}

impl hud::TelemetrySource for TelemetryUnit {
    fn get_telemetry(&self) -> hud::Telemetry {
        let euler = quaternion_to_euler(self.imu.quat);
        let roll = -euler.theta as i16;
        let mut pitch = -euler.phi as i8;
        if pitch > 90 {
            pitch = 90
        } else if pitch < -90 {
            pitch = -90
        };
        let attitude = hud::Attitude { roll, pitch };
        let heading = ((-euler.psi as isize + 360) % 360) as u16;
        hud::Telemetry { attitude, heading, g_force: self.g_force(), ..Default::default() }
    }
}

static mut G_TELEMETRY_UNIT: MaybeUninit<TelemetryUnit> = MaybeUninit::uninit();

pub fn accel_gyro_handler(event: (Acceleration, Gyro)) {
    unsafe { &mut *G_TELEMETRY_UNIT.as_mut_ptr() }.on_accel_gyro_event(event);
}

pub fn init(gyro_accel_sample_rate: usize, calibration_loop: u16) -> &'static TelemetryUnit {
    let imu = Madgwick::new(1.0 / gyro_accel_sample_rate as f32, 0.1);
    unsafe {
        G_TELEMETRY_UNIT = MaybeUninit::new(TelemetryUnit {
            imu,
            calibration_loop,
            counter: 0,
            measurements: Default::default(),
            calibrated: false,
            calibration: Default::default(),
        });
        &*G_TELEMETRY_UNIT.as_ptr()
    }
}
