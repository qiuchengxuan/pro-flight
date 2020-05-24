pub type Tuple3<T> = (T, T, T);

#[derive(Copy, Clone, Default)]
pub struct Acceleration<T>(pub Tuple3<T>);

pub trait Accelerometer<T> {
    fn get_acceleration(&self) -> Acceleration<T>;
}

#[derive(Copy, Clone, Default)]
pub struct Gyro<T>(pub Tuple3<T>);

#[derive(Copy, Clone, Default)]
pub struct Temperature<T>(pub T);

pub trait Thermometer<T> {
    fn get_temperature(&self) -> Temperature<T>;
}
