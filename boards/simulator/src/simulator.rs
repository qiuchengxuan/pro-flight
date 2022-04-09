use pro_flight::{
    config,
    config::fcs::Configuration,
    datastructures::{
        control::Control,
        coordinate::Position,
        flight::FlightData,
        measurement::{
            distance::Distance, unit, Acceleration, Course, Gyro, Heading, VelocityVector,
        },
        output::Output,
    },
    service::{
        aviation::{mixer::ControlMixer, pid::PIDs},
        flight::data::FlightDataHUB,
        imu,
        info::Writer,
        variometer::Variometer,
    },
};

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
    imu: imu::IMU<'static>,
    variometer: Variometer,
    configuration: Configuration,
    mixer: ControlMixer<'static>,
    pids: PIDs<'static>,
    acceleration: bool,
    gyro: bool,
}

impl Simulator {
    pub fn new(config: Config) -> Self {
        let hub = Box::leak(Box::new(FlightDataHUB::default()));
        let imu = imu::IMU::new(config.sample_rate, hub);
        let reader = hub.reader();
        let variometer = Variometer::new(1000 / config.altimeter_rate);
        let configuration = config::get().fcs.configuration;
        let mut mixer = ControlMixer::new(reader.input, 50);
        hub.output.write(Output::from(&mixer.mix(), configuration));
        let pids = PIDs::new(hub.reader().gyroscope, &config::get().fcs.pids);
        Self { hub, imu, variometer, configuration, mixer, pids, acceleration: false, gyro: false }
    }

    pub fn get_telemetry(&self) -> FlightData {
        self.hub.reader().read()
    }

    fn update_output(&mut self) {
        let control = self.pids.next_control(self.mixer.mix());
        let output = Output::from(&control, self.configuration);
        self.hub.output.write(output);
    }

    pub fn update_input(&mut self, input: Control) {
        self.hub.input.write(input);
        self.update_output();
    }

    pub fn update_acceleration(&mut self, acceleration: Acceleration) {
        self.hub.accelerometer.write(acceleration);
        if self.gyro {
            trace!("Invoke IMU update");
            self.imu.invoke();
            self.update_output();
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
            self.update_output();
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
