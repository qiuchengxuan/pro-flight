use alloc::rc::Rc;
use core::mem::MaybeUninit;

use bmp280::bus::Bus;
use bmp280::measurement::{Calibration, RawPressure, RawTemperature};
use bmp280::registers::{PressureOversampling, StandbyTime, TemperatureOversampling};
use bmp280::{Mode, BMP280};

use crate::datastructures::data_source::overwriting::{OverwritingData, OverwritingDataSource};
use crate::datastructures::data_source::DataWriter;
use crate::datastructures::measurement::Pressure;
use crate::sys::timer::SysTimer;

pub const BMP280_SAMPLE_RATE: usize = 16;

static mut CALIBRATION: MaybeUninit<Calibration> = MaybeUninit::uninit();
static mut BUFFER: Option<OverwritingData<Pressure>> = None;

pub unsafe fn on_dma_receive(dma_buffer: &[u8; 8]) {
    let calibration = &*CALIBRATION.as_ptr();
    let raw_pressure = RawPressure::from_bytes(&dma_buffer[2..]);
    let t_fine = RawTemperature::from_bytes(&dma_buffer[5..]).t_fine(calibration);
    let pressure = raw_pressure.compensated(t_fine, calibration);
    if let Some(ref mut buffer) = BUFFER {
        buffer.write(Pressure(pressure));
    }
}

pub fn init_data_source() -> OverwritingDataSource<Pressure> {
    unsafe { BUFFER = Some(OverwritingData::sized(8)) };
    let buffer = unsafe { &Rc::from_raw(BUFFER.as_ref().unwrap()) };
    core::mem::forget(buffer);
    OverwritingDataSource::new(buffer)
}

pub fn init<E, BUS: Bus<Error = E>>(bus: BUS) -> Result<bool, E> {
    let mut bmp280 = BMP280::new(bus);
    let mut delay = SysTimer::new();
    bmp280.reset(&mut delay)?;
    if !bmp280.verify()? {
        return Ok(false);
    }
    info!("BMP280 detected");
    bmp280.set_pressure_oversampling(PressureOversampling::StandardResolution)?;
    bmp280.set_temperature_oversampling(TemperatureOversampling::UltraLowPower)?;
    bmp280.set_mode(Mode::Normal)?;
    bmp280.set_standby_time(StandbyTime::Hertz16)?;
    bmp280.set_iir_filter(8)?;
    let calibration = bmp280.read_calibration()?;
    unsafe { CALIBRATION = MaybeUninit::new(calibration) };
    Ok(true)
}
