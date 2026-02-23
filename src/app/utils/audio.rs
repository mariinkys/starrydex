use gst::prelude::*;
use gstreamer as gst;

#[cfg(unix)]
pub fn play_audio_file(file_path: &str) {
    let uri = if file_path.starts_with('/') {
        format!("file://{file_path}")
    } else {
        format!("file:///{file_path}")
    };

    let Ok(playbin) = gst::ElementFactory::make("playbin")
        .property("uri", &uri)
        .build()
    else {
        eprintln!("Failed to create playbin");
        return;
    };

    let _ = playbin.set_state(gst::State::Playing);

    std::thread::spawn(move || {
        let bus = playbin.bus().unwrap();
        for msg in bus.iter_timed(gst::ClockTime::NONE) {
            use gst::MessageView;
            match msg.view() {
                MessageView::Eos(..) | MessageView::Error(..) => break,
                _ => {}
            }
        }
        let _ = playbin.set_state(gst::State::Null);
        drop(playbin);
    });
}
