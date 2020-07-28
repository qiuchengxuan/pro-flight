use crc::Hasher32;
use embedded_hal::blocking::spi::{Transfer, Write};
use max7456::character_memory::{CharData, CHAR_DATA_SIZE};
use max7456::registers::Standard;
use max7456::MAX7456;

use crate::config;
use crate::hal::io::Read;
use crate::sys::fs::File;
use crate::sys::timer::SysTimer;

const HEADER_SIZE: usize = 8;

impl From<config::Standard> for Standard {
    fn from(standard: config::Standard) -> Standard {
        match standard {
            config::Standard::PAL => Standard::PAL,
            config::Standard::NTSC => Standard::NTSC,
        }
    }
}

fn check_font<E, B, CRC>(file: &mut File, osd: &mut MAX7456<B>, crc: &mut CRC) -> Result<bool, E>
where
    B: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CRC: Hasher32,
{
    let length = match file.metadata() {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            warn!("{:?}", e);
            return Ok(false);
        }
    };

    if length as usize != HEADER_SIZE + 256 * CHAR_DATA_SIZE {
        return Ok(false);
    }

    let mut bytes: [u8; 4] = [0u8; 4];
    file.read(&mut bytes).ok();
    if &bytes != b"7456" {
        return Ok(false);
    }
    file.read(&mut bytes).ok();

    osd.enable_display(false)?;
    crc.reset();
    let mut char_data: CharData = [0u8; CHAR_DATA_SIZE];
    for i in 0..256 {
        osd.load_char(i as u8, &mut char_data)?;
        crc.write(&char_data);
    }
    if crc.sum32() == u32::from_be_bytes(bytes) {
        return Ok(true);
    }

    info!("Uploading OSD font");
    let mut delay = SysTimer::new();
    for i in 0..256 {
        if file.read(&mut char_data).ok().unwrap_or(0) != CHAR_DATA_SIZE {
            return Ok(false);
        }
        crc.write(&char_data);
        osd.store_char(i as u8, &char_data, &mut delay)?;
    }
    info!("Upload complete");
    Ok(true)
}

pub fn init<BUS, E, CRC: Hasher32>(bus: BUS, crc: &mut CRC) -> Result<(), E>
where
    BUS: Write<u8, Error = E> + Transfer<u8, Error = E>,
{
    let config = &config::get().osd;
    let mut max7456 = MAX7456::new(bus);
    let mut delay = SysTimer::new();
    max7456.reset(&mut delay)?;
    max7456.set_standard(config.standard.into())?;
    if config.offset.horizental != 0 {
        max7456.set_horizental_offset(config.offset.horizental)?;
    }
    if config.offset.vertical != 0 {
        max7456.set_vertical_offset(config.offset.vertical)?;
    }
    if config.font != "" {
        match File::open(config.font) {
            Ok(mut file) => {
                check_font(&mut file, &mut max7456, crc)?;
                file.close();
            }
            Err(e) => warn!("Open file {} failed: {:?}", config.font, e),
        };
    }
    max7456.enable_display(true)?;
    Ok(())
}
