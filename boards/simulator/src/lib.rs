#[macro_use]
extern crate log;
extern crate pro_flight;

use pro_flight::components::flight_data_hub::FlightDataHUB;
use pro_flight::components::mixer::ControlMixer;
use pro_flight::components::pipeline;
use pro_flight::datastructures::control::Control;
use pro_flight::datastructures::flight::FlightData;
use pro_flight::datastructures::measurement::{Acceleration, Gyro};
use pro_flight::sync::DataWriter;

#[no_mangle]
fn get_jiffies() -> u64 {
    std::time::Instant::now().elapsed().as_nanos() as u64
}

pub struct Simulator {
    hub: &'static FlightDataHUB,
    imu: pipeline::imu::IMU<'static>,
    mixer: ControlMixer<'static>,
    acceleration: bool,
    gyro: bool,
}

impl Simulator {
    pub fn new(sample_rate: usize) -> Self {
        let hub = Box::leak(Box::new(FlightDataHUB::default()));
        let imu = pipeline::imu::IMU::new(sample_rate, hub);
        let reader = hub.reader();
        let mut mixer = ControlMixer::new(reader.input, 50);
        hub.output.write(mixer.mix());
        Self { hub, imu, mixer, acceleration: false, gyro: false }
    }

    pub fn get_telemetry(&self) -> FlightData {
        self.hub.reader().read()
    }

    pub fn update_input(&mut self, input: Control) {
        self.hub.input.write(input);
        self.hub.output.write(self.mixer.mix());
    }

    pub fn update_acceleration(&mut self, acceleration: Acceleration) {
        self.hub.accelerometer.write(acceleration);
        if self.gyro {
            trace!("Invoke IMU update");
            self.imu.invoke();
            self.hub.output.write(self.mixer.mix());
            self.gyro = false;
        } else {
            self.acceleration = true;
        }
    }

    pub fn update_gyro(&mut self, gyro: Gyro) {
        self.hub.gyroscope.write(gyro);
        if self.acceleration {
            trace!("Invoke IMU update");
            self.imu.invoke();
            self.hub.output.write(self.mixer.mix());
            self.acceleration = false;
        } else {
            self.gyro = true;
        }
    }
}
