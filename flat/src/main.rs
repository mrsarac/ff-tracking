use fframes::{EncoderOptions, RenderOptions, StaticMediaProvider};
#[path = "../../tools/backend.rs"]
mod backend;
use flat::{FlatMedia, FlatVideo};
use std::process::ExitCode;

fn main() -> ExitCode {
    let media = FlatMedia::prepare().expect("media");
    let video = FlatVideo::new();

    backend::run(
        &video,
        RenderOptions {
            media: Some(&media),
            default_font: "JetBrains Mono",
            // Near lossless: the lens pass films this file and magnifies it up to 3x.
            video_encoder_options: EncoderOptions {
                preferred_encoder: Some("libx264"),
                codec_params: Some(&[("crf", "8"), ("preset", "fast"), ("tune", "animation")]),
                ..Default::default()
            },
            ..Default::default()
        },
        "out/pass/flat.mp4",
        false,
    )
}
