use crate::chars::multi_byte_to_wide_char;

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

    pub fn codepage(&self) -> u32 {
        match self {
            SaoriCharset::ShiftJIS => 932,
            SaoriCharset::EucJP => 20932,
            SaoriCharset::UTF8 => 65001,
            SaoriCharset::ISO2022JP => 50222,
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

        let wide_chars = multi_byte_to_wide_char(from, charset.codepage())
            .map_err(|_| SaoriRequestCharsetError::DecodeFailed)?;

        let p = wide_chars.partition_point(|v| *v != 0);

        Ok((String::from_utf16_lossy(&wide_chars[..p]), charset))
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

        Ok((version, command))
    }

    /// リクエスト中のSecurityLevelを処理する関数。
    fn parse_security_level(line: &str) -> Option<SaoriSecurityLevel> {
        if line.starts_with("SecurityLevel: ") {
            if line.ends_with(SaoriSecurityLevel::Local.to_str()) {
                Some(SaoriSecurityLevel::Local)
            } else if line.ends_with(SaoriSecurityLevel::External.to_str()) {
                Some(SaoriSecurityLevel::External)
            } else {
                None
            }
        } else {
            None
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
            Some(body)
        } else {
            None
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    mod saori_request {
        use super::*;

        mod from_u8 {
            use super::*;

            use encoding_rs;

            #[test]
            fn success_when_valid_request() {
                let case = "EXECUTE SAORI/1.0\r\n
SecurityLevel: Local\r\n
Charset: Shift_JIS\r\n
Argument0: 零\r\n
\r\n\0
";
                let (case_bytes, _encoding, _is_err) = encoding_rs::SHIFT_JIS.encode(case);
                let expect = SaoriRequest {
                    version: SaoriVersion::V1_0,
                    command: SaoriCommand::Execute,
                    security_level: Some(SaoriSecurityLevel::Local),
                    argument: vec![String::from("零")],
                    charset: SaoriCharset::ShiftJIS,
                    sender: None,
                };
                assert_eq!(SaoriRequest::from_u8(&case_bytes), Ok(expect));
            }

            #[test]
            fn failed_when_invalid_request() {
                let case = "SAORI/1.0\r\n
Argument0: 123\r\n
\r\n\0
";
                assert_eq!(
                    SaoriRequest::from_u8(case.as_bytes()),
                    Err(SaoriRequestError::VersionLine(
                        SaoriRequestVersionLineError::NoCommand
                    ))
                );
            }
        }

        mod decode_u8 {
            use super::*;

            use encoding_rs;

            #[test]
            fn success_when_valid_bytes() {
                let case = "EXECUTE SAORI/1.0\r\nCharset: Shift_JIS\r\nArgument0: 一\r\n\r\n";
                let case_string = format!("{}\0", case);

                let (case_bytes, _encoding, _is_err) = encoding_rs::SHIFT_JIS.encode(&case_string);
                assert_eq!(
                    SaoriRequest::decode_u8(&case_bytes),
                    Ok((case.to_string(), SaoriCharset::ShiftJIS))
                );
            }
        }

        mod parse_version_line {
            use super::*;

            #[test]
            fn success_when_valid_command_version() {
                let case = "GET Version SAORI/1.0\r\n\r\n\0";
                assert_eq!(
                    SaoriRequest::parse_version_line(case),
                    Ok((SaoriVersion::V1_0, SaoriCommand::GetVersion))
                );
            }

            #[test]
            fn success_when_valid_command_execute() {
                let case = "EXECUTE SAORI/1.0\r\nCharset: UTF-8\r\nArgument0: 零\r\n\r\n\0";
                assert_eq!(
                    SaoriRequest::parse_version_line(case),
                    Ok((SaoriVersion::V1_0, SaoriCommand::Execute))
                );
            }

            #[test]
            fn failed_when_enmpty_request() {
                let case = "";
                assert_eq!(
                    SaoriRequest::parse_version_line(case),
                    Err(SaoriRequestVersionLineError::EmptyRequest)
                );
            }

            #[test]
            fn failed_when_no_command() {
                let case = "SAORI/1.0\r\n\r\n\0";
                assert_eq!(
                    SaoriRequest::parse_version_line(case),
                    Err(SaoriRequestVersionLineError::NoCommand)
                );
            }

            #[test]
            fn failed_when_no_version() {
                let case = "GET Version \r\n\r\n\0";
                assert_eq!(
                    SaoriRequest::parse_version_line(case),
                    Err(SaoriRequestVersionLineError::NoVersion)
                );
            }
        }

        mod parse_security_level {
            use super::*;

            #[test]
            fn some_value_when_valid_security_line() {
                let case = "SecurityLevel: Local";
                assert_eq!(
                    SaoriRequest::parse_security_level(case),
                    Some(SaoriSecurityLevel::Local)
                );

                let case = "SecurityLevel: External";
                assert_eq!(
                    SaoriRequest::parse_security_level(case),
                    Some(SaoriSecurityLevel::External),
                );
            }

            #[test]
            fn none_when_it_is_not_security_line() {
                let case = "Argument0: test";
                assert!(SaoriRequest::parse_security_level(case).is_none());
            }
        }

        mod parse_argument {
            use super::*;

            #[test]
            fn success_when_valid_argument_lines() {
                let mut arguments = Vec::new();

                let case = "Argument123: 一二三";
                assert_eq!(SaoriRequest::parse_argument(case, &mut arguments), Ok(()));
                assert_eq!(arguments.get(123), Some(&String::from("一二三")));

                let case = "Argument124: 一二四";
                assert_eq!(SaoriRequest::parse_argument(case, &mut arguments), Ok(()));
                let case = "Argument1: 一";
                assert_eq!(SaoriRequest::parse_argument(case, &mut arguments), Ok(()));

                assert_eq!(arguments.get(124), Some(&String::from("一二四")));
                assert_eq!(arguments.get(1), Some(&String::from("一")));
                assert_eq!(arguments.get(123), Some(&String::from("一二三")));
            }

            #[test]
            fn success_when_line_is_empty() {
                let mut arguments = Vec::new();

                let case = "";
                assert_eq!(SaoriRequest::parse_argument(case, &mut arguments), Ok(()));
            }

            #[test]
            fn failed_when_invalid_separator() {
                let mut arguments = Vec::new();

                let case = "Argument 123";
                assert_eq!(
                    SaoriRequest::parse_argument(case, &mut arguments),
                    Err(SaoriRequestArgumentError::InvalidSeparator)
                );
            }

            #[test]
            fn failed_when_invalid_index() {
                let mut arguments = Vec::new();

                let case = "Argument: 123";
                assert_eq!(
                    SaoriRequest::parse_argument(case, &mut arguments),
                    Err(SaoriRequestArgumentError::NoIndex)
                );
            }
        }

        mod parse_sender {
            use super::*;

            #[test]
            fn some_value_when_valid_sender_line() {
                let case = "Sender: materia";
                assert_eq!(
                    SaoriRequest::parse_sender(case),
                    Some(String::from("materia"))
                );
            }

            #[test]
            fn none_when_it_is_not_sender_line() {
                let case = "Argument0: 123";
                assert!(SaoriRequest::parse_sender(case).is_none());
            }
        }
    }
}
