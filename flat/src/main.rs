use fframes::{EncoderOptions, RenderOptions, StaticMediaProvider, cli};
use fframes_skia_renderer::{
    SkiaFFramesRenderer, SkiaPipelineConcurrencyPolicy, SkiaPipelineConfig, metal::SkiaMetalCtx,
};
use flat::{FlatMedia, FlatVideo, HEIGHT, WIDTH};
use std::process::ExitCode;

fn main() -> ExitCode {
    let media = FlatMedia::prepare().expect("media");
    let video = FlatVideo::new();
    let gpu = SkiaMetalCtx::new(WIDTH, HEIGHT).expect("GPU context");

    cli::new(
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
    )
    .backend(
        SkiaFFramesRenderer::new_metal(
            &gpu,
            SkiaPipelineConfig {
                concurrency_policy: SkiaPipelineConcurrencyPolicy::MaxPerformance,
                ..Default::default()
            },
        )
        .expect("skia renderer"),
    )
    .preview(fframes_native_player::cli_preview)
    .default_output("out/pass/flat.mp4")
    .run()
}
