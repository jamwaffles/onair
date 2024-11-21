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
        .arg("-v")
        .arg("/dev/video0")
        .output()
        .await
        .unwrap()
        .stderr;

    let output = String::from_utf8_lossy(&out);

    // Look for successful output headers. If not present, device might not be connected so we call
    // it `false.
    if !output.contains("USER PID ACCESS") {
        return false;
    }

    let lines = output
        .trim()
        .lines()
        // Ignore header and droidcam process
        .filter(|line| !line.contains("COMMAND") && !line.contains("droidcam"))
        .count();

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

    if let Some(state) = device_lines
        .find(|line| line.contains("state:"))
        .and_then(|status| status.split_whitespace().last())
    {
        return state == "RUNNING";
    }

    // Pipewire method
    // ---

    // We're looking for this output:
    //
    //     id 51, type PipeWire:Interface:Node/3
    //         object.serial = "51"
    //         object.path = "alsa:pcm:2:hw:2:capture"
    //         factory.id = "18"
    //         client.id = "35"
    //         device.id = "44"
    //         priority.session = "2000"
    //         priority.driver = "2000"
    //         node.description = "Razer Seiren Mini Mono"
    //         node.name = "alsa_input.usb-Razer_Inc_Razer_Seiren_Mini_UC2114L03205445-00.mono-fallback"
    //         node.nick = "Razer Seiren Mini"
    //         media.class = "Audio/Source"
    //
    // Note that the indentation uses tabs, not spaces. We want the `51` ID on the first line.

    let out = tokio::process::Command::new("pw-cli")
        .arg("ls")
        .output()
        .await
        .unwrap()
        .stdout;

    let output = String::from_utf8_lossy(&out);

    let lines = output.lines();

    let pipewire_node_id = {
        let mut curr_id = 0;
        let mut found_id = None;

        for line in lines {
            if line.contains("\tid ") {
                let mut chunks = line.split_ascii_whitespace();

                curr_id = chunks
                    .nth(1)
                    .expect("Not enough chunks")
                    .split(",")
                    .next()
                    .unwrap()
                    .parse::<i32>()
                    .expect("Bad number");
            }

            if line.contains("node.description") && line.contains("Razer Seiren Mini Mono") {
                found_id = Some(curr_id);

                break;
            }
        }

        found_id
    };

    let Some(pipewire_node_id) = pipewire_node_id else {
        println!("No mic ID");

        return false;
    };

    let out = tokio::process::Command::new("pw-cli")
        .arg("info")
        .arg(pipewire_node_id.to_string())
        .output()
        .await
        .unwrap()
        .stdout;

    let output = String::from_utf8_lossy(&out);

    output
        .lines()
        .find(|line| line.contains("state: \"running\""))
        .is_some()
}
