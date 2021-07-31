use std::num::NonZeroU32;

use glassbench::*;
use image::imageops;
use resize::Pixel::RGB8;
use rgb::{FromSlice, RGB};

use fast_image_resize::ImageData;
use fast_image_resize::{CpuExtensions, FilterType, PixelType, ResizeAlg, Resizer};

mod utils;

pub fn bench_downscale_rgb(bench: &mut Bench) {
    let src_image = utils::get_big_rgb_image();
    let new_width = NonZeroU32::new(852).unwrap();
    let new_height = NonZeroU32::new(567).unwrap();

    let alg_names = ["Nearest", "Bilinear", "CatmullRom", "Lanczos3"];

    // image crate
    // https://crates.io/crates/image
    for alg_name in alg_names {
        let filter = match alg_name {
            "Nearest" => imageops::Nearest,
            "Bilinear" => imageops::Triangle,
            "CatmullRom" => imageops::CatmullRom,
            "Lanczos3" => imageops::Lanczos3,
            _ => continue,
        };
        bench.task(format!("image - {}", alg_name), |task| {
            task.iter(|| {
                imageops::resize(&src_image, new_width.get(), new_height.get(), filter);
            })
        });
    }

    // resize crate
    // https://crates.io/crates/resize
    for alg_name in alg_names {
        let resize_src_image = src_image.as_raw().as_rgb();
        let mut dst = vec![RGB::new(0, 0, 0); (new_width.get() * new_height.get()) as usize];
        bench.task(format!("resize - {}", alg_name), |task| {
            let filter = match alg_name {
                "Nearest" => resize::Type::Point,
                "Bilinear" => resize::Type::Triangle,
                "CatmullRom" => resize::Type::Catrom,
                "Lanczos3" => resize::Type::Lanczos3,
                _ => return,
            };
            let mut resize = resize::new(
                src_image.width() as usize,
                src_image.height() as usize,
                new_width.get() as usize,
                new_height.get() as usize,
                RGB8,
                filter,
            )
            .unwrap();
            task.iter(|| {
                resize.resize(resize_src_image, &mut dst).unwrap();
            })
        });
    }

    // fast_image_resize crate;
    for cpu_ext in [
        CpuExtensions::None,
        CpuExtensions::Sse4_1,
        CpuExtensions::Avx2,
    ] {
        let ext_name = match cpu_ext {
            CpuExtensions::None => "rust",
            CpuExtensions::Sse4_1 => "sse4.1",
            CpuExtensions::Avx2 => "avx2",
            _ => continue,
        };
        for alg_name in alg_names {
            let src_rgba_image = utils::get_big_rgba_image();
            let buf: Vec<u32> = src_rgba_image
                .as_raw()
                .chunks_exact(4)
                .map(|p| u32::from_le_bytes([p[0], p[1], p[2], p[3]]))
                .collect();
            let src_image_data = ImageData::from_pixels(
                NonZeroU32::new(src_image.width()).unwrap(),
                NonZeroU32::new(src_image.height()).unwrap(),
                buf,
                PixelType::U8x4,
            )
            .unwrap();
            let src_view = src_image_data.src_view();
            let mut dst_image = ImageData::new(new_width, new_height, PixelType::U8x4);
            let mut dst_view = dst_image.dst_view();

            let resize_alg = match alg_name {
                "Nearest" => ResizeAlg::Nearest,
                "Bilinear" => ResizeAlg::Convolution(FilterType::Bilinear),
                "CatmullRom" => ResizeAlg::Convolution(FilterType::CatmulRom),
                "Lanczos3" => ResizeAlg::Convolution(FilterType::Lanczos3),
                _ => return,
            };
            let mut fast_resizer = Resizer::new(resize_alg);

            unsafe {
                fast_resizer.reset_internal_buffers();
                fast_resizer.set_cpu_extensions(cpu_ext);
            }

            bench.task(format!("fir {} - {}", ext_name, alg_name), |task| {
                task.iter(|| {
                    fast_resizer.resize(&src_view, &mut dst_view);
                })
            });
        }
    }

    utils::print_md_table(bench);
}

glassbench!("Compare resize of RGB image", bench_downscale_rgb,);