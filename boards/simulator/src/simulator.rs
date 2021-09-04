use pro_flight::components::flight_data_hub::FlightDataHUB;
use pro_flight::components::mixer::ControlMixer;
use pro_flight::components::pipeline;
use pro_flight::components::variometer::Variometer;
use pro_flight::datastructures::control::Control;
use pro_flight::datastructures::coordinate::Position;
use pro_flight::datastructures::flight::FlightData;
use pro_flight::datastructures::measurement::{
    distance::Distance, unit, Acceleration, Course, Gyro, Heading, VelocityVector,
};
use pro_flight::sync::DataWriter;

pub struct Config {
    pub sample_rate: usize,
    pub altimeter_rate: usize,
    pub gnss_rate: usize,
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub struct GNSS {
    heading: Heading,
    course: Course,
    position: Position,
    velocity: VelocityVector<i32, unit::MMpS>,
}

pub struct Simulator {
    hub: &'static FlightDataHUB,
    imu: pipeline::imu::IMU<'static>,
    variometer: Variometer,
    mixer: ControlMixer<'static>,
    acceleration: bool,
    gyro: bool,
}

impl Simulator {
    pub fn new(config: Config) -> Self {
        let hub = Box::leak(Box::new(FlightDataHUB::default()));
        let imu = pipeline::imu::IMU::new(config.sample_rate, hub);
        let reader = hub.reader();
        let variometer = Variometer::new(1000 / config.altimeter_rate);
        let mut mixer = ControlMixer::new(reader.input, 50);
        hub.output.write(mixer.mix());
        Self { hub, imu, variometer, mixer, acceleration: false, gyro: false }
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

    pub fn update_altitude(&mut self, altitude: Distance<i32, unit::CentiMeter>) {
        self.hub.altimeter.write(altitude);
        self.hub.vertical_speed.write(self.variometer.update(altitude.into()));
    }

    pub fn update_gnss(&mut self, gnss: GNSS) {
        self.hub.gnss_heading.write(gnss.heading);
        self.hub.gnss_course.write(gnss.course);
        self.hub.gnss_velocity.write(gnss.velocity);
        self.hub.gnss_position.write(gnss.position);
    }
}
