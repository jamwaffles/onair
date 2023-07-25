use axum::{routing::get, Router};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route(
            "/webcam",
            get(|| async {
                let cam_status = cam_status().await;

                let out = serde_json::json!({
                    "webcam": { "is_on": cam_status },

                });

                println!("Poll webcam {:?}", out);

                out.to_string()
            }),
        )
        .route(
            "/mic",
            get(|| async {
                let mic_status = mic_status().await;

                let out = serde_json::json!({

                    "mic": { "is_on": mic_status },
                });

                println!("Poll mic {:?}", out);

                out.to_string()
            }),
        );

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:6969".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn cam_status() -> bool {
    let out = tokio::process::Command::new("fuser")
        .arg("/dev/video0")
        .output()
        .await
        .unwrap()
        .stdout;

    let output = String::from_utf8_lossy(&out);

    let lines = output.lines().count();

    lines > 0
}

async fn mic_status() -> bool {
    // /dev/snd method
    // ---

    // let devices = std::fs::read_dir("/dev/snd").unwrap();

    // let it = devices
    //     .filter_map(|entry| entry.ok())
    //     .filter(|device| device.file_name().into_string().unwrap().starts_with("pcm"));

    // for device in it {
    //     let out = tokio::process::Command::new("fuser")
    //         .arg(device.path())
    //         .output()
    //         .await
    //         .unwrap()
    //         .stdout;

    //     let output = String::from_utf8_lossy(&out);

    //     let lines = output.lines().count();

    //     if lines > 0 {
    //         return true;
    //     }
    // }

    // false

    // Pulseaudio method
    // ---

    let mic_search = "Razer";

    let out = tokio::process::Command::new("pacmd")
        .arg("list-sources")
        .output()
        .await
        .unwrap()
        .stdout;

    let output = String::from_utf8_lossy(&out);

    let lines = output.lines();

    // Find output lines belonging to our search device
    let mut device_lines =
        lines.skip_while(|line| !(line.contains("name:") && line.contains(mic_search)));

    let status = device_lines
        .find(|line| line.contains("state:"))
        .expect("No status line");

    let state = status.split_whitespace().last().expect("No status part");

    state == "RUNNING"
}
