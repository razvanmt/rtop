use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::Response,
    response::{Html, IntoResponse},
    routing::get,
    Router, Server,
};
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::broadcast;

type Snapshot = Vec<f32>;

#[tokio::main]
async fn main() {
    let (tx, _) = broadcast::channel::<Snapshot>(1);

    let app_state = AppState{
        tx: tx.clone(),
    };

    let router = Router::new()
    .route("/", get(root_get))
    .route("/index.mjs", get(indexmjs_get))
    .route("/index.css", get(indexcss_get))
    .route("/api/cpus", get(websocket_cpus_get))
    .with_state(app_state.clone());

    tokio::task::spawn_blocking(move || {
        let mut sys = System::new();
        loop {
            sys.refresh_cpu();
            let v: Vec<_> = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).collect();
            let _ = tx.send(v);

            std::thread::sleep(System::MINIMUM_CPU_UPDATE_INTERVAL);
        }
    });



    let server = Server::bind(&"0.0.0.0:7032".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on {addr}");

    server.await.unwrap();
    println!("Hello, world!");
}

#[derive(Clone)]
struct AppState{
    tx: broadcast::Sender<Snapshot>,
}

#[axum::debug_handler]
async fn root_get() -> impl IntoResponse {
    // Not a good solution for prod, but we're not getting there. 
    let markup = tokio::fs::read_to_string("src/index.html").await.unwrap();
    Html(markup)
}

#[axum::debug_handler]
async fn indexmjs_get() -> impl IntoResponse {
    // Not a good solution for prod, but we're not getting there. 
    let markup = tokio::fs::read_to_string("src/index.mjs").await.unwrap();
    Response::builder()
    .header("content-type", "application/javascript;charset=utf-8")
    .body(markup)
    .unwrap()
}

#[axum::debug_handler]
async fn indexcss_get() -> impl IntoResponse {
    // Not a good solution for prod, but we're not getting there. 
    let markup = tokio::fs::read_to_string("src/index.css").await.unwrap();
    Response::builder()
    .header("content-type", "text/css;charset=utf-8")
    .body(markup)
    .unwrap()
}


#[axum::debug_handler]
async fn websocket_cpus_get(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(|ws: WebSocket| async {
        cpus_stream(state, ws).await
    })
}

async fn cpus_stream(app_state: AppState, mut ws: WebSocket) {
    let mut rx = app_state.tx.subscribe();

    while let Ok(msg) = rx.recv().await {
        let payload = serde_json::to_string(&msg).unwrap();
        ws.send(Message::Text(payload)).await.unwrap();
    }
}