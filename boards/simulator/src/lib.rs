#[macro_use]
extern crate log;
extern crate pro_flight;
#[macro_use]
extern crate serde;

pub mod simulator;

pub use simulator::{Config, Simulator, GNSS};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Mutex;
use pro_flight::datastructures::control::Control;
use pro_flight::datastructures::measurement::{distance::Distance, unit, Acceleration, Gyro};

#[no_mangle]
fn get_jiffies() -> u64 {
    std::time::Instant::now().elapsed().as_nanos() as u64
}

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

#[post("/sensors/altimeter")]
async fn update_altitude(altitude: web::Json<Distance<i32, unit::CentiMeter>>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_altitude(*altitude);
    HttpResponse::Ok()
}

#[post("/sensors/gnss")]
async fn update_gnss(gnss: web::Json<GNSS>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_gnss(*gnss);
    HttpResponse::Ok()
}

pub async fn start(config: Config, listen: &str) -> std::io::Result<()> {
    *SIMULATOR.lock().await = Some(Simulator::new(config));
    let server = HttpServer::new(|| {
        App::new()
            .service(get_telemetry)
            .service(update_input)
            .service(update_acceleration)
            .service(update_gyro)
            .service(update_altitude)
            .service(update_gnss)
    });
    let server =
        if listen.starts_with("/") { server.bind_uds(listen)? } else { server.bind(listen)? };
    info!("Start listening on {}", listen);
    server.run().await
}
