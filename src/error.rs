use thiserror::Error;

#[derive(Error, Debug)]
pub enum VkError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("VK API error {code}: {message}")]
    ApiError { code: i32, message: String },

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Authentication error: {0}")]
    AuthError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    #[error("{0}")]
    Custom(String),
}

impl VkError {
    pub fn api_error<T: Into<i32>, M: Into<String>>(code: T, message: M) -> Self {
        VkError::ApiError {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn is_rate_limit(&self) -> bool {
        matches!(self, VkError::RateLimit)
    }

    pub fn is_api_error(&self) -> bool {
        matches!(self, VkError::ApiError { .. })
    }

    pub fn api_error_code(&self) -> Option<i32> {
        match self {
            VkError::ApiError { code, .. } => Some(*code),
            _ => None,
        }
    }
}

pub type VkResult<T> = Result<T, VkError>;

impl From<&str> for VkError {
    fn from(s: &str) -> Self {
        VkError::Custom(s.to_string())
    }
}

impl From<String> for VkError {
    fn from(s: String) -> Self {
        VkError::Custom(s)
    }
}

pub trait VkResponseExt {
    fn extract_error(self) -> VkResult<serde_json::Value>;
    fn has_error(&self) -> bool;
}

impl VkResponseExt for serde_json::Value {
    fn extract_error(self) -> VkResult<serde_json::Value> {
        if let Some(error) = self.get("error") {
            let code = error["error_code"]
                .as_i64()
                .ok_or_else(|| VkError::InvalidResponse("Missing error_code".to_string()))?;

            let message = error["error_msg"]
                .as_str()
                .ok_or_else(|| VkError::InvalidResponse("Missing error_msg".to_string()))?
                .to_string();

            Err(VkError::api_error(code as i32, message))
        } else {
            Ok(self)
        }
    }

    fn has_error(&self) -> bool {
        self.get("error").is_some()
    }
}
