#![allow(unused)]
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use reqwest::Client;
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
    let mut header = Header::new(Algorithm::ES256);
    header.kid = Some(config.key_id.clone());

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

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

fn extract_transaction_id(
    provided_transaction_id: Option<String>,
) -> Result<String, OmniNewsError> {
    if let Some(transaction_id) = provided_transaction_id {
        if !transaction_id.is_empty() {
            return Ok(transaction_id);
        }
    }
    omninews_subscription_error!("트랜잭션 ID가 제공되지 않았습니다.");
    Err(OmniNewsError::NotFound("Not FOund Transaction ID".into()))
}

async fn fetch_subscription_status(
    config: &AppStoreConfig,
    transaction_id: &str,
    is_sandbox: bool,
) -> Result<SubscriptionData, OmniNewsError> {
    let token = generate_app_store_server_jwt(config)?;

    let base_url = if is_sandbox {
        "https://api.storekit-sandbox.itunes.apple.com/inApps"
    } else {
        "https://api.storekit.itunes.apple.com/inApps"
    };

    let client = Client::new();

    let url = format!("{base_url}/v1/subscriptions/{transaction_id}");

    omninews_subscription_info!("App Store 구독 상태 조회 URL: {}", url);

    let response = client
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .map_err(|e| {
            omninews_subscription_error!("App Store 구독 상태 API 호출 오류: {}", e);
            OmniNewsError::Request(e)
        })?;
    omninews_subscription_info!("App Store 구독 상태 응답 코드: {}", response.status());

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        omninews_subscription_error!("App Store 구독 상태 조회 실패: {} - {}", status, error_text);
        return Err(OmniNewsError::FetchUrl);
    }

    let data: serde_json::Value = response.json().await.map_err(|e| {
        omninews_subscription_error!("App Store 구독 상태 응답 파싱 오류: {}", e);
        OmniNewsError::JsonParseError("Failed to parse response".into())
    })?;

    omninews_subscription_info!("App Store 구독 상태 응답: {:?}", data);

    parse_subscription_data(&data, transaction_id)
}

fn parse_subscription_data(
    data: &serde_json::Value,
    transaction_id: &str,
) -> Result<SubscriptionData, OmniNewsError> {
    // 데이터 객체 추출
    let data_obj = data.get("data").ok_or_else(|| {
        omninews_subscription_error!("App Store API 응답에 'data' 필드가 없습니다");
        OmniNewsError::JsonParseError("Missing 'data' field".into())
    })?;

    // 최신 구독 상태 가져오기
    let status = data_obj
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("EXPIRED");

    // 만료 시간 추출
    let expires_date_ms = data_obj
        .get("expiresDate")
        .and_then(|d| d.as_i64())
        .unwrap_or_else(|| {
            // 현재 시간으로 설정 (이미 만료된 것으로 처리)
            Utc::now().timestamp_millis()
        });

    // 밀리초를 초로 변환
    let expires_date = expires_date_ms / 1000;

    // 제품 ID 추출
    let product_id = data_obj
        .get("productId")
        .and_then(|p| p.as_str())
        .unwrap_or("unknown")
        .to_string();

    // 오리지널 트랜잭션 ID 추출
    let original_transaction_id = data_obj
        .get("originalTransactionId")
        .and_then(|t| t.as_str())
        .unwrap_or(transaction_id)
        .to_string();

    // 현재 시간 (초)
    let now = Utc::now().timestamp();

    // 구독이 활성화되어 있는지 확인
    let is_active = expires_date > now
        && (status == "ACTIVE" || status == "BILLING_RETRY" || status == "GRACE_PERIOD");

    Ok(SubscriptionData {
        product_id,
        original_transaction_id,
        transaction_id: transaction_id.to_string(),
        expires_date,
        is_active,
    })
}

fn decode_recipt_data(recipt_data: &str) -> Result<DecodedReceipt, OmniNewsError> {
    // TODO JWT 형식으로 아래와 같은 데이터 뽑을 수 있음.
    /*
    *{
        "subscriptionGroupIdentifier": "21745813",
        "purchaseDate": 1754226201786,
        "transactionId": "2",
        "storefrontId": "143466",
        "type": "Auto-Renewable Subscription",
        "webOrderLineItemId": "2",
        "deviceVerification": "7Vrh5rn4gyDMnogYFs/hB/TaT25aBxN/WVcuEA7aILrSjD/SXvICoHK1u+45Rrii",
        "price": 2200000,
        "expiresDate": 1754226231786,
        "originalPurchaseDate": 1754226141775,
        "transactionReason": "RENEWAL",
        "inAppOwnershipType": "PURCHASED",
        "originalTransactionId": "0",
        "isUpgraded": false,
        "productId": "kdh.omninews.premium",
        "bundleId": "com.kdh.omninews",
        "currency": "KRW",
        "signedDate": 1754226202571,
        "environment": "Xcode",
        "appTransactionId": "0",
        "deviceVerificationNonce": "e1391c79-d181-4c8f-b5a8-1466761a7abb",
        "quantity": 1,
        "storefront": "KOR"
    }
    */
    Ok(DecodedReceipt {
        purchase_date: Utc::now().naive_utc(), // TODO: 실제 값으로 변경
        transaction_id: "dummy_transaction_id".to_string(), // TODO: 실제 값으로 변경
        original_transaction_id: "dummy_original_transaction_id".to_string(), // TODO: 실제 값으로
        auto_renew: true,                      // TODO: 실제 값으로 변경
        // 30 days after
        expires_date: Utc::now()
            .naive_utc()
            .checked_add_signed(chrono::Duration::days(30))
            .unwrap(), // TODO: 실제 값으로 변경
        product_id: "dummy_product_id".to_string(), // TODO: 실제 값으로 변경
    })
}

pub async fn verify_subscription(
    pool: &MySqlPool,
    user_email: &str,
) -> Result<OmninewsSubscriptionResponseDto, OmniNewsError> {
    match omninews_subscription_repository::verify_subscription(pool, user_email).await {
        Ok(res) => Ok(OmninewsSubscriptionResponseDto::from_model(res)),
        Err(_) => {
            omninews_subscription_warn!("사용자 {}는 구독 중이지 않습니다.", user_email);
            Err(OmniNewsError::NotFound("User not found".into()))
        }
    }
}

pub async fn register_subscription(
    pool: &MySqlPool,
    user_email: &str,
    receipt: OmninewsReceiptRequestDto,
) -> Result<bool, OmniNewsError> {
    let decode_receipt = decode_recipt_data(&receipt.receipt_data.clone().unwrap_or_default())?;
    let new_subscription = NewOmniNewsSubscription {
        user_subscription_receipt_data: Some(receipt.receipt_data.clone().unwrap_or_default()),
        user_subscription_product_id: Some(decode_receipt.product_id),
        user_subscription_platform: receipt.platform.clone(),
        user_subscription_plan: Some(true),
        user_subscription_auto_renew: Some(decode_receipt.auto_renew),
        user_subscription_is_test: receipt.is_test,
        user_subscription_start_date: Some(decode_receipt.purchase_date),
        user_subscription_end_date: Some(decode_receipt.expires_date),
    };

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

// TODO receipt갖고 애플, 구글에 정상 영수증인지 검증
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
    omninews_subscription_info!(
        "IOS 구독 영수증 검증 시작: 사용자={}, 테스트모드={}",
        user_email,
        receipt.is_test.unwrap_or_default()
    );
    if let Some(res) = receipt.is_test {
        if res {
            omninews_subscription_info!("[Servie] 테스트 모드로 영수증 검증을 건너뜁니다.");
            return Ok(true);
        }
    }

    // App Store 설정 로드
    let config = load_app_store_config()?;

    let decode_receipt = decode_recipt_data(&receipt.receipt_data.clone().unwrap_or_default())?;

    // App Store Server API 호출 및 구독 상태 조회
    let subscription_data = fetch_subscription_status(
        &config,
        &decode_receipt.original_transaction_id,
        receipt.is_test.unwrap_or(false),
    )
    .await?;

    omninews_subscription_info!(
        "IOS 구독 영수증 검증 완료: 사용자={}, 트랜잭션 ID={}, 상태={}",
        user_email,
        decode_receipt.original_transaction_id,
        subscription_data.is_active
    );
    Ok(subscription_data.is_active)
}

async fn validate_google_receipt(
    user_email: &str,
    receipt: &OmninewsReceiptRequestDto,
) -> Result<bool, OmniNewsError> {
    Ok(true)
}
