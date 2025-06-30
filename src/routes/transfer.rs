use axum::{Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use solana_program::instruction::Instruction;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use spl_token::instruction as token_instruction;
use base64::engine::general_purpose::STANDARD as base64_engine;
use base64::Engine;
use solana_sdk::signature::read_keypair_file;
use axum::http::StatusCode;
use serde_json::json;

#[derive(Deserialize)]
pub struct SendSolRequest {
    pub from: String,
    pub to: String,
    pub lamports: u64,
}

#[derive(Deserialize)]
pub struct SendTokenRequest {
    pub destination: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
}

#[derive(Serialize)]
pub struct TransferResponse {
    pub program_id: String,
    pub accounts: Vec<String>,
    pub instruction_data: String,
}

pub async fn send_sol(Json(payload): Json<SendSolRequest>) -> impl IntoResponse {
    // Validate required fields
    if payload.from.is_empty() || payload.to.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Missing required fields: from and to addresses are required"
            }))
        );
    }

    // Validate lamports amount
    if payload.lamports == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Amount must be greater than 0"
            }))
        );
    }

    // Parse 'from' public key
    let from = match payload.from.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `from` address"
                }))
            );
        }
    };

    // Parse 'to' public key
    let to = match payload.to.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `to` address"
                }))
            );
        }
    };

    // Create transfer instruction
    let ix: Instruction = system_instruction::transfer(&from, &to, payload.lamports);
    let data = base64_engine.encode(&ix.data);
    let accounts = ix.accounts.iter().map(|acct| acct.pubkey.to_string()).collect();

    (StatusCode::OK, Json(json!({
        "success": true,
        "data": TransferResponse {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data: data,
        }
    })))
}

pub async fn send_token(Json(payload): Json<SendTokenRequest>) -> impl IntoResponse {
    // Validate required fields
    if payload.destination.is_empty() || payload.mint.is_empty() || payload.owner.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Missing required fields: destination, mint, and owner addresses are required"
            }))
        );
    }

    // Validate amount
    if payload.amount == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Amount must be greater than 0"
            }))
        );
    }

    // Parse destination public key
    let dest = match payload.destination.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `destination` address"
                }))
            );
        }
    };

    // Parse mint public key
    let mint = match payload.mint.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `mint` address"
                }))
            );
        }
    };

    // Parse owner public key
    let owner = match payload.owner.parse::<Pubkey>() {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "success": false,
                    "error": "Invalid `owner` address"
                }))
            );
        }
    };

    // Create token transfer instruction
    let ix = match token_instruction::transfer(
        &spl_token::id(), 
        &mint, 
        &dest, 
        &owner, 
        &[], 
        payload.amount
    ) {
        Ok(instruction) => instruction,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "success": false,
                    "error": format!("Failed to create token transfer instruction: {}", e)
                }))
            );
        }
    };

    let data = base64_engine.encode(&ix.data);
    let accounts = ix.accounts.iter().map(|acct| json!({
        "pubkey": acct.pubkey.to_string(),
        "is_signer": acct.is_signer,
        "is_writable": acct.is_writable
    })).collect::<Vec<_>>();

    (StatusCode::OK, Json(json!({
        "success": true,
        "data": {
            "program_id": ix.program_id.to_string(),
            "accounts": accounts,
            "instruction_data": data,
        }
    })))
}
