use std::sync::{Arc, Mutex};

use axum::{Router, Server, routing::get, extract::State, Json, response::{IntoResponse, Html}};
use sysinfo::{CpuExt, System, SystemExt};

#[tokio::main]
async fn main() {
    let router = Router::new()
    .route("/", get(root_get))
    .route("/api/cpus", get(cpu_get))
    .with_state(AppState {sys: Arc::new(Mutex::new(System::new())) 
    });

    let server = Server::bind(&"0.0.0.0:7032".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on {addr}");

    server.await.unwrap();
    println!("Hello, world!");
}

#[derive(Clone)]
struct AppState{
    sys: Arc<Mutex<System>>,
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    Html(include_str!("index.html"))
}

#[axum::debug_handler]
async fn cpu_get(State(state): State<AppState>) -> impl IntoResponse {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu();

    let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();

    Json(v)
}