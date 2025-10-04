use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::{DateTime, FixedOffset, Utc};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
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
        omninews_subscription::{
            DecodeSignedRenewalInfo, DecodeSignedTransactionInfo, NewOmniNewsSubscription,
        },
    },
    omninews_subscription_error, omninews_subscription_info, omninews_subscription_warn,
    repository::omninews_subscription_repository,
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
) -> Result<OmninewsSubscriptionResponseDto, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email.into()).await?;
    let transaction_id = get_transaction_id_from_db(pool, user_id).await?;

    let is_sandbox = is_sandbox(&transaction_id).await?;

    let (signed_transaction_info, signed_renewal_info) =
        get_subscription_transaction_info(is_sandbox, &transaction_id).await?;

    let expires_date_utc = match validate_expires_date(
        user_email,
        signed_transaction_info.expires_date.unwrap_or_default(),
    )
    .await
    {
        Ok(date) => date,
        Err(e) => {
            // update status, auto_renew as 0 to db
            match omninews_subscription_repository::expired_subscription(pool, user_id).await {
                Ok(_) => omninews_subscription_info!(
                    "[Service] Expired subscription status updated for user {}",
                    user_email
                ),
                Err(e) => omninews_subscription_error!(
                    "[Service] Failed to update expired subscription status for user {}: {}",
                    user_email,
                    e
                ),
            }
            return Err(e);
        }
    };
    // signed_transaction_info의 purchase_date는 최신 구독(갱신 포함)일임.
    let renew_date =
        DateTime::from_timestamp_millis(signed_transaction_info.purchase_date.unwrap_or_default())
            .unwrap_or_default()
            .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
            .naive_local();
    let expires_date = expires_date_utc.naive_local();
    let auto_renew = signed_renewal_info.auto_renew_status.unwrap_or_default();

    // update auto_renew, renew_date, expires_date to db
    match omninews_subscription_repository::update_verify_subscription_info(
        pool,
        user_id,
        auto_renew != 0,
        renew_date,
        expires_date,
    )
    .await
    {
        Ok(_) => omninews_subscription_info!(
            "[Service] Subscription info updated for user {}",
            user_email
        ),
        Err(e) => omninews_subscription_error!(
            "[Service] Failed to update subscription info for user {}: {}",
            user_email,
            e
        ),
    }

    Ok(OmninewsSubscriptionResponseDto {
        is_active: true,
        product_id: signed_transaction_info.product_id.unwrap_or_default(),
        expires_date,
    })
}

pub async fn register_subscription(
    pool: &MySqlPool,
    user_email: &str,
    receipt: OmninewsReceiptRequestDto,
) -> Result<bool, OmniNewsError> {
    // App Store Server API에서 original_transaction_id도 된다 함.
    // The identifier of a transaction that belongs to the customer, and which may be an original transaction identifier (originalTransactionId).
    // VerifyTransaction의 경우 최초 구독 시의 정보만 반영됨으로, 중단했다 다시 갱신할 때는 기간이
    // 다름. 이에 VerifySubscription을 통해서 정보를 가져와야 함.

    let transaction_id = receipt.transaction_id.unwrap_or_default();
    omninews_subscription_info!("transaction_id: {transaction_id}");
    let is_sandbox = is_sandbox(&transaction_id).await?;

    let (signed_transaction_info, signed_renewal_info) =
        match get_subscription_transaction_info(is_sandbox, &transaction_id).await {
            Ok(res) => res,
            Err(_) => {
                omninews_subscription_error!(
                    "[Service] Invalid transaction_id for user : {user_email}"
                );
                return Err(OmniNewsError::InvalidValue("Invalid transaction id".into()));
            }
        };

    let expires_date_utc = validate_expires_date(
        user_email,
        signed_transaction_info.expires_date.unwrap_or_default(),
    )
    .await?;

    // register subscription info to db
    let user_id = user_service::find_user_id_by_email(pool, user_email.into()).await?;
    let new_omninews_subscription = NewOmniNewsSubscription {
        user_id: Some(user_id),
        omninews_subscription_transaction_id: receipt.original_transaction_id,
        omninews_subscription_status: Some(true),
        omninews_subscription_product_id: signed_transaction_info.product_id,
        omninews_subscription_auto_renew: Some(
            signed_renewal_info.auto_renew_status.unwrap_or_default() != 0,
        ),
        omninews_subscription_platform: receipt.platform.clone(),
        omninews_subscription_start_date: Some(
            DateTime::from_timestamp_millis(
                signed_transaction_info
                    .original_purchase_date
                    .unwrap_or_default(),
            )
            .unwrap_or_default()
            .with_timezone(&FixedOffset::east_opt(9 * 3600).unwrap())
            .naive_local(),
        ),
        omninews_subscription_renew_date: None,
        omninews_subscription_end_date: Some(expires_date_utc.naive_local()),
        omninews_subscription_is_sandbox: Some(is_sandbox),
    };

    match omninews_subscription_repository::register_subscription(pool, new_omninews_subscription)
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

pub async fn verify_is_subscribed_user(
    pool: &MySqlPool,
    user_email: &str,
) -> Result<bool, OmniNewsError> {
    let user_id = user_service::find_user_id_by_email(pool, user_email.into()).await?;

    match omninews_subscription_repository::select_subscription_status_by_user_email(pool, user_id)
        .await
    {
        Ok(is_subscribed) => {
            if is_subscribed {
                Ok(true)
            } else {
                omninews_subscription_warn!(
                    "[Service] User {} is not an active subscriber.",
                    user_email
                );
                Err(OmniNewsError::NotFound("No active subscription".into()))
            }
        }

        Err(e) => {
            omninews_subscription_error!(
                "[Service] Failed to validate subscription for user {}: {}",
                user_email,
                e
            );
            Err(OmniNewsError::Database(e))
        }
    }
}

// ---------------------- Helpers ----------------------

fn load_app_store_config() -> Result<AppStoreConfig, OmniNewsError> {
    // Load the App Store configuration from environment variables or a config file
    let private_key = env::var("APPLE_PRIVATE_KEY")
        .map_err(|_| OmniNewsError::Config("APP_STORE_PRIVATE_KEY not set".into()))?;
    let key_id = env::var("APPLE_KEY_ID")
        .map_err(|_| OmniNewsError::Config("APP_STORE_KEY_ID not set".into()))?;
    let issuer_id = env::var("APPLE_ISSUER_ID")
        .map_err(|_| OmniNewsError::Config("APP_STORE_ISSUER_ID not set".into()))?;
    let bundle_id = env::var("APPLE_BUNDLE_ID")
        .map_err(|_| OmniNewsError::Config("APP_STORE_BUNDLE_ID not set".into()))?;

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

    let encoding_key = EncodingKey::from_ec_pem(config.private_key.as_bytes()).map_err(|e| {
        omninews_subscription_error!("Private key 인코딩 오류: {}", e);
        OmniNewsError::Config("Invalid private key".into())
    })?;

    let token = encode(&header, &claims, &encoding_key).map_err(|e| {
        omninews_subscription_error!("JWT 생성 오류: {}", e);
        OmniNewsError::TokenCreateError
    })?;

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

async fn get_subscription_transaction_info(
    is_sandbox: bool,
    transaction_id: &str,
) -> Result<(DecodeSignedTransactionInfo, DecodeSignedRenewalInfo), OmniNewsError> {
    let url = if !is_sandbox {
        ProductionApi::VerifySubscription.url(transaction_id)
    } else {
        SandboxApi::VerifySubscription.url(transaction_id)
    };
    let res = call_app_store_api(&url).await?;

    let last_transaction = res
        .get("data")
        .and_then(|v| v.get(0))
        .and_then(|v| v.get("lastTransactions"))
        .and_then(|v| v.get(0))
        .unwrap_or_default();

    let signed_transaction_info_jwt = last_transaction
        .get("signedTransactionInfo")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let signed_renewal_info_jwt = last_transaction
        .get("signedRenewalInfo")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let signed_transaction_info =
        decode_jwt_data::<DecodeSignedTransactionInfo>(signed_transaction_info_jwt)?;

    let signed_renewal_info = decode_jwt_data::<DecodeSignedRenewalInfo>(signed_renewal_info_jwt)?;

    Ok((signed_transaction_info, signed_renewal_info))
}

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
async fn validate_expires_date(
    user_email: &str,
    expires_date: i64,
) -> Result<DateTime<FixedOffset>, OmniNewsError> {
    let expires_date_utc = DateTime::from_timestamp_millis(expires_date)
        .unwrap_or_default()
        .with_timezone(&FixedOffset::east_opt(9 * 60 * 60).unwrap());

    let now_utc = Utc::now().with_timezone(&FixedOffset::east_opt(9 * 60 * 60).unwrap());
    if expires_date_utc < now_utc {
        omninews_subscription_error!(
            "[Service] Subscription expired for user {}: expires_date: {}, now: {}",
            user_email,
            expires_date_utc,
            now_utc
        );
        return Err(OmniNewsError::Expired("Subscription expired".into()));
    }

    Ok(expires_date_utc)
}

async fn get_transaction_id_from_db(
    pool: &sqlx::Pool<sqlx::MySql>,
    user_id: i32,
) -> Result<String, OmniNewsError> {
    let transaction_id =
        match omninews_subscription_repository::select_subscription_transaction_id(pool, user_id)
            .await
        {
            Ok(res) => res,
            Err(_) => {
                omninews_subscription_warn!(
                    "[Service] Not found subscription transation_id user : {user_id}."
                );
                return Err(OmniNewsError::NotFound("not found transaction_id".into()));
            }
        };
    Ok(transaction_id)
}

/// transaction_id가 production에 있는지 확인
/// 있으면 production, 없으면 sandbox
async fn is_sandbox(transaction_id: &str) -> Result<bool, OmniNewsError> {
    let config = load_app_store_config()?;
    let auth_token = generate_app_store_server_jwt(&config)?;

    let product_url = ProductionApi::VerifyTransaction.url(transaction_id);

    let client = Client::new();
    let is_product = {
        let res = client
            .get(product_url)
            .bearer_auth(auth_token)
            .send()
            .await
            .unwrap();

        res.status() != StatusCode::NOT_FOUND
    };

    Ok(!is_product)
}
