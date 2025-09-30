#![allow(unused)]
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, NaiveDateTime, Utc};
use jsonwebtoken::{decode, decode_header, encode, Algorithm, EncodingKey, Header};
use jwt_rustcrypto::decode_only;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

use crate::{
    dto::omninews_subscription::{
        request::OmninewsReceiptRequestDto, response::OmninewsSubscriptionResponseDto,
    },
    model::{
        error::OmniNewsError,
        omninews_subscription::{DecodedReceipt, NewOmniNewsSubscription},
    },
    omninews_subscription_error, omninews_subscription_info, omninews_subscription_warn,
    repository::omninews_subscription_repository,
};

#[derive(Debug, Clone)]
struct AppStoreConfig {
    private_key: String,
    key_id: String,
    issuer_id: String,
    bundle_id: String,
}

fn load_app_store_config() -> Result<AppStoreConfig, OmniNewsError> {
    omninews_subscription_info!("App Store Server API 설정 로드 중...");
    // Load the App Store configuration from environment variables or a config file
    let private_key = env::var("APPLE_PRIVATE_KEY")
        .map_err(|_| OmniNewsError::Config("APP_STORE_PRIVATE_KEY not set".into()))?;
    let key_id = env::var("APPLE_KEY_ID")
        .map_err(|_| OmniNewsError::Config("APP_STORE_KEY_ID not set".into()))?;
    let issuer_id = env::var("APPLE_ISSUER_ID")
        .map_err(|_| OmniNewsError::Config("APP_STORE_ISSUER_ID not set".into()))?;
    let bundle_id = env::var("APPLE_BUNDLE_ID")
        .map_err(|_| OmniNewsError::Config("APP_STORE_BUNDLE_ID not set".into()))?;

    omninews_subscription_info!("App Store Server API 설정 로드 완료");
    Ok(AppStoreConfig {
        private_key,
        key_id,
        issuer_id,
        bundle_id,
    })
}

#[derive(Debug, Serialize, Deserialize)]
struct AppStoreServerApiClaims {
    iss: String, // Issuer
    iat: u64,    // Issued at
    exp: u64,    // Expiration
    aud: String, // Audience
    bid: String, // Bundle ID
}

#[derive(Debug)]
pub struct SubscriptionData {
    pub product_id: String,
    pub original_transaction_id: String,
    pub transaction_id: String,
    pub expires_date: i64,
    pub is_active: bool,
}

fn generate_app_store_server_jwt(config: &AppStoreConfig) -> Result<String, OmniNewsError> {
    // set jwt header
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(config.key_id.clone());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    // set payload
    let claims = AppStoreServerApiClaims {
        iss: config.issuer_id.clone(),
        iat: now,
        exp: now + 20 * 60, // 20 minutes expiration
        aud: "appstoreconnect-v1".to_string(),
        bid: config.bundle_id.clone(),
    };
    omninews_subscription_info!("Claims 생성: {:?}", claims);

    let encoding_key = EncodingKey::from_ec_pem(config.private_key.as_bytes()).map_err(|e| {
        omninews_subscription_error!("Private key 인코딩 오류: {}", e);
        OmniNewsError::Config("Invalid private key".into())
    })?;

    let token = encode(&header, &claims, &encoding_key).map_err(|e| {
        omninews_subscription_error!("JWT 생성 오류: {}", e);
        OmniNewsError::TokenCreateError
    })?;

    omninews_subscription_info!("App Store Server JWT 생성 성공: {}", token);

    Ok(token)
}

fn decode_recipt_data(receipt_data: &str) -> Result<DecodedReceipt, OmniNewsError> {
    let payload = decode_only(receipt_data)
        .map_err(|e| {
            omninews_subscription_error!("[Service] Failed to decode receipt data: {}", e);
            OmniNewsError::DecodeError
        })?
        .payload;
    Ok(DecodedReceipt::new(&payload))
}

pub async fn verify_subscription(
    pool: &MySqlPool,
    user_email: &str,
) -> Result<OmninewsSubscriptionResponseDto, OmniNewsError> {
    match omninews_subscription_repository::verify_subscription(pool, user_email).await {
        Ok(res) => Ok(OmninewsSubscriptionResponseDto::from_model(res)),
        Err(_) => {
            omninews_subscription_warn!(
                "사용자 {}는 구독 중이지 않거나 기간이 만료됐습니다.",
                user_email
            );
            Err(OmniNewsError::NotFound(
                "Subscription info not found".into(),
            ))
        }
    }
}

pub async fn register_subscription(
    pool: &MySqlPool,
    user_email: &str,
    receipt: OmninewsReceiptRequestDto,
) -> Result<bool, OmniNewsError> {
    let app_store_config = load_app_store_config()?;
    let auth_token = generate_app_store_server_jwt(&app_store_config)?;

    let payload = decode_recipt_data(&receipt.receipt_data.clone().unwrap_or_default())?;

    // App Store Server API에서 original_transaction_id도 된다 함.
    // The identifier of a transaction that belongs to the customer, and which may be an original transaction identifier (originalTransactionId).
    let validate_recipt = validate_transaction_id(
        &auth_token,
        &payload.original_transaction_id.clone().unwrap_or_default(),
        receipt.is_test.unwrap_or_default(),
    )
    .await;

    if !validate_recipt {
        omninews_subscription_error!("[Service] Invalid transaction ID");
        return Err(OmniNewsError::NotFound("Invalid transaction ID".into()));
    }

    let new_subscription = NewOmniNewsSubscription::new(
        receipt.platform.clone(),
        payload.product_id.clone(),
        payload.original_transaction_id.clone(),
        receipt.platform,
        receipt.is_test,
        Some(
            DateTime::from_timestamp(payload.signed_date.unwrap_or_default(), 0)
                .unwrap()
                .naive_utc(),
        ),
        Some(
            DateTime::from_timestamp(payload.expires_date.unwrap_or_default(), 0)
                .unwrap()
                .naive_utc(),
        ),
        Some(payload.type_.unwrap_or_default() == "Auto-Renewable Subscription"),
    );

    match omninews_subscription_repository::register_subscription(
        pool,
        user_email,
        new_subscription,
    )
    .await
    {
        Ok(response) => Ok(response),
        Err(e) => {
            omninews_subscription_error!(
                "Failed to register subscription for user {}: {}",
                user_email,
                e
            );
            Err(OmniNewsError::Database(e))
        }
    }
}

async fn validate_transaction_id(auth_token: &str, transaction_id: &str, is_sandbox: bool) -> bool {
    // 0 -> product, 1 -> sandbox
    let product_url = format!(
        "https://api.storekit.itunes.apple.com/inApps/v1/transactions/{}",
        transaction_id
    );

    let sandbox_url = format!(
        "https://api.storekit-sandbox.itunes.apple.com/inApps/v1/transactions/{}",
        transaction_id
    );
    let client = Client::new();

    let data = if !is_sandbox {
        client
            .get(product_url)
            .bearer_auth(auth_token)
            .send()
            .await
            .map_err(|e| {
                omninews_subscription_error!("[Service] Failed to fetch transaction info: {}", e);
                OmniNewsError::Request(e)
            })
            .unwrap()
    } else {
        omninews_subscription_info!("[Service] Test environment");
        client
            .get(sandbox_url)
            .bearer_auth(auth_token)
            .send()
            .await
            .map_err(|e| {
                omninews_subscription_error!("[Service] Failed to fetch transaction info: {}", e);
                OmniNewsError::Request(e)
            })
            .unwrap()
    };

    if data.status() != StatusCode::OK {
        omninews_subscription_error!(
            "[Service] Failed to fetch transaction info: HTTP {}",
            data.status()
        );
        return false;
    }

    true
}

// TODO:  receipt갖고 애플, 구글에 정상 영수증인지 검증
pub async fn validate_receipt(
    user_email: &str,
    receipt: OmninewsReceiptRequestDto,
) -> Result<bool, OmniNewsError> {
    let platform = &receipt.clone().platform.unwrap_or_default();
    if platform == "ios" {
        return validate_apple_receipt(user_email, &receipt).await;
    } else if platform == "android" {
        return validate_google_receipt(user_email, &receipt).await;
    }
    omninews_subscription_error!("Unsupported platform: {}", &platform);
    Err(OmniNewsError::NotFound("Unsupported platform".into()))
}

async fn validate_apple_receipt(
    user_email: &str,
    receipt: &OmninewsReceiptRequestDto,
) -> Result<bool, OmniNewsError> {
    Ok(true)
}

async fn validate_google_receipt(
    user_email: &str,
    receipt: &OmninewsReceiptRequestDto,
) -> Result<bool, OmniNewsError> {
    Ok(true)
}
