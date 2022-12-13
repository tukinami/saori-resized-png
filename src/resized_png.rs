use std::{num::NonZeroU32, path::PathBuf};

use fast_image_resize as fir;
use image::io::Reader as ImageReader;

use crate::error::ResizedPngError;

pub(crate) fn get_image_type(src_path: &PathBuf) -> &'static str {
    let Ok(reader) = ImageReader::open(src_path).and_then(|v| v.with_guessed_format()) else {
        return "UNKNOWN";
    };

    match reader.format() {
        Some(v) => match v {
            image::ImageFormat::Avif => "AVIF",
            image::ImageFormat::Bmp => "BMP",
            image::ImageFormat::Dds => "DDS",
            image::ImageFormat::Farbfeld => "FARBFELD",
            image::ImageFormat::Gif => "GIF",
            image::ImageFormat::Hdr => "HDR",
            image::ImageFormat::Ico => "ICO",
            image::ImageFormat::Jpeg => "JPEG",
            image::ImageFormat::OpenExr => "OPENEXR",
            image::ImageFormat::Png => "PNG",
            image::ImageFormat::Pnm => "PNM",
            image::ImageFormat::Tga => "TGA",
            image::ImageFormat::Tiff => "TIFF",
            image::ImageFormat::WebP => "WEBP",
            _ => "UNKNOWN",
        },
        None => "UNKNOWN",
    }
}

pub(crate) fn to_resized_png(
    src_path: &PathBuf,
    dist_path: &PathBuf,
    width_command: i64,
    height_command: i64,
) -> Result<(), ResizedPngError> {
    let reader = ImageReader::open(src_path).and_then(|v| v.with_guessed_format())?;
    let input_img = reader.decode()?;

    let pixel_type = fir::PixelType::U8x4;

    let (input_width, input_height) = NonZeroU32::new(input_img.width())
        .zip(NonZeroU32::new(input_img.height()))
        .ok_or(ResizedPngError::InputSizeError)?;

    let mut input_image = fir::Image::from_vec_u8(
        input_width,
        input_height,
        input_img.to_rgba8().into_raw(),
        pixel_type,
    )?;

    let alpha_mul_div = fir::MulDiv::default();
    alpha_mul_div
        .divide_alpha_inplace(&mut input_image.view_mut())
        .expect("limited target pixel type.");

    // サイズが計算できないときは、何もせず終了。
    let (output_width, output_height) =
        match output_size(width_command, height_command, input_width, input_height) {
            Some(v) => v,
            None => return Ok(()),
        };

    let mut output_image = fir::Image::new(output_width, output_height, input_image.pixel_type());
    let mut output_view = output_image.view_mut();

    let mut resizer = fir::Resizer::new(fir::ResizeAlg::Convolution(fir::FilterType::Lanczos3));
    resizer
        .resize(&input_image.view(), &mut output_view)
        .expect("pixel type is same");

    alpha_mul_div
        .divide_alpha_inplace(&mut output_view)
        .expect("limited target pixel type.");

    image::save_buffer_with_format(
        dist_path,
        output_image.buffer(),
        output_image.width().get(),
        output_image.height().get(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;

    Ok(())
}

fn output_size(
    width_command: i64,
    height_command: i64,
    input_width: NonZeroU32,
    input_height: NonZeroU32,
) -> Option<(NonZeroU32, NonZeroU32)> {
    // 両方とも0未満ならサイズなし。
    if width_command < 0 && height_command < 0 {
        return None;
    }

    // command が0の場合は元のサイズが指定されているとして扱う。
    let width_origin = match width_command {
        w if w == 0 => input_width.get() as i64,
        w => w,
    };
    let height_origin = match height_command {
        h if h == 0 => input_height.get() as i64,
        h => h,
    };

    // originが0未満の場合はもう片方の拡大率に従う。
    let width_temp = match width_origin {
        w if w < 0 => {
            let ratio = height_origin as f64 / input_height.get() as f64;

            (input_width.get() as f64 * ratio) as u32
        }
        w => w as u32,
    };
    let height_temp = match height_origin {
        h if h < 0 => {
            let ratio = width_origin as f64 / input_width.get() as f64;

            (input_height.get() as f64 * ratio) as u32
        }
        h => h as u32,
    };

    // tempが0の場合は1にfallbackして返す。
    let width = NonZeroU32::new(width_temp).unwrap_or(NonZeroU32::new(1).unwrap());
    let height = NonZeroU32::new(height_temp).unwrap_or(NonZeroU32::new(1).unwrap());

    Some((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod get_image_type {
        use super::*;

        #[test]
        fn checking_value_when_image_file_exists() {
            let path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_target/image/sample.png");

            assert_eq!(get_image_type(&path), "PNG");
        }

        #[test]
        fn checking_value_when_non_image_file_exists() {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");

            assert_eq!(get_image_type(&path), "UNKNOWN");
        }

        #[test]
        fn checking_value_when_file_does_not_exist() {
            let path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_target/something_wrong.png");
            assert_eq!(get_image_type(&path), "UNKNOWN");
        }
    }

    mod to_resized_png {
        use super::*;

        use tempfile::tempdir;

        #[test]
        fn success_when_input_image_is_png() {
            let out_dir = tempdir().unwrap();

            let src_path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_target/image/sample.png");
            let dist_path = out_dir.path().join("from_png.png");
            let width_command = 50;
            let height_command = 100;

            to_resized_png(&src_path, &dist_path, width_command, height_command).unwrap();

            assert!(dist_path.exists());

            out_dir.close().unwrap();
        }

        #[test]
        fn success_when_input_image_is_webp() {
            let out_dir = tempdir().unwrap();

            let src_path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_target/image/sample.webp");
            let dist_path = out_dir.path().join("from_webp.png");
            let width_command = -1;
            let height_command = 50;

            to_resized_png(&src_path, &dist_path, width_command, height_command).unwrap();

            assert!(dist_path.exists());

            out_dir.close().unwrap();
        }

        #[test]
        fn success_when_input_image_is_bmp() {
            let out_dir = tempdir().unwrap();

            let src_path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_target/image/sample.bmp");
            let dist_path = out_dir.path().join("from_bmp.png");
            let width_command = 50;
            let height_command = -1;

            to_resized_png(&src_path, &dist_path, width_command, height_command).unwrap();

            assert!(dist_path.exists());

            out_dir.close().unwrap();
        }

        #[test]
        fn success_when_input_image_is_jpg() {
            let out_dir = tempdir().unwrap();

            let src_path =
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_target/image/sample.jpg");
            let dist_path = out_dir.path().join("from_jpg.png");
            let width_command = 0;
            let height_command = 0;

            to_resized_png(&src_path, &dist_path, width_command, height_command).unwrap();

            assert!(dist_path.exists());

            out_dir.close().unwrap();
        }
    }

    mod output_size {
        use super::*;

        #[test]
        fn none_when_both_width_and_height_are_minus() {
            let width_command = -1;
            let height_command = -1;
            let input_width = NonZeroU32::new(100).unwrap();
            let input_height = NonZeroU32::new(200).unwrap();

            assert!(
                output_size(width_command, height_command, input_width, input_height).is_none()
            );
        }

        #[test]
        fn original_value_when_width_and_height_are_0() {
            let width_command = 0;
            let height_command = 0;
            let input_width = NonZeroU32::new(100).unwrap();
            let input_height = NonZeroU32::new(200).unwrap();

            let (width, height) =
                output_size(width_command, height_command, input_width, input_height).unwrap();

            assert_eq!(width, input_width);
            assert_eq!(height, input_height);
        }

        #[test]
        fn keep_aspect_ratio_when_one_of_width_and_height_is_minus() {
            let width_command = -1;
            let height_command = 100;
            let input_width = NonZeroU32::new(100).unwrap();
            let input_height = NonZeroU32::new(200).unwrap();

            let (width, height) =
                output_size(width_command, height_command, input_width, input_height).unwrap();

            assert_eq!(width, NonZeroU32::new(50).unwrap());
            assert_eq!(height, NonZeroU32::new(100).unwrap());
        }

        #[test]
        fn target_values_when_width_and_henght_are_not_0_and_minis() {
            let width_command = 200;
            let height_command = 300;
            let input_width = NonZeroU32::new(100).unwrap();
            let input_height = NonZeroU32::new(200).unwrap();

            let (width, height) =
                output_size(width_command, height_command, input_width, input_height).unwrap();

            assert_eq!(width, NonZeroU32::new(200).unwrap());
            assert_eq!(height, NonZeroU32::new(300).unwrap());
        }
    }
}
