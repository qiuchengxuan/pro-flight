use core::fmt::Write;

use bmp280::bus::Bus;
use bmp280::measurement::Calibration;
use bmp280::registers::{PressureOversampling, TemperatureOversampling};
use bmp280::{Mode, BMP280};
use embedded_hal::blocking::delay::DelayMs;

use crate::components::logger::Logger;

pub fn init<E, BUS, D>(bus: BUS, delay: &mut D) -> Result<Option<Calibration>, E>
where
    BUS: Bus<Error = E>,
    D: DelayMs<u8>,
{
    let mut bmp280 = BMP280::new(bus);
    bmp280.reset(delay)?;
    if !bmp280.verify()? {
        return Ok(None);
    }
    log!("BMP280 detected");
    bmp280.set_pressure_oversampling(PressureOversampling::StandardResolution)?;
    bmp280.set_temperature_oversampling(TemperatureOversampling::UltraLowPower)?;
    bmp280.set_mode(Mode::Normal)?;
    bmp280.set_standby_time(50)?; // at 20hz
    bmp280.set_iir_filter(8)?;
    Ok(Some(bmp280.read_calibration()?))
}
