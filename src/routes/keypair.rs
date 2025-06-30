use axum::{Json, response::IntoResponse, http::StatusCode};
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use bs58::encode as bs58_encode;
use serde::Serialize;
use std::error::Error;
use std::fmt;

#[derive(Serialize)]
pub struct KeypairResponse {
    pub pubkey: String,
    pub secret: String,
}

#[derive(Serialize)]
pub struct SuccessResponse<T> {
    pub success: bool,
    pub data: T,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub success: bool,
    pub error: String,
}

#[derive(Debug)]
pub struct KeypairGenerationError {
    message: String,
}

impl fmt::Display for KeypairGenerationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for KeypairGenerationError {}

pub async fn generate_keypair() -> impl IntoResponse {
    match generate_keypair_internal().await {
        Ok(keypair_response) => {
            let success_response = SuccessResponse {
                success: true,
                data: keypair_response,
            };
            (StatusCode::OK, Json(success_response)).into_response()
        }
        Err(e) => {
            let error_response = ErrorResponse {
                success: false,
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

async fn generate_keypair_internal() -> Result<KeypairResponse, KeypairGenerationError> {
    let mut csprng = OsRng;
    
    // Generate the keypair
    let kp = Keypair::generate(&mut csprng);
    
    // Encode public key
    let pubkey = bs58_encode(kp.public.to_bytes()).into_string();
    
    // Encode the full keypair (64 bytes: 32 private + 32 public)
    let secret = bs58_encode(kp.to_bytes()).into_string();
    
    Ok(KeypairResponse { pubkey, secret })
}

// Alternative endpoint that returns only the public key for security
pub async fn generate_pubkey_only() -> impl IntoResponse {
    match generate_pubkey_internal().await {
        Ok(pubkey) => {
            let success_response = SuccessResponse {
                success: true,
                data: serde_json::json!({ "pubkey": pubkey }),
            };
            (StatusCode::OK, Json(success_response)).into_response()
        }
        Err(e) => {
            let error_response = ErrorResponse {
                success: false,
                error: e.to_string(),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

async fn generate_pubkey_internal() -> Result<String, KeypairGenerationError> {
    let mut csprng = OsRng;
    let kp = Keypair::generate(&mut csprng);
    let pubkey = bs58_encode(kp.public.to_bytes()).into_string();
    Ok(pubkey)
}

// Utility function to validate a base58 encoded keypair
pub fn validate_keypair_format(keypair_str: &str) -> Result<(), KeypairGenerationError> {
    match bs58::decode(keypair_str).into_vec() {
        Ok(bytes) => {
            if bytes.len() != 64 {
                return Err(KeypairGenerationError {
                    message: "Invalid keypair length. Expected 64 bytes.".to_string(),
                });
            }
            Ok(())
        }
        Err(_) => Err(KeypairGenerationError {
            message: "Invalid base58 encoding".to_string(),
        }),
    }
}
