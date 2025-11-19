use std::fmt::{Debug, Formatter};
use std::ops::Deref;
use thiserror::Error;

/// Errors that can occur during route validation and registration
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum RouterError {
    /// Multiple routes are registered with the same path
    #[error(
        "Duplicate route detected:\n\
            Path: '{path}'\n\
            First defined in: {first_component}\n\
            Also defined in: {second_component}\n\
        Each route path must be unique."
    )]
    DuplicateRoute {
        path: String,
        first_component: String,
        second_component: String,
    },

    /// No routes have been registered
    #[error(
        "No routes have been registered.\n\
        \n\
        Make sure you have:\n\
            1. Defined routes using #[route(\"/path\")]\n\
            2. Imported all modules containing routes\n\
            3. Called the route functions at least once (for WASM)"
    )]
    NoRoutesRegistered,

    /// Route path is invalid
    #[error("Invalid route path '{path}': {reason}")]
    InvalidRoutePath { path: String, reason: String },
}

/// Result type for router operations
pub type Result<T> = std::result::Result<T, RouterError>;

/// Multiple validation errors collected together
#[derive(Error, Debug, Clone)]
#[error(
    "Route validation failed with {count} error(s):\n{errors}",
    count = .errors_list.len(),
    errors = self.build_display()
)]
pub struct ValidationErrors {
    #[source]
    errors_list: RouterErrorsList,
}

impl ValidationErrors {
    /// Create a new validation error collection
    pub fn new() -> Self {
        Self {
            errors_list: RouterErrorsList(Vec::new()),
        }
    }

    /// Add a new error to the collection
    pub fn add(&mut self, error: RouterError) {
        self.errors_list.0.push(error);
    }

    /// Check if the collection is empty
    pub fn is_empty(&self) -> bool {
        self.errors_list.is_empty()
    }

    /// Get the number of errors in the collection
    pub fn len(&self) -> usize {
        self.errors_list.len()
    }

    /// Get the errors as a string
    pub fn errors(&self) -> &[RouterError] {
        &self.errors_list
    }

    fn build_display(&self) -> String {
        self.errors_list
            .iter()
            .enumerate()
            .map(|(i, err)| format!("\n{}. {}", i + 1, err))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl Default for ValidationErrors {
    fn default() -> Self {
        Self::new()
    }
}

/// Wrapper around a vector of router errors
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RouterErrorsList(Vec<RouterError>);

impl std::error::Error for RouterErrorsList {}

impl std::fmt::Display for RouterErrorsList {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<Vec<RouterError>> for RouterErrorsList {
    fn from(errors: Vec<RouterError>) -> Self {
        Self(errors)
    }
}

impl Deref for RouterErrorsList {
    type Target = Vec<RouterError>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_router_error_duplicate_display() {
        let error = RouterError::DuplicateRoute {
            path: "/test".to_string(),
            first_component: "module::Component1".to_string(),
            second_component: "module::Component2".to_string(),
        };

        let message = error.to_string();
        assert!(message.contains("Duplicate route"));
        assert!(message.contains("/test"));
        assert!(message.contains("Component1"));
        assert!(message.contains("Component2"));
    }

    #[test]
    fn test_no_routes_error_display() {
        let error = RouterError::NoRoutesRegistered;
        let message = error.to_string();

        assert!(message.contains("No routes"));
        assert!(message.contains("#[route"));
        assert!(message.contains("Imported all modules"));
    }

    #[test]
    fn test_invalid_path_error_display() {
        let error = RouterError::InvalidRoutePath {
            path: "invalid".to_string(),
            reason: "must start with '/'".to_string(),
        };

        let message = error.to_string();
        assert!(message.contains("Invalid route path"));
        assert!(message.contains("invalid"));
        assert!(message.contains("must start"));
    }

    #[test]
    fn test_validation_errors_single() {
        let mut errors = ValidationErrors::new();
        errors.add(RouterError::NoRoutesRegistered);

        let message = errors.to_string();
        assert!(message.contains("1 error"));
        assert!(message.contains("No routes"));
    }

    #[test]
    fn test_validation_errors_multiple() {
        let mut errors = ValidationErrors::new();
        errors.add(RouterError::DuplicateRoute {
            path: "/test1".to_string(),
            first_component: "A".to_string(),
            second_component: "B".to_string(),
        });
        errors.add(RouterError::DuplicateRoute {
            path: "/test2".to_string(),
            first_component: "C".to_string(),
            second_component: "D".to_string(),
        });

        let message = errors.to_string();
        assert!(message.contains("2 error"));
        assert!(message.contains("/test1"));
        assert!(message.contains("/test2"));
        assert!(message.contains("1."));
        assert!(message.contains("2."));
    }

    #[test]
    fn test_validation_errors_is_empty() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());
        assert_eq!(errors.len(), 0);

        errors.add(RouterError::NoRoutesRegistered);
        assert!(!errors.is_empty());
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validation_errors_access() {
        let mut errors = ValidationErrors::new();
        errors.add(RouterError::NoRoutesRegistered);
        errors.add(RouterError::InvalidRoutePath {
            path: "/test".to_string(),
            reason: "test".to_string(),
        });

        assert_eq!(errors.errors().len(), 2);
        assert!(matches!(
            errors.errors()[0],
            RouterError::NoRoutesRegistered
        ));
        assert!(matches!(
            errors.errors()[1],
            RouterError::InvalidRoutePath { .. }
        ));
    }

    #[test]
    fn test_router_error_implements_error_trait() {
        let error: Box<dyn std::error::Error> = Box::new(RouterError::NoRoutesRegistered);
        assert!(error.to_string().contains("No routes"));
    }

    #[test]
    fn test_validation_errors_implements_error_trait() {
        let mut errors = ValidationErrors::new();
        errors.add(RouterError::NoRoutesRegistered);

        let error: Box<dyn std::error::Error> = Box::new(errors);
        assert!(error.to_string().contains("validation failed"));
    }
}
