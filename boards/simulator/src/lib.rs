#[macro_use]
extern crate log;

pub mod simulator;

pub use simulator::{Config, Simulator};

use std::sync::atomic::{AtomicUsize, Ordering};

use actix_web::{get, post, put, web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Mutex;
use fugit::NanosDurationU64;
use pro_flight::{
    protocol::serial::gnss::out::GNSS,
    types::{
        control,
        measurement::{unit, Acceleration, Altitude, Distance, Gyro, ENU},
    },
};

static TICK: AtomicUsize = AtomicUsize::new(0);

#[no_mangle]
fn get_jiffies() -> NanosDurationU64 {
    NanosDurationU64::millis(TICK.load(Ordering::Relaxed) as u64)
}

static SIMULATOR: Mutex<Option<Simulator>> = Mutex::new(None);

#[post("/tick")]
async fn tick() -> impl Responder {
    TICK.fetch_add(1, Ordering::Relaxed);
    HttpResponse::Ok()
}

#[get("/telemetry")]
async fn get_telemetry() -> impl Responder {
    web::Json(SIMULATOR.lock().await.as_ref().unwrap().collect())
}

#[put("/input")]
async fn update_input(axes: web::Json<control::Axes>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_input(*axes);
    HttpResponse::Ok()
}

#[put("/sensors/accelerometer")]
async fn update_acceleration(acceleration: web::Json<Acceleration<ENU>>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_acceleration(*acceleration);
    HttpResponse::Ok()
}

#[put("/sensors/gyroscope")]
async fn update_gyro(gyro: web::Json<Gyro<unit::DEGs>>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_gyro(*gyro);
    HttpResponse::Ok()
}

#[put("/sensors/altimeter")]
async fn update_altitude(altitude: web::Json<Distance<i32, unit::CentiMeter>>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_altitude(Altitude(*altitude));
    HttpResponse::Ok()
}

#[put("/sensors/gnss")]
async fn update_gnss(gnss: web::Json<GNSS>) -> impl Responder {
    SIMULATOR.lock().await.as_mut().unwrap().update_gnss(*gnss);
    HttpResponse::Ok()
}

pub async fn start(config: Config, listen: &str) -> std::io::Result<()> {
    *SIMULATOR.lock().await = Some(Simulator::new(config));
    let server = HttpServer::new(|| {
        App::new()
            .service(tick)
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
