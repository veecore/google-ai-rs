use std::{error::Error as StdError, fmt, io};

use crate::auth::Error as AuthError;

/// Unified error type for the Google Generative AI client
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Errors occurring during client setup/configuration
    Setup(SetupError),
    /// Network-level errors (transport failures, connection issues)
    Net(NetError),
    /// Service-level errors (API responses, business logic errors)
    Service(ServiceError),
    /// Streaming operation errors (both I/O and nested errors)
    Stream(ActionError<io::Error>),
    /// Configuration option errors
    Auth(AuthError),
    /// Invalid parameter passed to API
    InvalidArgument(Box<dyn StdError + Send + Sync>),
    /// Malformed or unsupported content structure
    InvalidContent(Box<dyn StdError + Send + Sync>),
}

impl Error {
    /// Returns the root cause of the error, if available
    pub fn root_cause(&self) -> &(dyn StdError + 'static) {
        match self {
            Error::Setup(e) => e.source().unwrap_or(e),
            Error::Net(e) => e.source().unwrap_or(e),
            Error::Service(e) => e.source().unwrap_or(e),
            Error::Stream(e) => e.source().unwrap_or(e),
            Error::Auth(e) => e.source().unwrap_or(e),
            Error::InvalidArgument(_) => self,
            Error::InvalidContent(_) => self,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Setup(e) => write!(f, "Setup Error: {e}"),
            Error::Net(e) => write!(f, "Network Error: {e}"),
            Error::Service(e) => write!(f, "Service Error: {e}"),
            Error::Stream(e) => write!(f, "Stream Error: {e}"),
            Error::Auth(e) => write!(f, "Authentication Error: {e}"),
            Error::InvalidArgument(msg) => write!(f, "Invalid argument: {msg}"),
            Error::InvalidContent(msg) => write!(f, "Invalid content: {msg}"),
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Error::Setup(e) => e.source(),
            Error::Net(e) => e.source(),
            Error::Service(e) => e.source(),
            Error::Stream(e) => e.source(),
            Error::Auth(e) => e.source(),
            Error::InvalidArgument(e) => e.source(),
            Error::InvalidContent(e) => e.source(),
        }
    }
}

impl From<AuthError> for Error {
    fn from(err: AuthError) -> Self {
        Error::Auth(err)
    }
}

/// Error occurring during client setup/configuration or TryIntoContent
#[derive(Debug)]
pub struct SetupError {
    pub context: String,
    pub err: Box<dyn StdError + Send + Sync>,
}

#[allow(clippy::new_ret_no_self)]
impl SetupError {
    /// Creates a new SetupError with context information
    pub fn new<E>(context: impl Into<String>, err: E) -> Error
    where
        E: StdError + Send + Sync + 'static,
    {
        Error::Setup(Self {
            context: context.into(),
            err: Box::new(err),
        })
    }
}

impl fmt::Display for SetupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (Caused by: {})", self.context, self.err)
    }
}

impl StdError for SetupError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.err.as_ref())
    }
}

/// Error wrapper for operations that might fail due to external actions
#[derive(Debug)]
pub enum ActionError<E: StdError + Send + 'static> {
    /// Error originating from an external action
    Action(E),
    /// Nested client error
    Error(Box<Error>),
}

impl<E: StdError + Send> ActionError<E> {
    /// Determines the primary cause category of the error
    pub fn blame(&self) -> ActionErrorBlame {
        match self {
            ActionError::Action(_) => ActionErrorBlame::Action,
            ActionError::Error(err) => match err.as_ref() {
                Error::Setup(_) => ActionErrorBlame::Unknown,
                Error::Net(_) => ActionErrorBlame::Network,
                Error::Service(_) => ActionErrorBlame::Service,
                Error::Stream(inner) => inner.blame(),
                _ => ActionErrorBlame::Unknown,
            },
        }
    }
}

impl<E: StdError + Send + 'static> fmt::Display for ActionError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ActionError::Action(e) => write!(f, "Action failed: {e}"),
            ActionError::Error(e) => write!(f, "Client error: {e}"),
        }
    }
}

impl<E: StdError + Send> StdError for ActionError<E> {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ActionError::Action(e) => Some(e),
            ActionError::Error(e) => Some(e),
        }
    }
}

/// Categorization of error sources for troubleshooting
#[derive(Debug, PartialEq)]
#[non_exhaustive]
pub enum ActionErrorBlame {
    Network,
    Service,
    Unknown,
    Action,
}

/// Network-related errors (transport layer)
#[derive(Debug)]
#[non_exhaustive]
pub enum NetError {
    TransportFailure(TonicTransportError),
    ServiceUnavailable(TonicStatus),
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetError::TransportFailure(e) => write!(f, "Transport failure: {e}"),
            NetError::ServiceUnavailable(e) => write!(f, "Service unavailable: {e}"),
        }
    }
}

impl StdError for NetError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            NetError::TransportFailure(e) => Some(e),
            NetError::ServiceUnavailable(e) => Some(e),
        }
    }
}

/// Service-level errors (API responses)
#[derive(Debug)]
#[non_exhaustive]
pub enum ServiceError {
    ApiError(TonicStatus),
    InvalidResponse(Box<dyn StdError + Send + Sync>),
    InvalidContent(Box<dyn StdError + Send + Sync>),
}

impl fmt::Display for ServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceError::ApiError(status) => write!(f, "API Error: {status}"),
            ServiceError::InvalidResponse(msg) => write!(f, "Invalid response: {msg}"),
            ServiceError::InvalidContent(msg) => write!(f, "Invalid content: {msg}"),
        }
    }
}

impl StdError for ServiceError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ServiceError::ApiError(e) => Some(e),
            ServiceError::InvalidResponse(_) => None,
            ServiceError::InvalidContent(_) => None,
        }
    }
}

/// Wrapper for Tonic transport errors with improved diagnostics
#[derive(Debug)]
pub struct TonicTransportError(pub Box<tonic::transport::Error>);

impl fmt::Display for TonicTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transport error: {}", self.0)?;
        if let Some(source) = self.0.source() {
            write!(f, " (Caused by: {source})")?;
        }
        Ok(())
    }
}

impl StdError for TonicTransportError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

/// Wrapper for Tonic status errors with enhanced formatting
#[derive(Debug)]
pub struct TonicStatus(pub Box<tonic::Status>);
// TODO: tonic::Status's size been reduced... remove the boxing

impl fmt::Display for TonicStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Status: {}", self.0)?;
        if let Some(source) = self.0.source() {
            write!(f, " (Root cause: {source})")?;
        }
        Ok(())
    }
}

impl StdError for TonicStatus {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

// TODO: Check if it contains "Service was not ready" and report
// as transport
/// Converts Tonic status to appropriate error type
pub(super) fn status_into_error(status: tonic::Status) -> Error {
    if status.source().is_some() {
        Error::Net(NetError::ServiceUnavailable(TonicStatus(Box::new(status))))
    } else {
        Error::Service(ServiceError::ApiError(TonicStatus(Box::new(status))))
    }
}

impl From<ServiceError> for Error {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::InvalidContent(msg) => Error::InvalidContent(msg),
            other => Error::Service(other),
        }
    }
}

impl From<String> for Error {
    fn from(err: String) -> Self {
        Error::InvalidArgument(err.into())
    }
}

impl From<&str> for Error {
    fn from(err: &str) -> Self {
        Error::InvalidArgument(err.into())
    }
}

// TODO: Totally revamp with backward-compatibility
