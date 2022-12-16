use crate::{
    chars::wide_char_to_multi_byte,
    request::{SaoriCharset, SaoriRequest, SaoriVersion},
};

#[derive(PartialEq, Debug)]
pub enum SaoriStatus {
    OK,
    NoContent,
    BadRequest,
    #[allow(dead_code)]
    InternalServerError,
}

impl SaoriStatus {
    pub fn to_code(&self) -> u16 {
        match self {
            SaoriStatus::OK => 200,
            SaoriStatus::NoContent => 204,
            SaoriStatus::BadRequest => 400,
            SaoriStatus::InternalServerError => 500,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriStatus::OK => "OK",
            SaoriStatus::NoContent => "No Content",
            SaoriStatus::BadRequest => "Bad Request",
            SaoriStatus::InternalServerError => "Internal Server Error",
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum SaoriResponseError {
    DecodeFailed,
}

#[derive(PartialEq, Debug)]
pub struct SaoriResponse {
    version: SaoriVersion,
    status: SaoriStatus,
    result: String,
    value: Vec<String>,
    charset: SaoriCharset,
}

impl SaoriResponse {
    /// status がBad Request である自身を生成する
    pub fn new_bad_request() -> SaoriResponse {
        SaoriResponse {
            version: SaoriVersion::V1_0,
            status: SaoriStatus::BadRequest,
            result: String::new(),
            value: Vec::new(),
            charset: SaoriCharset::UTF8,
        }
    }

    /// リクエストから自身を生成する
    pub fn from_request(request: &SaoriRequest) -> SaoriResponse {
        SaoriResponse {
            version: request.version().clone(),
            status: SaoriStatus::NoContent,
            result: String::new(),
            value: Vec::new(),
            charset: request.charset().clone(),
        }
    }

    #[allow(dead_code)]
    pub fn status(&self) -> &SaoriStatus {
        &self.status
    }
    #[allow(dead_code)]
    pub fn set_status(&mut self, status: SaoriStatus) {
        self.status = status;
    }

    #[allow(dead_code)]
    pub fn result(&self) -> &str {
        &self.result
    }
    #[allow(dead_code)]
    pub fn set_result(&mut self, result: String) {
        self.result = result;

        self.on_change_result_and_value();
    }

    #[allow(dead_code)]
    pub fn value(&self) -> &[String] {
        &self.value
    }
    #[allow(dead_code)]
    pub fn set_value(&mut self, value: Vec<String>) {
        self.value = value;

        self.on_change_result_and_value();
    }

    /// resultとvalueが変更されたときに呼ばれる
    /// statusの切替を行う(Ok <=> No Content)
    fn on_change_result_and_value(&mut self) {
        match self.status {
            SaoriStatus::BadRequest | SaoriStatus::InternalServerError => return,
            _ => {
                if !self.result.is_empty() || !self.value.is_empty() {
                    self.status = SaoriStatus::OK;
                } else {
                    self.status = SaoriStatus::NoContent
                }
            }
        }
    }

    /// 自身をエンコードされた文字バイト列にして返す
    pub fn to_encoded_bytes(&mut self) -> Result<Vec<i8>, SaoriResponseError> {
        let req = self.to_string();

        let mut wide_chars: Vec<u16> = req.encode_utf16().collect();

        let result = wide_char_to_multi_byte(&mut wide_chars, self.charset.codepage())
            .map_err(|_| SaoriResponseError::DecodeFailed)?;

        Ok(result)
    }

    /// 自身を文字列にして返す
    fn to_string(&self) -> String {
        let mut result = String::new();

        result.push_str(&format!(
            "{} {} {}\r\nCharset: {}\r\n",
            self.version.to_str(),
            self.status.to_code(),
            self.status.to_str(),
            self.charset.to_str()
        ));
        match self.status {
            SaoriStatus::OK => {
                if !self.result.is_empty() {
                    result.push_str(&format!("Result: {}\r\n", self.result));
                }

                for (index, value) in self.value.iter().enumerate() {
                    result.push_str(&format!("Value{}: {}\r\n", index, value));
                }
            }
            _ => {}
        }
        result.push_str("\r\n\0");

        return result;
    }
}
