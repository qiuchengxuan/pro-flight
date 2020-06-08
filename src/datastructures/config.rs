#[derive(Value)]
pub struct Calibration {
    x: i32,
    y: i32,
    z: i32,
}

#[derive(Value)]
pub struct Config {
    calibration: Calibration,
}
