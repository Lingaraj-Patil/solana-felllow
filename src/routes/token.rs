use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use spl_token::instruction as token_instruction;
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine;
use serde_json::json;
use axum::http::StatusCode;
use ed25519_dalek::Keypair;
use bs58;
use solana_sdk::signature::read_keypair_file;

#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub mintAuthority: String,
    pub mint: String,
    pub decimals: u8,
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub program_id: String,
    pub accounts: Vec<serde_json::Value>,
    pub instruction_data: String,
}

pub async fn create_token(Json(payload): Json<CreateTokenRequest>) -> impl IntoResponse {
    // Parse mint public key
    let mint_pubkey = match payload.mint.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `mint` address"
                })),
            );
        }
    };

    // Parse authority public key
    let authority = match payload.mintAuthority.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `mintAuthority` address"
                })),
            );
        }
    };

    // Create initialize mint instruction
    let ix: Instruction = match token_instruction::initialize_mint(
        &spl_token::id(),
        &mint_pubkey,
        &authority,
        None,
        payload.decimals,
    ) {
        Ok(instruction) => instruction,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Failed to create initialize mint instruction: {}", e)
                })),
            );
        }
    };

    let data = base64_engine.encode(&ix.data);
    let accounts = ix.accounts.iter().map(|acct| serde_json::json!({
        "pubkey": acct.pubkey.to_string(),
        "is_signer": acct.is_signer,
        "is_writable": acct.is_writable
    })).collect::<Vec<_>>();

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "data": TokenResponse {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data: data,
        }
    })))
}

#[derive(Deserialize)]
pub struct MintTokenRequest {
    pub mint: String,
    pub destination: String,
    pub authority: String,
    pub amount: u64,
}

pub async fn mint_token(Json(payload): Json<MintTokenRequest>) -> impl IntoResponse {
    // Parse mint public key
    let mint = match payload.mint.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `mint` address"
                })),
            );
        }
    };

    // Parse destination public key
    let dest = match payload.destination.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `destination` address"
                })),
            );
        }
    };

    // Parse authority public key
    let auth = match payload.authority.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `authority` address"
                })),
            );
        }
    };

    // Create mint to instruction
    let ix = match token_instruction::mint_to(
        &spl_token::id(), 
        &mint, 
        &dest, 
        &auth, 
        &[], 
        payload.amount
    ) {
        Ok(instruction) => instruction,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Failed to create mint_to instruction: {}", e)
                })),
            );
        }
    };

    let data = base64_engine.encode(&ix.data);
    let accounts = ix.accounts.iter().map(|acct| serde_json::json!({
        "pubkey": acct.pubkey.to_string(),
        "is_signer": acct.is_signer,
        "is_writable": acct.is_writable
    })).collect::<Vec<_>>();

    (StatusCode::OK, Json(serde_json::json!({
        "success": true,
        "data": TokenResponse {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data: data,
        }
    })))
}