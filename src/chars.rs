use winapi::{
    shared::{
        minwindef::LPBOOL,
        ntdef::{LPCSTR, LPWSTR, NULL},
    },
    um::{
        stringapiset::{MultiByteToWideChar, WideCharToMultiByte},
        winnls::MB_PRECOMPOSED,
        winnt::LPSTR,
    },
};

pub(crate) fn multi_byte_to_wide_char(from: &[u8], codepage: u32) -> Result<Vec<u16>, ()> {
    let mut from_buf: Vec<i8> = from.iter().map(|v| *v as i8).collect();
    from_buf.push(0);

    let to_buf_size = unsafe {
        MultiByteToWideChar(
            codepage,
            MB_PRECOMPOSED,
            from_buf.as_ptr(),
            -1,
            NULL as LPWSTR,
            0,
        )
    };

    if to_buf_size == 0 {
        return Err(());
    }

    let mut to_buf = vec![0; to_buf_size as usize + 1];
    let result = unsafe {
        MultiByteToWideChar(
            codepage,
            MB_PRECOMPOSED,
            from_buf.as_ptr(),
            -1,
            to_buf.as_mut_ptr(),
            to_buf_size,
        )
    };

    if result == 0 {
        Err(())
    } else {
        Ok(to_buf)
    }
}

pub(crate) fn wide_char_to_multi_byte(from: &mut Vec<u16>, codepage: u32) -> Result<Vec<i8>, ()> {
    from.push(0);

    let to_buf_size = unsafe {
        WideCharToMultiByte(
            codepage,
            0,
            from.as_ptr(),
            -1,
            NULL as LPSTR,
            0,
            NULL as LPCSTR,
            NULL as LPBOOL,
        )
    };

    if to_buf_size == 0 {
        return Err(());
    }

    let mut to_buf: Vec<i8> = vec![0; to_buf_size as usize + 1];
    let result = unsafe {
        WideCharToMultiByte(
            codepage,
            0,
            from.as_ptr(),
            -1,
            to_buf.as_mut_ptr(),
            to_buf_size,
            NULL as LPCSTR,
            NULL as LPBOOL,
        )
    };

    if result == 0 {
        Err(())
    } else {
        Ok(to_buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::request::SaoriCharset;

    use super::*;

    use encoding_rs;

    mod multi_byte_to_wide_char {
        use super::*;

        #[test]
        fn success_when_encoding_and_codepage_is_same() {
            let case = "あいうえお仕様";
            let (case_byte, _encoding, _is_err) = encoding_rs::SHIFT_JIS.encode(case);

            let result =
                multi_byte_to_wide_char(&case_byte, SaoriCharset::ShiftJIS.codepage()).unwrap();

            let p = result.partition_point(|v| *v != 0);

            let result = String::from_utf16_lossy(&result[..p]);

            assert_eq!(&result, case);
        }
    }

    mod wide_char_to_multi_byte {
        use super::*;

        #[test]
        fn success_when_valid_wide_char_and_codepage_with_shift_jis() {
            let case = "あいうえお仕様";
            let mut case_chars: Vec<u16> = case.encode_utf16().collect();

            let result =
                wide_char_to_multi_byte(&mut case_chars, SaoriCharset::ShiftJIS.codepage())
                    .unwrap();

            let result: Vec<u8> = result.iter().map(|v| *v as u8).collect();

            let p = result.partition_point(|v| *v != 0);
            let (encoded, _encoding, _is_err) = encoding_rs::SHIFT_JIS.decode(&result[..p]);

            assert_eq!(&encoded, case);
        }
    }
}
