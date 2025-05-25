use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rsa::{
    pkcs1::{DecodeRsaPrivateKey, Error as Pkcs1Error},
    pkcs1v15::SigningKey,
    pkcs8::{DecodePrivateKey, Error as Pkcs8Error},
    signature::{RandomizedSigner, SignatureEncoding},
    RsaPrivateKey,
};
use serde::Deserialize;
use serde_json::json;
use sha2::Sha256;
use std::{
    path::Path,
    sync::Arc,
    time::{Duration, SystemTime, SystemTimeError},
};
use thiserror::Error;
use tokio::sync::RwLock;
use tonic::metadata::{Ascii, MetadataValue};

/// Authentication configuration options
#[derive(Clone, Debug)]
pub enum Auth {
    /// API key authentication (simple but less secure)
    ApiKey(String),
    /// JWT-based service account authentication (recommended for production)
    TokenSource(TokenSource),
}

impl<S: Into<String>> From<S> for Auth {
    fn from(value: S) -> Self {
        Auth::ApiKey(value.into())
    }
}

/// JSON Web Token configuration for service account authentication
#[derive(Deserialize, Clone, Debug)]
pub struct JWTConfig {
    /// Service account client email (format: name@project.iam.gserviceaccount.com)
    #[serde(rename = "client_email")]
    pub client_email: String,

    /// RSA private key in PEM or DER format (keep secure!)
    #[serde(rename = "private_key")]
    pub private_key: String,

    /// Optional private key identifier from Google Cloud
    #[serde(rename = "private_key_id")]
    pub private_key_id: String,

    /// Token lifetime duration (default: 1 hour, max 1 hour recommended)
    #[serde(skip)]
    pub lifetime: Option<Duration>,
}

/// Token generation source types
#[derive(Clone, Debug)]
pub enum TokenSource {
    /// JSON Web Token authentication flow
    Jwt { jwt: Box<JwtService> },
}

/// Authentication error types
#[derive(Debug, Error)]
pub enum Error {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Private key parsing failed: {0}")]
    PrivateKey(#[from] PrivateKeyError),

    #[error("System time error: {0}")]
    SystemTime(#[from] SystemTimeError),

    #[error("Token generation failed: {0}")]
    TokenGeneration(String),

    #[error("Invalid header value")]
    InvalidHeader,

    #[error("Invalid token lifetime")]
    InvalidLifetime,
}

/// Private key parsing specific errors
#[derive(Debug, Error)]
pub enum PrivateKeyError {
    #[error("PKCS#1 parsing error: {0}")]
    Pkcs1(#[from] Pkcs1Error),

    #[error("PKCS#8 parsing error: {0}")]
    Pkcs8(#[from] Pkcs8Error),

    #[error("PEM format error: {0}")]
    Pem(#[from] pem::PemError),
}

const DEFAULT_TOKEN_LIFETIME: Duration = Duration::from_secs(3600);
const MAX_TOKEN_LIFETIME: Duration = Duration::from_secs(3600);
const JWT_AUDIENCE: &str = "https://generativelanguage.googleapis.com/";
const API_KEY_HEADER: &str = "x-goog-api-key";
const AUTH_HEADER: &str = "authorization";

impl Auth {
    /// Creates API key authentication
    pub fn new(api_key: &str) -> Self {
        Self::ApiKey(api_key.to_owned())
    }

    /// Creates service account authentication from JSON file
    ///
    /// # Example
    /// ```
    /// # use google_ai_rs::auth::Auth;
    /// # async fn f() -> Result<(), Box<dyn std::error::Error>> {
    /// let auth = Auth::service_account("path/to/service-account.json")
    ///     .await
    ///     .expect("Valid service account");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn service_account<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self::TokenSource(
            TokenSource::from_service_account(path).await?,
        ))
    }

    /// Creates JWT authentication from configuration
    pub async fn from_jwt_config(config: JWTConfig) -> Result<Self, Error> {
        Ok(Self::TokenSource(TokenSource::from_jwt(config).await?))
    }
}

impl TokenSource {
    /// Creates service account authentication from JSON file
    pub async fn from_service_account<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let json = tokio::fs::read(path).await?;
        let config: JWTConfig = serde_json::from_slice(&json)?;
        Self::from_jwt(config).await
    }

    /// Creates JWT authentication from configuration
    pub async fn from_jwt(config: JWTConfig) -> Result<Self, Error> {
        let private_key = parse_private_key(config.private_key.as_bytes())?;
        let signing_key = SigningKey::<Sha256>::new(private_key);

        Ok(Self::Jwt {
            jwt: Box::new(JwtService {
                config,
                signing_key,
                cache: Arc::new(RwLock::new(JwtCache {
                    token: MetadataValue::from_static(""),
                    expires_at: SystemTime::now(),
                })),
            }),
        })
    }
}

/// Adds authentication headers to gRPC requests
pub(super) async fn add_auth<T>(request: &mut tonic::Request<T>, auth: &Auth) -> Result<(), Error> {
    match auth {
        Auth::ApiKey(key) => {
            let value = key.parse().map_err(|_| Error::InvalidHeader)?;
            request.metadata_mut().insert(API_KEY_HEADER, value);
        }
        Auth::TokenSource(TokenSource::Jwt { jwt }) => {
            let token = jwt.get_token().await?;
            request.metadata_mut().insert(AUTH_HEADER, token);
        }
    }
    Ok(())
}

use hidden::JwtService;

mod hidden {
    use super::*;

    /// JWT token service with caching
    #[derive(Clone, Debug)]
    pub struct JwtService {
        pub(super) config: JWTConfig,
        pub(super) signing_key: SigningKey<Sha256>,
        pub(super) cache: Arc<RwLock<JwtCache>>,
    }
}

/// Cached JWT token data
#[derive(Debug)]
struct JwtCache {
    token: MetadataValue<Ascii>,
    expires_at: SystemTime,
}

impl JwtService {
    /// Generates a new signed JWT token
    async fn generate_token(&self) -> Result<(String, SystemTime), Error> {
        let header = json!({
            "alg": "RS256",
            "typ": "JWT",
            "kid": self.config.private_key_id
        });

        let now = SystemTime::now();
        let iat = now.duration_since(SystemTime::UNIX_EPOCH)?.as_secs();
        let lifetime = self.config.lifetime.unwrap_or(DEFAULT_TOKEN_LIFETIME);

        // Validate token lifetime
        if lifetime > MAX_TOKEN_LIFETIME {
            return Err(Error::InvalidLifetime);
        }

        let exp = iat + lifetime.as_secs();
        let claims = json!({
            "iss": self.config.client_email,
            "sub": self.config.client_email,
            "aud": JWT_AUDIENCE,
            "exp": exp,
            "iat": iat
        });

        let encoded_header = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?);
        let encoded_claims = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims)?);
        let message = format!("{encoded_header}.{encoded_claims}");

        let signature = self
            .signing_key
            .sign_with_rng(&mut rand::thread_rng(), message.as_bytes())
            .to_bytes();

        let encoded_sig = URL_SAFE_NO_PAD.encode(signature);
        let jwt = format!("{message}.{encoded_sig}");
        let expires_at = now + lifetime;

        Ok((jwt, expires_at))
    }

    /// Retrieves valid token from cache or generates new one
    async fn get_token(&self) -> Result<MetadataValue<Ascii>, Error> {
        // Fast path: check cache with read lock
        {
            let cache = self.cache.read().await;
            if SystemTime::now() < cache.expires_at {
                return Ok(cache.token.clone());
            }
        }

        // Slow path: regenerate token with write lock
        let (new_token, expires_at) = self.generate_token().await?;
        let mut cache = self.cache.write().await;

        *cache = JwtCache {
            token: format!("Bearer {new_token}")
                .parse()
                .map_err(|_| Error::InvalidHeader)?,
            expires_at,
        };

        Ok(cache.token.clone())
    }
}

/// Parses RSA private key from multiple formats
fn parse_private_key(bytes: &[u8]) -> Result<RsaPrivateKey, PrivateKeyError> {
    // Try PEM format first
    if let Ok(pem) = pem::parse(bytes) {
        return RsaPrivateKey::from_pkcs8_der(pem.contents())
            .or_else(|_| RsaPrivateKey::from_pkcs1_der(pem.contents()))
            .map_err(Into::into);
    }

    // Fallback to DER format
    RsaPrivateKey::from_pkcs8_der(bytes)
        .or_else(|_| RsaPrivateKey::from_pkcs1_der(bytes))
        .map_err(Into::into)
}
