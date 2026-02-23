#[cfg(unix)]
pub fn play_audio_file(file_path: &str) {
    let uri = if file_path.starts_with('/') {
        format!("file://{file_path}")
    } else {
        format!("file:///{file_path}")
    };

    let child = std::process::Command::new("gst-launch-1.0")
        .args(["playbin", &format!("uri={uri}")])
        .spawn();

    if let Ok(mut child) = child {
        std::thread::spawn(move || {
            let _ = child.wait();
        });
    }
}
