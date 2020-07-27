use crc::Hasher32;
use embedded_hal::blocking::spi::{Transfer, Write};
use max7456::character_memory::{CharData, CHAR_DATA_SIZE};
use max7456::font::{char_block_to_byte, validate_header, ByteBlock, HeaderBlock};
use max7456::registers::Standard;
use max7456::MAX7456;

use crate::config;
use crate::hal::io::Read;
use crate::sys::fs::File;
use crate::sys::timer::SysTimer;

impl From<config::Standard> for Standard {
    fn from(standard: config::Standard) -> Standard {
        match standard {
            config::Standard::PAL => Standard::PAL,
            config::Standard::NTSC => Standard::NTSC,
        }
    }
}

fn read_char<E: core::fmt::Debug>(reader: &mut dyn Read<Error = E>) -> Option<CharData> {
    let mut byte_block: ByteBlock = Default::default();
    let mut char_data: CharData = [0u8; CHAR_DATA_SIZE];
    for i in 0..char_data.len() {
        let size = match reader.read(&mut byte_block) {
            Ok(size) => size,
            Err(e) => {
                warn!("{:?}", e);
                return None;
            }
        };
        if size != byte_block.len() {
            return None;
        }
        if let Some(byte) = char_block_to_byte(&byte_block) {
            char_data[i] = byte
        } else {
            return None;
        }
    }
    Some(char_data)
}

fn upload<E, BUS, CRC>(file: &mut File, osd: &mut MAX7456<BUS>, crc: &mut CRC) -> Result<bool, E>
where
    BUS: Write<u8, Error = E> + Transfer<u8, Error = E>,
    CRC: Hasher32,
{
    osd.enable_display(false)?;
    let mut header_block: HeaderBlock = Default::default();
    let size = match file.read(&mut header_block) {
        Ok(size) => size,
        Err(e) => {
            warn!("{:?}", e);
            return Ok(false);
        }
    };
    if size != header_block.len() || !validate_header(&header_block) {
        warn!("Wrong file heading");
        return Ok(false);
    }

    crc.reset();
    let mut delay = SysTimer::new();
    for i in 0..256 {
        if let Some(char_data) = read_char(file) {
            crc.write(&char_data);
            osd.store_char(i as u8, &char_data, &mut delay)?;
        }
    }
    info!("Upload complete, checksum = {:#x}", crc.sum32());
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
                upload(&mut file, &mut max7456, crc)?;
                file.close();
            }
            Err(e) => warn!("Open file {} failed: {:?}", config.font, e),
        };
    }
    max7456.enable_display(true)?;
    Ok(())
}
