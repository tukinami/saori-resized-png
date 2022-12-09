use std::path::PathBuf;

use crate::request::*;
use crate::resized_png::{get_image_type, to_resized_png};
use crate::response::*;

/// load時に呼ばれる関数
pub fn load(_path: &str) {}

/// unload時に呼ばれる関数
pub fn unload(_path: &str) {}

/// request GET Version時に呼ばれる関数
pub fn get_version(_path: &str, _request: &SaoriRequest, response: &mut SaoriResponse) {
    response.set_result(String::from(env!("CARGO_PKG_VERSION")));
}

/// request EXECUTE時に呼ばれる関数
/// メインの処理はここに記述する
pub fn execute(path: &str, request: &SaoriRequest, response: &mut SaoriResponse) {
    let args = request.argument();
    let mut path = PathBuf::from(path);
    if !path.is_dir() {
        path.pop();
    }

    if let Some(func) = args.get(0) {
        match func.as_str() {
            "GetImageType" => {
                if let Some(input_path_str) = args.get(1) {
                    let input_path = path.join(input_path_str);

                    let v = get_image_type(&input_path);

                    response.set_result(v.to_string());
                }
            }
            "ToResizedPng" => {
                if let (
                    Some(input_path_str),
                    Some(output_path_str),
                    Some(width_str),
                    Some(height_str),
                ) = (args.get(1), args.get(2), args.get(3), args.get(4))
                {
                    let Ok(width_command) = width_str.parse::<i64>() else { return };
                    let Ok(height_command) = height_str.parse::<i64>() else { return };

                    let input_path = path.clone().join(input_path_str);
                    let output_path = path.join(output_path_str);

                    let v = match to_resized_png(
                        &input_path,
                        &output_path,
                        width_command,
                        height_command,
                    ) {
                        Ok(()) => 0,
                        Err(e) => e.to_code(),
                    };

                    response.set_result(format!("{}", v));
                }
            }
            _ => {}
        }
    }
}
