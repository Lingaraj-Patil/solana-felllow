use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use bs58::{encode as bs58_encode, decode as bs58_decode};
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine;
use axum::http::StatusCode;
use serde_json::json;

#[derive(Deserialize)]
pub struct SignRequest {
    pub message: String,
    pub secret: String,
}

#[derive(Serialize)]
pub struct SignResponse {
    pub signature: String,
    pub public_key: String,
    pub message: String,
}

pub async fn sign_message(Json(payload): Json<SignRequest>) -> impl IntoResponse {
    // Validate required fields
    if payload.message.is_empty() || payload.secret.is_empty() {
        return (
            StatusCode::BAD_REQUEST, 
            Json(json!({
                "success": false,
                "error": "Missing required fields"
            }))
        );
    }

    // Decode the secret key from base58
    let secret_bytes = match bs58_decode(&payload.secret).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid base58 encoding for secret key"
                }))
            );
        }
    };

    // Validate secret key length (ed25519 keys should be 32 bytes)
    if secret_bytes.len() != 32 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Secret key must be exactly 32 bytes"
            }))
        );
    }

    // Create keypair from secret bytes
    let kp = match Keypair::from_bytes(&secret_bytes) {
        Ok(keypair) => keypair,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": format!("Invalid secret key: {}", e)
                }))
            );
        }
    };

    // Sign the message
    let sig: Signature = kp.sign(payload.message.as_bytes());
    let signature = base64_engine.encode(sig.to_bytes());

    (StatusCode::OK, Json(json!({
        "success": true,
        "data": SignResponse {
            signature,
            public_key: bs58_encode(kp.public.to_bytes()).into_string(),
            message: payload.message,
        }
    })))
}

#[derive(Deserialize)]
pub struct VerifyRequest {
    pub message: String,
    pub signature: String,
    pub pubkey: String,
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub valid: bool,
    pub message: String,
    pub pubkey: String,
}

pub async fn verify_message(Json(payload): Json<VerifyRequest>) -> impl IntoResponse {
    // Validate required fields
    if payload.message.is_empty() || payload.signature.is_empty() || payload.pubkey.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Missing required fields"
            }))
        );
    }

    // Decode public key from base58
    let pub_bytes = match bs58_decode(&payload.pubkey).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid base58 encoding for public key"
                }))
            );
        }
    };

    // Validate public key length (ed25519 public keys should be 32 bytes)
    if pub_bytes.len() != 32 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Public key must be exactly 32 bytes"
            }))
        );
    }

    // Create public key from bytes
    let pubkey = match PublicKey::from_bytes(&pub_bytes) {
        Ok(pk) => pk,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": format!("Invalid public key: {}", e)
                }))
            );
        }
    };

    // Decode signature from base64
    let sig_bytes = match base64_engine.decode(&payload.signature) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid base64 encoding for signature"
                }))
            );
        }
    };

    // Validate signature length (ed25519 signatures should be 64 bytes)
    if sig_bytes.len() != 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Signature must be exactly 64 bytes"
            }))
        );
    }

    // Create signature from bytes
    let signature = match Signature::from_bytes(&sig_bytes) {
        Ok(sig) => sig,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": format!("Invalid signature: {}", e)
                }))
            );
        }
    };

    // Verify the signature
    let valid = pubkey.verify(payload.message.as_bytes(), &signature).is_ok();

    (StatusCode::OK, Json(json!({
        "success": true,
        "data": VerifyResponse {
            valid,
            message: payload.message,
            pubkey: payload.pubkey,
        }
    })))
}
