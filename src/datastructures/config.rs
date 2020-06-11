#[derive(Value)]
pub struct XYZ {
    x: i16,
    y: i16,
    z: i16,
}

#[derive(Value)]
pub struct Calibration {
    pub acceleration: XYZ,
}

#[derive(Value)]
pub struct Config {
    pub calibration: Calibration,
}
