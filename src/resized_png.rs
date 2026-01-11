use std::{num::NonZeroU32, path::PathBuf};

// use fast_image_resize::{self as fir, IntoImageView, ResizeOptions};
// use image::ImageReader;

use image::ImageReader;

use fast_image_resize::{images::Image, PixelType, ResizeOptions, Resizer};

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
    let input_src = ImageReader::open(src_path)
        .and_then(|v| v.with_guessed_format())?
        .decode()?;

    // サイズが計算できないときは、何もせず終了。
    let (output_width, output_height) = match output_size(
        width_command,
        height_command,
        input_src.width(),
        input_src.height(),
    ) {
        Some(v) => v,
        None => return Ok(()),
    };

    let input_image = Image::from_vec_u8(
        input_src.width(),
        input_src.height(),
        input_src.into_rgba8().into_vec(),
        PixelType::U8x4,
    )
    .map_err(|_| ResizedPngError::Unsupported)?;

    let mut output_image = Image::new(output_width.into(), output_height.into(), PixelType::U8x4);

    let mut resizer = Resizer::new();
    resizer
        .resize(
            &input_image,
            &mut output_image,
            Some(&ResizeOptions::default()),
        )
        .map_err(|_| ResizedPngError::Unsupported)?;

    image::save_buffer_with_format(
        dist_path,
        output_image.buffer(),
        output_image.width(),
        output_image.height(),
        image::ColorType::Rgba8,
        image::ImageFormat::Png,
    )?;

    Ok(())
}

fn output_size(
    width_command: i64,
    height_command: i64,
    input_width: u32,
    input_height: u32,
) -> Option<(NonZeroU32, NonZeroU32)> {
    // 両方とも0未満ならサイズなし。
    if width_command < 0 && height_command < 0 {
        return None;
    }

    // command が0の場合は元のサイズが指定されているとして扱う。
    let width_origin = match width_command {
        0 => input_width as i64,
        w => w,
    };
    let height_origin = match height_command {
        0 => input_height as i64,
        h => h,
    };

    // originが0未満の場合はもう片方の拡大率に従う。
    let width_temp = match width_origin {
        w if w < 0 => {
            let ratio = height_origin as f64 / input_height as f64;

            (input_width as f64 * ratio) as u32
        }
        w => w as u32,
    };
    let height_temp = match height_origin {
        h if h < 0 => {
            let ratio = width_origin as f64 / input_width as f64;

            (input_height as f64 * ratio) as u32
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
            let input_width = 100;
            let input_height = 200;

            assert!(
                output_size(width_command, height_command, input_width, input_height).is_none()
            );
        }

        #[test]
        fn original_value_when_width_and_height_are_0() {
            let width_command = 0;
            let height_command = 0;
            let input_width = 100;
            let input_height = 200;

            let (width, height) =
                output_size(width_command, height_command, input_width, input_height).unwrap();

            assert_eq!(width, NonZeroU32::new(input_width).unwrap());
            assert_eq!(height, NonZeroU32::new(input_height).unwrap());
        }

        #[test]
        fn keep_aspect_ratio_when_one_of_width_and_height_is_minus() {
            let width_command = -1;
            let height_command = 100;
            let input_width = 100;
            let input_height = 200;

            let (width, height) =
                output_size(width_command, height_command, input_width, input_height).unwrap();

            assert_eq!(width, NonZeroU32::new(50).unwrap());
            assert_eq!(height, NonZeroU32::new(100).unwrap());
        }

        #[test]
        fn target_values_when_width_and_henght_are_not_0_and_minis() {
            let width_command = 200;
            let height_command = 300;
            let input_width = 100;
            let input_height = 200;

            let (width, height) =
                output_size(width_command, height_command, input_width, input_height).unwrap();

            assert_eq!(width, NonZeroU32::new(200).unwrap());
            assert_eq!(height, NonZeroU32::new(300).unwrap());
        }
    }
}
