use super::types::{AppError, ValidationError};
use std::fmt;

/// Human-readable multi-line error report, suitable for logs.
pub struct ErrorReport<'a>(pub &'a AppError);

impl<'a> fmt::Display for ErrorReport<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let e = self.0;
        write!(f, "[{}] {} (code: {})", e.category().as_str(), e, e.code())?;

        // Expand validation field list inline
        if let AppError::Validation(ValidationError::Multiple(fields)) = e {
            write!(f, " — fields:")?;
            for fe in fields {
                write!(f, " {}={}", fe.field, fe.message)?;
            }
        }

        Ok(())
    }
}

/// Terse one-line format for metric labels / tracing spans.
pub fn error_label(e: &AppError) -> String {
    format!("{}:{}", e.category().as_str(), e.code())
}

/// Format validation errors as a human-readable bullet list.
pub fn format_validation(e: &ValidationError) -> String {
    match e {
        ValidationError::Field { field, message } => format!("• {field}: {message}"),
        ValidationError::Multiple(fields) => fields
            .iter()
            .map(|f| format!("• {}: {}", f.field, f.message))
            .collect::<Vec<_>>()
            .join("\n"),
        ValidationError::Format { field, expected } => {
            format!("• {field}: expected {expected}")
        }
    }
}

// Keep ErrorCategory display logic alongside format helpers
use super::types::ErrorCategory;

impl ErrorCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorCategory::Auth => "auth",
            ErrorCategory::Validation => "validation",
            ErrorCategory::Export => "export",
            ErrorCategory::Email => "email",
            ErrorCategory::Storage => "storage",
            ErrorCategory::Contract => "contract",
            ErrorCategory::Webhook => "webhook",
            ErrorCategory::Notification => "notification",
            ErrorCategory::Serialization => "serialization",
            ErrorCategory::Analytics => "analytics",
            ErrorCategory::NotFound => "not_found",
            ErrorCategory::Internal => "internal",
            ErrorCategory::RateLimit => "rate_limit",
            ErrorCategory::BadRequest => "bad_request",
        }
    }
}
