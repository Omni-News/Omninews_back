#![allow(unused)]
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};
use jsonwebtoken::{decode, decode_header, encode, Algorithm, EncodingKey, Header};
use jwt_rustcrypto::decode_only;
use reqwest::{Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use sqlx::MySqlPool;

use crate::{
    dto::omninews_subscription::{
        request::OmninewsReceiptRequestDto, response::OmninewsSubscriptionResponseDto,
    },
    model::{
        appstore_api::{ProductionApi, SandboxApi},
        error::OmniNewsError,
        omninews_subscription::{DecodeLastTransaction, DecodedReceipt, NewOmniNewsSubscription},
    },
    omninews_subscription_error, omninews_subscription_info, omninews_subscription_warn,
    repository::{omninews_subscription_repository, user_repository},
    service::user_service,
};

#[derive(Debug, Clone)]
struct AppStoreConfig {
    private_key: String,
    key_id: String,
    issuer_id: String,
    bundle_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AppStoreServerApiClaims {
    iss: String, // Issuer
    iat: u64,    // Issued at
    exp: u64,    // Expiration
    aud: String, // Audience
    bid: String, // Bundle ID
}

/// 1. find user info(user_subscription_info, transaction_id) from db
/// 2. validate transaction id with App Store Server API
pub async fn verify_subscription(
    pool: &MySqlPool,
    user_email: &str,
    is_sandbox: bool,
) -> Result<OmninewsSubscriptionResponseDto, OmniNewsError> {
    // get transation_id from db
    let transaction_id = match omninews_subscription_repository::select_subscription_transaction_id(
        pool, user_email,
    )
    .await
    {
        Ok(res) => res,
        Err(_) => {
            omninews_subscription_error!(
                "[Service] Not found subscription transation_id user : {user_email}."
            );
            return Err(OmniNewsError::NotFound("not found transaction_id".into()));
        }
    };

    let url = if !is_sandbox {
        ProductionApi::VerifySubscription.url(&transaction_id)
    } else {
        SandboxApi::VerifySubscription.url(&transaction_id)
    };

    /*
        {
            "environment": "Sandbox",
            "bundleId": "com.kdh.omninews",
            "data": [
                {
                "subscriptionGroupIdentifier": "21745813",
                "lastTransactions": [
                    {
                    "originalTransactionId": "2000001024227281",
                    "status": 1,
                    "signedTransactionInfo": "eyJhbGciOi...생략...",
                    "signedRenewalInfo": "eyJhbGciOi...생략..."
                    }
                ]
                }
            ]
        }
    */
    let res = call_app_store_api(&url).await?;
    let jwt_token = res
        .get("data")
        .and_then(|v| {
            omninews_subscription_info!("1. {:?}", v.as_str());
            v.get("lastTransactions").and_then(|v| {
                omninews_subscription_info!("2. {:?}", v.as_str());
                v.get("signedTransactionInfo").and_then(|v| v.as_str())
            })
        })
        .unwrap_or_default();

    let res = decode_jwt_data::<DecodeLastTransaction>(jwt_token)?;

    let expires_date_utc =
        DateTime::from_timestamp_millis(res.expires_date.unwrap_or_default()).unwrap();
    if expires_date_utc < Utc::now() {
        omninews_subscription_error!(
            "[Service] User {} subscription expired. subscription expired at {}.",
            user_email,
            expires_date_utc
        );
        return Err(OmniNewsError::Expired("Subscription expired".into()));
    }

    Ok(OmninewsSubscriptionResponseDto {
        is_active: true,
        product_id: res.product_id.unwrap_or_default(),
        expires_date: expires_date_utc.naive_utc(),
    })
}

//TODO:
//
//
//
pub async fn register_subscription(
    pool: &MySqlPool,
    user_email: &str,
    receipt: OmninewsReceiptRequestDto,
) -> Result<bool, OmniNewsError> {
    let app_store_config = load_app_store_config()?;
    let auth_token = generate_app_store_server_jwt(&app_store_config)?;

    let payload =
        decode_jwt_data::<DecodedReceipt>(&receipt.receipt_data.clone().unwrap_or_default())?;

    // App Store Server API에서 original_transaction_id도 된다 함.
    // The identifier of a transaction that belongs to the customer, and which may be an original transaction identifier (originalTransactionId).
    let transaction_id = payload.original_transaction_id.clone().unwrap_or_default();
    let url = if !receipt.is_test.unwrap_or_default() {
        ProductionApi::VerifyTransaction.url(transaction_id.as_str())
    } else {
        SandboxApi::VerifyTransaction.url(transaction_id.as_str())
    };

    let res = call_app_store_api(&url).await.is_ok();

    omninews_subscription_info!("new_subscription payload: {payload:?}");
    if !res {
        omninews_subscription_error!("[Service] Invalid transaction ID");
        return Err(OmniNewsError::NotFound("Invalid transaction ID".into()));
    }

    let new_subscription = NewOmniNewsSubscription::new(
        receipt.receipt_data,
        payload.product_id.clone(),
        payload.original_transaction_id.clone(),
        receipt.platform,
        receipt.is_test,
        Some(
            DateTime::from_timestamp_millis(payload.signed_date.unwrap_or_default())
                .unwrap()
                .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
                .naive_utc(),
        ),
        Some(
            DateTime::from_timestamp_millis(payload.expires_date.unwrap_or_default())
                .unwrap()
                .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
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

// ---------------------- Helpers ----------------------
async fn call_app_store_api(url: &str) -> Result<Value, OmniNewsError> {
    let config = load_app_store_config()?;
    let token = generate_app_store_server_jwt(&config)?;
    let client = Client::new();
    client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            omninews_subscription_error!(
                "[Service] Failed to call App Store API. url: {}: error: {}",
                url,
                e
            );
            OmniNewsError::Request(e)
        })?
        .json::<Value>()
        .await
        .map_err(|e| {
            omninews_subscription_error!(
                "[Service] Failed to parse App Store API. url: {} response: {}",
                url,
                e
            );
            OmniNewsError::FetchUrl
        })
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

fn decode_jwt_data<T>(jwt_token: &str) -> Result<T, OmniNewsError>
where
    T: DeserializeOwned,
{
    let payload = decode_only(jwt_token)
        .map_err(|e| {
            omninews_subscription_error!("[Service] Failed to decode jwt data: {}", e);
            OmniNewsError::DecodeError
        })?
        .payload;

    serde_json::from_value::<T>(payload).map_err(|e| {
        omninews_subscription_error!("[Service] Failed to parse jwt payload: {}", e);
        OmniNewsError::DecodeError
    })
}
