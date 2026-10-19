/// Canonical error code registry.
///
/// Every code follows the pattern `<category>.<variant>` and maps 1-to-1 with
/// a variant in `AppError` / its domain sub-enums. Codes are stable across
/// releases; HTTP status may change without breaking clients that match on codes.
///
/// | Code                              | HTTP | Description                              |
/// |-----------------------------------|------|------------------------------------------|
/// | auth.unauthorized                 | 401  | Missing or invalid credentials           |
/// | auth.forbidden                    | 403  | Credentials valid but access denied      |
/// | auth.token_expired                | 401  | JWT/session token has expired            |
/// | auth.invalid_token                | 401  | Token malformed or signature invalid     |
/// | auth.csrf_mismatch                | 403  | CSRF token absent or mismatched          |
/// | validation.field_error            | 422  | Single field failed validation           |
/// | validation.multiple_errors        | 422  | Several fields failed validation         |
/// | validation.format_error           | 422  | Field value has wrong format             |
/// | export.csv_error                  | 500  | CSV generation failed                    |
/// | export.json_error                 | 500  | JSON export failed                       |
/// | export.pdf_error                  | 500  | PDF generation failed                    |
/// | export.serialization_error        | 500  | Data serialization error during export   |
/// | export.invalid_format             | 400  | Unsupported export format requested      |
/// | email.service_error               | 500  | Upstream email provider failed           |
/// | email.template_error              | 500  | Handlebars template render failed        |
/// | email.invalid_address             | 422  | Recipient address malformed              |
/// | email.delivery_failed             | 500  | SMTP delivery failure                    |
/// | storage.service_error             | 500  | S3/storage backend error                 |
/// | storage.invalid_file              | 422  | File type/size validation failed         |
/// | storage.not_found                 | 404  | Requested file does not exist            |
/// | storage.upload_failed             | 500  | Upload to storage backend failed         |
/// | storage.quota_exceeded            | 429  | Storage quota exceeded                   |
/// | contract.call_failed              | 500  | Stellar/Soroban contract invocation fail |
/// | contract.not_found                | 404  | Contract or object ID not on-chain       |
/// | contract.invalid_state            | 409  | Contract state disallows this operation  |
/// | contract.insufficient_balance     | 402  | Token balance too low                    |
/// | contract.unauthorized             | 403  | Caller not permitted by contract         |
/// | webhook.delivery_failed           | 500  | HTTP POST to webhook URL failed          |
/// | webhook.invalid_url               | 422  | Webhook target URL malformed             |
/// | webhook.not_found                 | 404  | Webhook ID does not exist                |
/// | webhook.signature_mismatch        | 401  | Payload HMAC signature invalid           |
/// | notification.push_failed          | 500  | FCM/push notification delivery failed    |
/// | notification.invalid_token        | 422  | Device registration token invalid        |
/// | notification.service_unavailable  | 503  | Firebase/push service unreachable        |
/// | serialization.json_error          | 500  | JSON (de)serialization error             |
/// | serialization.csv_error           | 500  | CSV (de)serialization error              |
/// | serialization.decode_error        | 400  | Binary/base64 decode error               |
/// | analytics.computation_failed      | 500  | Metric calculation error                 |
/// | analytics.invalid_time_range      | 422  | Start/end time range invalid             |
/// | analytics.data_source_unavailable | 503  | Analytics data store unreachable         |
/// | not_found.<resource>              | 404  | Named resource not found                 |
/// | internal                          | 500  | Unexpected server error                  |
/// | rate_limit.exceeded               | 429  | Too many requests                        |
/// | bad_request                       | 400  | Malformed request                        |

pub const AUTH_UNAUTHORIZED: &str = "auth.unauthorized";
pub const AUTH_FORBIDDEN: &str = "auth.forbidden";
pub const AUTH_TOKEN_EXPIRED: &str = "auth.token_expired";
pub const AUTH_INVALID_TOKEN: &str = "auth.invalid_token";
pub const AUTH_CSRF_MISMATCH: &str = "auth.csrf_mismatch";

pub const VALIDATION_FIELD_ERROR: &str = "validation.field_error";
pub const VALIDATION_MULTIPLE_ERRORS: &str = "validation.multiple_errors";
pub const VALIDATION_FORMAT_ERROR: &str = "validation.format_error";

pub const EXPORT_CSV_ERROR: &str = "export.csv_error";
pub const EXPORT_JSON_ERROR: &str = "export.json_error";
pub const EXPORT_PDF_ERROR: &str = "export.pdf_error";
pub const EXPORT_SERIALIZATION_ERROR: &str = "export.serialization_error";
pub const EXPORT_INVALID_FORMAT: &str = "export.invalid_format";

pub const EMAIL_SERVICE_ERROR: &str = "email.service_error";
pub const EMAIL_TEMPLATE_ERROR: &str = "email.template_error";
pub const EMAIL_INVALID_ADDRESS: &str = "email.invalid_address";
pub const EMAIL_DELIVERY_FAILED: &str = "email.delivery_failed";

pub const STORAGE_SERVICE_ERROR: &str = "storage.service_error";
pub const STORAGE_INVALID_FILE: &str = "storage.invalid_file";
pub const STORAGE_NOT_FOUND: &str = "storage.not_found";
pub const STORAGE_UPLOAD_FAILED: &str = "storage.upload_failed";
pub const STORAGE_QUOTA_EXCEEDED: &str = "storage.quota_exceeded";

pub const CONTRACT_CALL_FAILED: &str = "contract.call_failed";
pub const CONTRACT_NOT_FOUND: &str = "contract.not_found";
pub const CONTRACT_INVALID_STATE: &str = "contract.invalid_state";
pub const CONTRACT_INSUFFICIENT_BALANCE: &str = "contract.insufficient_balance";
pub const CONTRACT_UNAUTHORIZED: &str = "contract.unauthorized";

pub const WEBHOOK_DELIVERY_FAILED: &str = "webhook.delivery_failed";
pub const WEBHOOK_INVALID_URL: &str = "webhook.invalid_url";
pub const WEBHOOK_NOT_FOUND: &str = "webhook.not_found";
pub const WEBHOOK_SIGNATURE_MISMATCH: &str = "webhook.signature_mismatch";

pub const NOTIFICATION_PUSH_FAILED: &str = "notification.push_failed";
pub const NOTIFICATION_INVALID_TOKEN: &str = "notification.invalid_token";
pub const NOTIFICATION_SERVICE_UNAVAILABLE: &str = "notification.service_unavailable";

pub const SERIALIZATION_JSON_ERROR: &str = "serialization.json_error";
pub const SERIALIZATION_CSV_ERROR: &str = "serialization.csv_error";
pub const SERIALIZATION_DECODE_ERROR: &str = "serialization.decode_error";

pub const ANALYTICS_COMPUTATION_FAILED: &str = "analytics.computation_failed";
pub const ANALYTICS_INVALID_TIME_RANGE: &str = "analytics.invalid_time_range";
pub const ANALYTICS_DATA_SOURCE_UNAVAILABLE: &str = "analytics.data_source_unavailable";

pub const INTERNAL: &str = "internal";
pub const RATE_LIMIT_EXCEEDED: &str = "rate_limit.exceeded";
pub const BAD_REQUEST: &str = "bad_request";
