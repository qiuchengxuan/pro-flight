#[macro_use]
extern crate log;
extern crate pro_flight;

pub mod simulator;

pub use simulator::Simulator;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use async_std::sync::Mutex;
use pro_flight::datastructures::control::Control;
use pro_flight::datastructures::measurement::{Acceleration, Gyro};

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

pub async fn start(sample_rate: usize, listen: &str) -> std::io::Result<()> {
    *SIMULATOR.lock().await = Some(Simulator::new(sample_rate));
    let server = HttpServer::new(|| {
        App::new()
            .service(get_telemetry)
            .service(update_input)
            .service(update_acceleration)
            .service(update_gyro)
    });
    let server =
        if listen.starts_with("/") { server.bind_uds(listen)? } else { server.bind(listen)? };
    info!("Start listening on {}", listen);
    server.run().await
}
