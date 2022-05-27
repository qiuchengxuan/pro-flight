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
    acceleration: Option<[f32; 3]>,
    gyro: Option<[f32; 3]>,
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
        match self.gyro.take() {
            Some(gyro) => {
                trace!("Invoke INS update");
                self.imu.update(acceleration.0.raw.into(), gyro.into());
                self.ins.update()
            }
            None => self.acceleration = Some(acceleration.0.raw.into()),
        }
    }

    pub fn update_gyro(&mut self, gyro: Gyro<unit::DEGs>) {
        match self.acceleration.take() {
            Some(acceleration) => {
                trace!("Invoke INS update");
                self.imu.update(acceleration.into(), gyro.0.raw.into());
                self.ins.update()
            }
            None => self.gyro = Some(gyro.0.raw.into()),
        }
    }

    pub fn update_altitude(&mut self, altitude: Altitude) {
        datastore::acquire().write_altitude(altitude);
    }

    pub fn update_gnss(&mut self, gnss: GNSS) {
        datastore::acquire().write_gnss(gnss.into());
    }
}
