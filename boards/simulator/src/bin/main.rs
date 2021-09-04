extern crate log;

use env_logger::Env;
use std::io::Read;

use pro_flight::config;
use pro_flight::config::yaml::YamlParser;

fn init<'a>(matches: &'a clap::ArgMatches<'a>) -> Result<simulator::Config, String> {
    let config_path = matches.value_of("config").unwrap_or("simulator.yaml");
    let mut file = std::fs::File::open(config_path)
        .map_err(|e| format!("Read config file {} failed: {}", config_path, e))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).map_err(|_| "Unable to read config-file")?;
    let config = YamlParser::new(buffer.as_str()).parse();
    config::replace(&config);
    let rate_str = matches.value_of("rate").unwrap_or("1000");
    let sample_rate = rate_str.parse::<usize>().map_err(|_| format!("Rate not a number"))?;
    let rate_str = matches.value_of("altimeter-rate").unwrap_or("10");
    let altimeter_rate = rate_str.parse::<usize>().map_err(|_| format!("Rate not a number"))?;
    let rate_str = matches.value_of("gnss-rate").unwrap_or("10");
    let gnss_rate = rate_str.parse::<usize>().map_err(|_| format!("Rate not a number"))?;
    Ok(simulator::Config { sample_rate, altimeter_rate, gnss_rate })
}

macro_rules! arg {
    ($name:literal, $help:literal) => {
        clap::Arg::with_name($name).long($name).help($help).takes_value(true)
    };
    ($name:literal, $short:literal, $help:literal) => {
        clap::Arg::with_name($name).short($short).long($name).help($help).takes_value(true)
    };
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let matches = clap::App::new("simulator")
        .version("0.1")
        .author("qiuchengxuan")
        .about("Pro-flight flight controller simulator")
        .arg(arg!("listen", "l", "Listen address"))
        .arg(arg!("config", "Config file path"))
        .arg(arg!("rate", "IMU sample rate"))
        .arg(arg!("altimeter-rate", "Altimeter sample rate"))
        .arg(arg!("gnss-rate", "GNSS sample rate"))
        .get_matches();
    let config = match init(&matches) {
        Ok(config) => config,
        Err(error) => {
            println!("{}", error);
            return Ok(());
        }
    };
    let listen = matches.value_of("listen").unwrap_or("127.0.0.1:8080");
    simulator::start(config, listen).await
}
