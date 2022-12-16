mod chars;
mod error;
mod procedure;
mod request;
mod resized_png;
mod response;

use winapi::ctypes::c_long;
use winapi::shared::minwindef::{BOOL, DWORD, HGLOBAL, HINSTANCE, LPVOID, MAX_PATH, TRUE};
use winapi::um::libloaderapi::GetModuleFileNameW;
use winapi::um::winbase::{GlobalAlloc, GlobalFree, GMEM_FIXED};
use winapi::um::winnt::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};

use std::slice;

use crate::request::{SaoriCommand, SaoriRequest};
use crate::response::SaoriResponse;

static mut DLL_PATH: String = String::new();

#[no_mangle]
pub extern "system" fn DllMain(
    h_module: HINSTANCE,
    ul_reason_for_call: DWORD,
    _l_reserved: LPVOID,
) -> BOOL {
    match ul_reason_for_call {
        DLL_PROCESS_ATTACH => {
            register_dll_path(h_module);
        }
        DLL_PROCESS_DETACH => {}
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {
            unload();
        }
        _ => {}
    }
    return TRUE;
}

fn register_dll_path(h_module: HINSTANCE) {
    let mut buf: [u16; MAX_PATH + 1] = [0; MAX_PATH + 1];
    unsafe {
        GetModuleFileNameW(h_module, buf.as_mut_ptr(), MAX_PATH as u32);
    }

    let p = buf.partition_point(|v| *v != 0);

    unsafe {
        DLL_PATH = String::from_utf16_lossy(&buf[..p]);
    }
}

#[no_mangle]
pub extern "cdecl" fn load(h: HGLOBAL, _len: c_long) -> BOOL {
    unsafe { GlobalFree(h) };

    unsafe { procedure::load(&DLL_PATH) };

    return TRUE;
}

#[no_mangle]
pub extern "cdecl" fn unload() -> BOOL {
    unsafe { procedure::unload(&DLL_PATH) };
    return TRUE;
}

#[no_mangle]
pub extern "cdecl" fn request(h: HGLOBAL, len: *mut c_long) -> HGLOBAL {
    // リクエストの取得
    let s = unsafe { hglobal_to_vec_u8(h, *len) };
    unsafe { GlobalFree(h) };

    let request = SaoriRequest::from_u8(&s);

    // 返答の組み立て
    let mut response = match &request {
        Ok(r) => SaoriResponse::from_request(r),
        Err(_e) => SaoriResponse::new_bad_request(),
    };

    if let Ok(r) = request {
        match r.command() {
            SaoriCommand::GetVersion => {
                unsafe { procedure::get_version(&DLL_PATH, &r, &mut response) };
            }
            SaoriCommand::Execute => {
                unsafe { procedure::execute(&DLL_PATH, &r, &mut response) };
            }
        }
    }

    let response_bytes = response.to_encoded_bytes().unwrap_or(Vec::new());

    let response = slice_i8_to_hglobal(len, &response_bytes);

    return response;
}

fn slice_i8_to_hglobal(h_len: *mut c_long, data: &[i8]) -> HGLOBAL {
    let data_len = data.len();

    let h = unsafe { GlobalAlloc(GMEM_FIXED, data_len) };

    unsafe { *h_len = data_len as c_long };

    let h_slice = unsafe { slice::from_raw_parts_mut(h as *mut i8, data_len) };

    for (index, value) in data.iter().enumerate() {
        h_slice[index] = *value;
    }

    return h;
}

fn hglobal_to_vec_u8(h: HGLOBAL, len: c_long) -> Vec<u8> {
    let mut s = vec![0; len as usize + 1];

    let slice = unsafe { slice::from_raw_parts(h as *const u8, len as usize) };

    for (index, value) in slice.iter().enumerate() {
        s[index] = *value;
    }
    s[len as usize] = b'\0';

    return s;
}
