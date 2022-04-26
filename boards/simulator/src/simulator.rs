use pro_flight::{
    collection::{Collection, Collector},
    datastore,
    fcs::FCS,
    imu::IMU,
    ins::{variometer::Variometer, INS},
    protocol::serial::gnss::out::GNSS,
    types::{
        control,
        measurement::{unit, Acceleration, Altitude, Gyro, ENU},
        sensor::{Axes, Readout},
    },
};

pub struct Config {
    pub sample_rate: usize,
    pub altimeter_rate: usize,
    pub gnss_rate: usize,
}

pub struct Simulator {
    imu: IMU,
    ins: INS,
    fcs: FCS,
    acceleration: Option<Readout>,
    gyro: Option<Readout>,
}

impl Simulator {
    pub fn new(config: Config) -> Self {
        let mut imu = IMU::new(config.sample_rate);
        imu.skip_calibration();
        let variometer = Variometer::new(1000 / config.altimeter_rate);
        let ins = INS::new(config.sample_rate, variometer);
        let mut fcs = FCS::new(1000);
        fcs.update();
        Self { imu, ins, fcs, acceleration: None, gyro: None }
    }

    pub fn collect(&self) -> Collection {
        Collector::new(datastore::acquire()).collect()
    }

    pub fn update_input(&mut self, axes: control::Axes) {
        let ds = datastore::acquire();
        ds.write_control(control::Control { rssi: 100, axes, commands: Default::default() });
        self.fcs.update();
    }

    pub fn update_acceleration(&mut self, acceleration: Acceleration<ENU>) {
        let acceleration = Readout {
            axes: Axes {
                x: (acceleration.0.raw[0] * 32768.0) as i32,
                y: (acceleration.0.raw[1] * 32768.0) as i32,
                z: (acceleration.0.raw[2] * 32768.0) as i32,
            },
            sensitive: 32768,
        };
        match self.gyro.take() {
            Some(gyro) => {
                trace!("Invoke INS update");
                self.imu.update(acceleration, gyro);
                self.ins.update()
            }
            None => self.acceleration = Some(acceleration),
        }
    }

    pub fn update_gyro(&mut self, gyro: Gyro<unit::DEGs>) {
        let gyro = Readout {
            axes: Axes {
                x: (gyro.0.raw[0] * 32768.0) as i32,
                y: (gyro.0.raw[1] * 32768.0) as i32,
                z: (gyro.0.raw[2] * 32768.0) as i32,
            },
            sensitive: 32768,
        };
        match self.acceleration.take() {
            Some(acceleration) => {
                trace!("Invoke INS update");
                self.imu.update(acceleration, gyro);
                self.ins.update()
            }
            None => self.gyro = Some(gyro),
        }
    }

    pub fn update_altitude(&mut self, altitude: Altitude) {
        datastore::acquire().write_altitude(altitude);
    }

    pub fn update_gnss(&mut self, gnss: GNSS) {
        datastore::acquire().write_gnss(gnss.into());
    }
}
