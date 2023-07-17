use axum::{routing::get, Router};
use std::process::Command;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new().route(
        "/",
        get(|| async {
            let mic_status = mic_status().await;
            let cam_status = cam_status().await;

            serde_json::json!({
                "webcam": cam_status,
                "mic": mic_status,
            })
            .to_string()
        }),
    );

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:6969".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn cam_status() -> bool {
    let out = Command::new("fuser")
        .arg("/dev/video0")
        .output()
        .unwrap()
        .stdout;

    let output = String::from_utf8_lossy(&out);

    let lines = output.lines().count();

    lines > 0
}

async fn mic_status() -> bool {
    let devices = std::fs::read_dir("/dev/snd").unwrap();

    let in_use = devices
        .filter_map(|entry| entry.ok())
        .filter(|device| device.file_name().into_string().unwrap().starts_with("pcm"))
        .filter(|entry| {
            let out = Command::new("fuser")
                .arg(entry.path())
                .output()
                .unwrap()
                .stdout;

            let output = String::from_utf8_lossy(&out);

            let lines = output.lines().count();

            lines > 0
        })
        .count();

    in_use > 0
}
