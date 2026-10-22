//! Shared by the two passes. CPU rendering never creates a GPU context.
use fframes::{RenderOptions, Video, cli, cpu::CpuRenderingBackend};
#[cfg(target_os = "macos")]
use fframes_skia_renderer::metal::SkiaMetalCtx as SkiaGpuCtx;
#[cfg(not(target_os = "macos"))]
use fframes_skia_renderer::vulkan::SkiaVulkanCtx as SkiaGpuCtx;
use fframes_skia_renderer::{
    SkiaCpuCtx, SkiaFFramesRenderer, SkiaPipelineConcurrencyPolicy, SkiaPipelineConfig,
};
use std::process::ExitCode;

#[derive(Debug, PartialEq)]
enum Backend {
    Auto,
    Gpu,
    Cpu,
}

impl Backend {
    fn parse(value: &str) -> Result<Self, &'static str> {
        match value {
            "auto" => Ok(Self::Auto),
            "gpu" => Ok(Self::Gpu),
            "cpu" => Ok(Self::Cpu),
            _ => Err("FF_BACKEND must be auto, gpu or cpu"),
        }
    }
}

pub fn run<'a, 'media: 'a, V: Video + Send + Sync>(
    video: &'a V,
    options: RenderOptions<'a, 'media>,
    output: &str,
    sksl: bool,
) -> ExitCode {
    let backend = match Backend::parse(&std::env::var("FF_BACKEND").unwrap_or("auto".into())) {
        Ok(backend) => backend,
        Err(err) => {
            eprintln!("{err}");
            return ExitCode::FAILURE;
        }
    };
    let pipeline = if cfg!(target_os = "macos") {
        SkiaPipelineConfig {
            concurrency_policy: SkiaPipelineConcurrencyPolicy::MaxPerformance,
            ..Default::default()
        }
    } else {
        // Bound GPU contexts and frame buffers on shared/discrete GPUs.
        SkiaPipelineConfig {
            concurrency_policy: SkiaPipelineConcurrencyPolicy::Concurrency(2),
            encoder_threads: 2,
            buffer_queue_size: 2,
            ..Default::default()
        }
    };
    let runner = cli::new(video, options)
        .preview(fframes_native_player::cli_preview)
        .default_output(output);
    if backend != Backend::Cpu {
        match SkiaGpuCtx::new(V::WIDTH, V::HEIGHT) {
            Ok(gpu) => {
                eprintln!(
                    "FF_BACKEND=gpu ({})",
                    if cfg!(target_os = "macos") {
                        "Metal"
                    } else {
                        "Vulkan"
                    }
                );
                return runner
                    .backend(SkiaFFramesRenderer::new(pipeline, &gpu))
                    .run();
            }
            Err(err) if backend == Backend::Auto => {
                eprintln!("GPU unavailable: {err}; falling back to CPU")
            }
            Err(err) => {
                eprintln!("GPU unavailable: {err}");
                return ExitCode::FAILURE;
            }
        }
    }
    if sksl {
        // tiny-skia cannot execute SkSL. Skia's raster backend can, including the lens.
        eprintln!("FF_BACKEND=cpu (Skia raster; full SkSL lens, no effects disabled)");
        let cpu = SkiaCpuCtx::new(V::WIDTH, V::HEIGHT);
        runner
            .backend(SkiaFFramesRenderer::new(pipeline, &cpu))
            .run()
    } else {
        eprintln!("FF_BACKEND=cpu (tiny-skia SVG)");
        runner
            .backend(CpuRenderingBackend {
                concurrency: 2,
                ..Default::default()
            })
            .run()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backend_selection() {
        assert_eq!(Backend::parse("auto"), Ok(Backend::Auto));
        assert_eq!(Backend::parse("gpu"), Ok(Backend::Gpu));
        assert_eq!(Backend::parse("cpu"), Ok(Backend::Cpu));
        assert!(Backend::parse("typo").is_err());
    }
}
