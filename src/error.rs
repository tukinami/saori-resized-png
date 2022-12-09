use fast_image_resize as fir;

#[derive(Debug)]
pub(crate) enum ResizedPngError {
    Unsupported,
    NotFound,
    IoError,
    DecodingError,
    EncodingError,
    ParameterError,
    LimitsError,
    InputSizeError,
}

impl ResizedPngError {
    pub(crate) fn to_code(&self) -> u32 {
        match self {
            Self::Unsupported => 1,
            Self::NotFound => 2,
            Self::IoError => 3,
            Self::DecodingError => 4,
            Self::EncodingError => 5,
            Self::ParameterError => 6,
            Self::LimitsError => 7,
            Self::InputSizeError => 8,
        }
    }
}

impl From<std::io::Error> for ResizedPngError {
    fn from(e: std::io::Error) -> Self {
        match e.kind() {
            std::io::ErrorKind::NotFound => Self::NotFound,
            _ => Self::IoError,
        }
    }
}

impl From<image::error::ImageError> for ResizedPngError {
    fn from(e: image::error::ImageError) -> Self {
        match e {
            image::ImageError::Decoding(_) => Self::DecodingError,
            image::ImageError::Encoding(_) => Self::EncodingError,
            image::ImageError::Parameter(_) => Self::ParameterError,
            image::ImageError::Limits(_) => Self::LimitsError,
            image::ImageError::Unsupported(_) => Self::Unsupported,
            image::ImageError::IoError(e) => e.into(),
        }
    }
}

impl From<fir::ImageBufferError> for ResizedPngError {
    fn from(e: fir::ImageBufferError) -> Self {
        match e {
            fir::ImageBufferError::InvalidBufferSize => Self::InputSizeError,
            fir::ImageBufferError::InvalidBufferAlignment => Self::DecodingError,
        }
    }
}
