extern crate log;

use env_logger::Env;
use std::io::Read;

use pro_flight::config;
use pro_flight::config::yaml::YamlParser;

fn init<'a>(matches: &'a clap::ArgMatches<'a>) -> Result<usize, String> {
    let config_path = matches.value_of("config").unwrap_or("simulator.yaml");
    let mut file = std::fs::File::open(config_path)
        .map_err(|e| format!("Read config file {} failed: {}", config_path, e))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).map_err(|_| "Unable to read config-file")?;
    let config = YamlParser::new(buffer.as_str()).parse();
    config::replace(&config);
    let rate_str = matches.value_of("rate").unwrap_or("1000");
    let sample_rate = rate_str.parse::<usize>().map_err(|_| format!("Rate not a number"))?;
    Ok(sample_rate)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let matches = clap::App::new("simulator")
        .version("0.1")
        .author("qiuchengxuan")
        .about("Pro-flight flight controller simulator")
        .arg(clap::Arg::with_name("listen").short("l").help("Listen address").takes_value(true))
        .arg(clap::Arg::with_name("config").long("config").help("Config file").takes_value(true))
        .arg(clap::Arg::with_name("rate").long("rate").help("Sample rate").takes_value(true))
        .get_matches();
    let sample_rate = match init(&matches) {
        Ok(rate) => rate,
        Err(error) => {
            println!("{}", error);
            return Ok(());
        }
    };
    let listen = matches.value_of("listen").unwrap_or("127.0.0.1:8080");
    simulator::start(sample_rate, listen).await
}
