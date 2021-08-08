#[macro_use]
extern crate log;

use env_logger::Env;
use std::io::Read;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Mutex;
use pro_flight::config;
use pro_flight::config::yaml::YamlParser;
use pro_flight::datastructures::control::Control;
use pro_flight::datastructures::measurement::{Acceleration, Gyro};
use simulator::Simulator;

static SIMULATOR: Mutex<Option<Simulator>> = Mutex::new(None);

#[get("/telemetry")]
async fn get_telemetry() -> impl Responder {
    web::Json(SIMULATOR.lock().await.as_ref().unwrap().get_telemetry())
}

#[post("/input")]
async fn update_input(input: web::Json<Control>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_input(*input);
    HttpResponse::Ok()
}

#[post("/sensors/accelerometer")]
async fn update_acceleration(acceleration: web::Json<Acceleration>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_acceleration(*acceleration);
    HttpResponse::Ok()
}

#[post("/sensors/gyroscope")]
async fn update_gyro(gyro: web::Json<Gyro>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_gyro(*gyro);
    HttpResponse::Ok()
}

async fn init<'a>(matches: &'a clap::ArgMatches<'a>) -> Result<(), String> {
    let config_path = matches.value_of("config").unwrap_or("simulator.yaml");
    let mut file = std::fs::File::open(config_path)
        .map_err(|e| format!("Read config file {} failed: {}", config_path, e))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer).map_err(|_| "Unable to read config-file")?;
    let config = YamlParser::new(buffer.as_str()).parse();
    config::replace(&config);
    let rate_str = matches.value_of("rate").unwrap_or("1000");
    let sample_rate = rate_str.parse::<usize>().map_err(|_| format!("Rate not a number"))?;
    let simulator = Simulator::new(sample_rate);
    *SIMULATOR.lock().await = Some(simulator);
    Ok(())
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
    if let Some(error) = init(&matches).await.err() {
        println!("{}", error);
        return Ok(());
    }
    let listen = matches.value_of("listen").unwrap_or("127.0.0.1:8080");
    info!("Start listening on {}", listen);
    let server = || {
        App::new()
            .service(get_telemetry)
            .service(update_input)
            .service(update_acceleration)
            .service(update_gyro)
    };
    if listen.starts_with("/") {
        HttpServer::new(server).bind_uds(listen)?.run().await
    } else {
        HttpServer::new(server).bind(listen)?.run().await
    }
}
