use core::fmt::Write;

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use embedded_sdmmc::{Controller, Directory, Mode, SdMmcSpi, TimeSource, Volume};

use rs_flight::components::logger::Logger;
use rs_flight::datastructures::config::Config;
type SdcardTuple<'a, SPI, CS, T> =
    (&'a mut Controller<SdMmcSpi<SPI, CS>, T>, &'a mut Volume, &'a mut Directory);

pub fn read_json_file<SPI, CS, T>(tuple: SdcardTuple<SPI, CS, T>) -> Config
where
    SPI: FullDuplex<u8>,
    CS: OutputPin,
    T: TimeSource,
    <SPI as FullDuplex<u8>>::Error: core::fmt::Debug,
{
    let (controller, volume, root) = tuple;
    let mut config: Config = Default::default();
    let mode = Mode::ReadWriteCreateOrTruncate;
    match controller.open_file_in_dir(volume, root, "config.json", mode) {
        Ok(mut file) => {
            if file.length() == 0 {
                let string = serde_json::to_string_pretty(&config).ok().unwrap();
                controller.write(volume, &mut file, string.as_bytes()).ok();
                return config;
            }
            let mut buffer = [0u8; 1024];
            match controller.read(volume, &mut file, &mut buffer) {
                Ok(size) => {
                    config = serde_json::from_slice(&buffer[..size]).ok().unwrap_or_default()
                }
                Err(e) => {
                    log!("Read config.json failed for {:?}", e);
                }
            };
            controller.close_file(volume, file).ok();
        }
        Err(e) => {
            log!("Open config.json failed for {:?}", e);
        }
    };
    config
}
