use serde_json::json;
use vk_bot_api::error::{VkError, VkResponseExt};

#[test]
fn test_vk_error_creation() {
    let err = VkError::api_error(14, "Captcha needed");
    assert!(err.is_api_error());
    assert_eq!(err.api_error_code(), Some(14));
}

#[test]
fn test_rate_limit_error() {
    let err = VkError::RateLimit;
    assert!(err.is_rate_limit());
    assert_eq!(err.api_error_code(), None);
}

#[test]
fn test_extract_error_success() {
    let response = json!({
        "response": {
            "items": [1, 2, 3]
        }
    });

    let result = response.clone().extract_error();
    assert!(result.is_ok());
    let value = result.unwrap();
    assert!(value.get("response").is_some());
    assert!(!response.has_error());
}

#[test]
fn test_extract_error_failure() {
    let response = json!({
        "error": {
            "error_code": 5,
            "error_msg": "User authorization failed: invalid session."
        }
    });

    assert!(response.has_error());
    let result = response.extract_error();
    assert!(result.is_err());

    if let Err(VkError::ApiError { code, message }) = result {
        assert_eq!(code, 5);
        assert_eq!(message, "User authorization failed: invalid session.");
    } else {
        panic!("Expected ApiError");
    }
}
