pub type Tuple3<T> = (T, T, T);

#[derive(Copy, Clone)]
pub struct Acceleration<T>(pub Tuple3<T>);

#[derive(Copy, Clone)]
pub struct Gyro<T>(pub Tuple3<T>);
