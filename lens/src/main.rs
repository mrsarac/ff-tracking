use fframes::{
    CombinedMediaProvider, EncoderOptions, MediaDirectory, MediaProvider, RenderOptions,
};
#[path = "../../tools/backend.rs"]
mod backend;
use lens::LensVideo;
use std::process::ExitCode;

fn main() -> ExitCode {
    // out/pass holds flat.mp4 (pass 1) and sfx.wav (tools/sfx.py); run from the workspace root.
    let dir = MediaDirectory::read_folder("out/pass")
        .expect("out/pass: render flat and run tools/sfx.py first");
    let pass = dir.process_media_source().expect("out/pass media");
    // Fonts for the camera tracker's labels.
    let fonts_dir = MediaDirectory::read_folder("media").expect("media folder");
    let fonts = fonts_dir.process_media_source().expect("fonts");
    let media = CombinedMediaProvider::from([&pass as &dyn MediaProvider, &fonts]);
    let video = LensVideo::new();

    backend::run(
        &video,
        RenderOptions {
            media: Some(&media),
            default_font: "JetBrains Mono",
            video_encoder_options: EncoderOptions {
                preferred_encoder: Some("libx264"),
                codec_params: Some(&[("crf", "16"), ("preset", "slow"), ("tune", "grain")]),
                ..Default::default()
            },
            ..Default::default()
        },
        "out/tracking.mp4",
        true,
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn lens_shader_compiles() {
        let shader = fframes::Shader::sksl(include_str!("../shaders/lens.sksl"));
        fframes_skia_renderer::render::compile_shader(&shader).expect("lens SkSL");
    }
}
