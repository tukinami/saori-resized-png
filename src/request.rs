use encoding_rs::*;

#[derive(PartialEq, Debug, Clone)]
pub enum SaoriVersion {
    V1_0,
}

impl SaoriVersion {
    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriVersion::V1_0 => "SAORI/1.0",
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum SaoriCommand {
    Execute,
    GetVersion,
}

impl SaoriCommand {
    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriCommand::Execute => "EXECUTE",
            SaoriCommand::GetVersion => "GET Version",
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum SaoriSecurityLevel {
    Local,
    External,
}

impl SaoriSecurityLevel {
    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriSecurityLevel::Local => "Local",
            SaoriSecurityLevel::External => "External",
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum SaoriCharset {
    ShiftJIS,
    EucJP,
    UTF8,
    ISO2022JP,
}

impl SaoriCharset {
    pub fn to_str(&self) -> &'static str {
        match self {
            SaoriCharset::ShiftJIS => "Shift_JIS",
            SaoriCharset::EucJP => "EUC-JP",
            SaoriCharset::UTF8 => "UTF-8",
            SaoriCharset::ISO2022JP => "ISO-2022-JP",
        }
    }

    pub fn to_encoding(&self) -> &'static Encoding {
        match self {
            SaoriCharset::ShiftJIS => SHIFT_JIS,
            SaoriCharset::EucJP => EUC_JP,
            SaoriCharset::UTF8 => UTF_8,
            SaoriCharset::ISO2022JP => ISO_2022_JP,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum SaoriRequestError {
    Charset(SaoriRequestCharsetError),
    VersionLine(SaoriRequestVersionLineError),
    Argument(SaoriRequestArgumentError),
}

impl From<SaoriRequestCharsetError> for SaoriRequestError {
    fn from(e: SaoriRequestCharsetError) -> SaoriRequestError {
        SaoriRequestError::Charset(e)
    }
}

impl From<SaoriRequestVersionLineError> for SaoriRequestError {
    fn from(e: SaoriRequestVersionLineError) -> SaoriRequestError {
        SaoriRequestError::VersionLine(e)
    }
}

impl From<SaoriRequestArgumentError> for SaoriRequestError {
    fn from(e: SaoriRequestArgumentError) -> SaoriRequestError {
        SaoriRequestError::Argument(e)
    }
}

#[derive(Debug, PartialEq)]
pub enum SaoriRequestCharsetError {
    DecodeFailed,
}

#[derive(Debug, PartialEq)]
pub enum SaoriRequestVersionLineError {
    EmptyRequest,
    NoVersion,
    NoCommand,
}

#[derive(Debug, PartialEq)]
pub enum SaoriRequestArgumentError {
    InvalidSeparator,
    NoIndex,
}

#[derive(PartialEq, Debug)]
pub struct SaoriRequest {
    version: SaoriVersion,
    command: SaoriCommand,
    security_level: Option<SaoriSecurityLevel>,
    argument: Vec<String>,
    charset: SaoriCharset,
    sender: Option<String>,
}

impl SaoriRequest {
    pub fn from_u8(from: &[u8]) -> Result<SaoriRequest, SaoriRequestError> {
        let (body, charset) = SaoriRequest::decode_u8(from)?;

        let (version, command) = SaoriRequest::parse_version_line(&body)?;

        let lines = body.lines();
        let mut security_level = None;
        let mut argument = Vec::new();
        let mut sender = None;

        for line in lines {
            security_level = security_level.or(SaoriRequest::parse_security_level(line));
            SaoriRequest::parse_argument(line, &mut argument)?;
            sender = sender.or(SaoriRequest::parse_sender(line));
        }

        Ok(SaoriRequest {
            version,
            command,
            security_level,
            argument,
            charset,
            sender,
        })
    }

    pub fn version(&self) -> &SaoriVersion {
        &self.version
    }

    pub fn command(&self) -> &SaoriCommand {
        &self.command
    }

    #[allow(dead_code)]
    pub fn security_level(&self) -> Option<&SaoriSecurityLevel> {
        self.security_level.as_ref()
    }

    #[allow(dead_code)]
    pub fn argument(&self) -> &[String] {
        &self.argument
    }

    pub fn charset(&self) -> &SaoriCharset {
        &self.charset
    }

    #[allow(dead_code)]
    pub fn sender(&self) -> Option<&String> {
        self.sender.as_ref()
    }

    /// リクエスト中のCharsetを処理し、デコードする関数。
    fn decode_u8(from: &[u8]) -> Result<(String, SaoriCharset), SaoriRequestCharsetError> {
        let temp = String::from_utf8_lossy(from);
        let temp_lines = temp.lines();

        let mut charset = SaoriCharset::ShiftJIS;

        for line in temp_lines {
            if line.starts_with("Charset: ") {
                if line.ends_with(SaoriCharset::ShiftJIS.to_str()) {
                    charset = SaoriCharset::ShiftJIS;
                } else if line.ends_with(SaoriCharset::EucJP.to_str()) {
                    charset = SaoriCharset::EucJP;
                } else if line.ends_with(SaoriCharset::UTF8.to_str()) {
                    charset = SaoriCharset::UTF8;
                } else if line.ends_with(SaoriCharset::ISO2022JP.to_str()) {
                    charset = SaoriCharset::ISO2022JP;
                } else {
                    charset = SaoriCharset::ShiftJIS;
                }
            }
        }

        let encoding = charset.to_encoding();

        let (body, _used_encoding, has_error) = encoding.decode(from);
        if has_error {
            return Err(SaoriRequestCharsetError::DecodeFailed);
        } else {
            return Ok((body.to_string(), charset));
        }
    }

    /// リクエスト中のバージョン・コマンドを処理する関数。
    fn parse_version_line(
        body: &str,
    ) -> Result<(SaoriVersion, SaoriCommand), SaoriRequestVersionLineError> {
        let first_line = if let Some(v) = body.lines().next() {
            v
        } else {
            return Err(SaoriRequestVersionLineError::EmptyRequest);
        };

        let version = if first_line.ends_with(SaoriVersion::V1_0.to_str()) {
            SaoriVersion::V1_0
        } else {
            return Err(SaoriRequestVersionLineError::NoVersion);
        };

        let command = if first_line.starts_with(SaoriCommand::Execute.to_str()) {
            SaoriCommand::Execute
        } else if first_line.starts_with(SaoriCommand::GetVersion.to_str()) {
            SaoriCommand::GetVersion
        } else {
            return Err(SaoriRequestVersionLineError::NoCommand);
        };

        return Ok((version, command));
    }

    /// リクエスト中のSecurityLevelを処理する関数。
    fn parse_security_level(line: &str) -> Option<SaoriSecurityLevel> {
        if line.starts_with("SecurityLevel: ") {
            if line.ends_with(SaoriSecurityLevel::Local.to_str()) {
                return Some(SaoriSecurityLevel::Local);
            } else if line.ends_with(SaoriSecurityLevel::External.to_str()) {
                return Some(SaoriSecurityLevel::External);
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    /// リクエスト中のArgument*を処理する関数。
    fn parse_argument(
        line: &str,
        argument: &mut Vec<String>,
    ) -> Result<(), SaoriRequestArgumentError> {
        if line.starts_with("Argument") {
            // 行分離
            let mut split = line.splitn(2, ": ");
            let (header, body) = match (split.next(), split.next()) {
                (Some(h), Some(b)) => (h, b),
                (_, _) => {
                    return Err(SaoriRequestArgumentError::InvalidSeparator);
                }
            };
            // 引数番号取得
            let index: String = header.chars().skip(8).collect();
            let index = if let Ok(v) = index.parse::<usize>() {
                v
            } else {
                return Err(SaoriRequestArgumentError::NoIndex);
            };
            // indexが入るようになるまでrequest.argumentを伸張する。
            while argument.len() <= index {
                argument.push(String::new());
            }
            // 引数取得
            argument[index] = body.to_string();
        }
        Ok(())
    }

    /// リクエスト中のSenderを処理する関数
    fn parse_sender(line: &str) -> Option<String> {
        if line.starts_with("Sender: ") {
            let body = line.replace("Sender: ", "");
            return Some(body);
        } else {
            return None;
        }
    }
}

#[cfg(test)]
mod test_request {
    use super::*;

    #[test]
    fn test_decode_u8() {
        let case = "EXECUTE SAORI/1.0\r\nCharset: UTF-8\r\nArgument0: わはは\r\n\r\n\0";
        assert_eq!(
            Ok((case.to_string(), SaoriCharset::UTF8)),
            SaoriRequest::decode_u8(case.as_bytes())
        );

        let case = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\nArgument0: わはは\r\n\r\n\0";
        assert_eq!(
            Err(SaoriRequestCharsetError::DecodeFailed),
            SaoriRequest::decode_u8(case.as_bytes())
        );
    }

    #[test]
    fn test_parse_version_line() {
        let case = "GET Version SAORI/1.0\r\n\r\n\0";
        assert_eq!(
            Ok((SaoriVersion::V1_0, SaoriCommand::GetVersion)),
            SaoriRequest::parse_version_line(case)
        );

        let case = "EXECUTE SAORI/1.0\r\nCharset: UTF-8\r\nArgument0: わはは\r\n\r\n\0";
        assert_eq!(
            Ok((SaoriVersion::V1_0, SaoriCommand::Execute)),
            SaoriRequest::parse_version_line(case)
        );

        let case = "";
        assert_eq!(
            Err(SaoriRequestVersionLineError::EmptyRequest),
            SaoriRequest::parse_version_line(case)
        );

        let case = "SAORI/1.0\r\n\r\n\0";
        assert_eq!(
            Err(SaoriRequestVersionLineError::NoCommand),
            SaoriRequest::parse_version_line(case)
        );

        let case = "GET Version \r\n\r\n\0";
        assert_eq!(
            Err(SaoriRequestVersionLineError::NoVersion),
            SaoriRequest::parse_version_line(case)
        );
    }

    #[test]
    fn test_parse_security_level() {
        let case = "SecurityLevel: Local";
        assert_eq!(
            Some(SaoriSecurityLevel::Local),
            SaoriRequest::parse_security_level(case)
        );

        let case = "SecurityLevel: External";
        assert_eq!(
            Some(SaoriSecurityLevel::External),
            SaoriRequest::parse_security_level(case)
        );

        let case = "Argument0: test";
        assert_eq!(None, SaoriRequest::parse_security_level(case));
    }

    #[test]
    fn test_parse_argument() {
        let mut argument = Vec::new();

        let case = "Argument123: わはは";
        let result = SaoriRequest::parse_argument(case, &mut argument);
        assert_eq!(Ok(()), result);
        assert_eq!(Some(&String::from("わはは")), argument.get(123));

        let case = "Argument124: ふふふ";
        let result = SaoriRequest::parse_argument(case, &mut argument);
        assert_eq!(Ok(()), result);
        let case = "Argument1: へへへ";
        let result = SaoriRequest::parse_argument(case, &mut argument);
        assert_eq!(Ok(()), result);

        assert_eq!(Some(&String::from("ふふふ")), argument.get(124));
        assert_eq!(Some(&String::from("へへへ")), argument.get(1));
        assert_eq!(Some(&String::from("わはは")), argument.get(123));

        let case = "";
        assert_eq!(Ok(()), SaoriRequest::parse_argument(case, &mut argument));

        let case = "Argument 123";
        assert_eq!(
            Err(SaoriRequestArgumentError::InvalidSeparator),
            SaoriRequest::parse_argument(case, &mut argument)
        );

        let case = "Argument: 123";
        assert_eq!(
            Err(SaoriRequestArgumentError::NoIndex),
            SaoriRequest::parse_argument(case, &mut argument)
        );
    }

    #[test]
    fn test_parse_sender() {
        let case = "Sender: materia";
        assert_eq!(
            Some(String::from("materia")),
            SaoriRequest::parse_sender(case)
        );

        let case = "Argument0: 123";
        assert_eq!(None, SaoriRequest::parse_sender(case));
    }

    #[test]
    fn test_from_u8() {
        // 通常
        let case = "EXECUTE SAORI/1.0\r\n
SecurityLevel: Local\r\n
Charset: UTF-8\r\n
Argument0: わはは\r\n
\r\n\0
";
        let expect = SaoriRequest {
            version: SaoriVersion::V1_0,
            command: SaoriCommand::Execute,
            security_level: Some(SaoriSecurityLevel::Local),
            argument: vec![String::from("わはは")],
            charset: SaoriCharset::UTF8,
            sender: None,
        };
        assert_eq!(Ok(expect), SaoriRequest::from_u8(case.as_bytes()));

        // 異常
        let case = "SAORI/1.0\r\n
Argument0: 123\r\n
\r\n\0
";
        assert_eq!(
            Err(SaoriRequestError::VersionLine(
                SaoriRequestVersionLineError::NoCommand
            )),
            SaoriRequest::from_u8(case.as_bytes())
        );
    }
}
