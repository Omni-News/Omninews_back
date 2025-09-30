use chrono::NaiveDateTime;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct NewOmniNewsSubscription {
    pub user_subscription_receipt_data: Option<String>,
    pub user_subscription_product_id: Option<String>,
    pub user_subscription_transaction_id: Option<String>,
    pub user_subscription_platform: Option<String>,
    pub user_subscription_is_test: Option<bool>,
    pub user_subscription_start_date: Option<NaiveDateTime>,
    pub user_subscription_end_date: Option<NaiveDateTime>,
    pub user_subscription_auto_renew: Option<bool>,
}

impl NewOmniNewsSubscription {
    pub fn new(
        receipt_data: Option<String>,
        product_id: Option<String>,
        transaction_id: Option<String>,
        platform: Option<String>,
        is_test: Option<bool>,
        start_date: Option<NaiveDateTime>,
        end_date: Option<NaiveDateTime>,
        auto_renew: Option<bool>,
    ) -> Self {
        NewOmniNewsSubscription {
            user_subscription_receipt_data: receipt_data,
            user_subscription_product_id: product_id,
            user_subscription_transaction_id: transaction_id,
            user_subscription_platform: platform,
            user_subscription_is_test: is_test,
            user_subscription_start_date: start_date,
            user_subscription_end_date: end_date,
            user_subscription_auto_renew: auto_renew,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Receipt {
    pub receipt_data: Option<String>,
    pub platform: Option<String>,
}

/*
{
  "transactionId": "2000001024227281",
  "originalTransactionId": "2000001024227281",
  "webOrderLineItemId": "2000000113649718",
  "bundleId": "com.kdh.omninews",
  "productId": "kdh.omninews.premium",
  "subscriptionGroupIdentifier": "21745813",
  "purchaseDate": 1759221862000,
  "originalPurchaseDate": 1759221863000,
  "expiresDate": 1759222162000,
  "quantity": 1,
  "type": "Auto-Renewable Subscription",
  "deviceVerification": "6G/2tzK/4kMKM3tPqlsQr2jKwcQy2Q1FIF43KEccN2GEwUvAoxrwtRYRSWyyI1O0",
  "deviceVerificationNonce": "c80e524d-84f2-4e12-ab43-b2e429db76c6",
  "inAppOwnershipType": "PURCHASED",
  "signedDate": 1759221862694,
  "environment": "Sandbox",
  "transactionReason": "PURCHASE",
  "storefront": "KOR",
  "storefrontId": "143466",
  "price": 2200000,
  "currency": "KRW",
  "appTransactionId": "704895469463456075"
}
*/
#[derive(Debug, Clone)]
pub struct DecodedReceipt {
    pub transaction_id: Option<String>,
    pub original_transaction_id: Option<String>,
    pub web_order_line_item_id: Option<String>,
    pub bundle_id: Option<String>,
    pub product_id: Option<String>,
    pub subscription_group_identifier: Option<String>,
    pub purchase_date: Option<i64>,
    pub original_purchase_date: Option<i64>,
    pub expires_date: Option<i64>,
    pub quantity: Option<i32>,
    pub type_: Option<String>,
    pub device_verification: Option<String>,
    pub device_verification_nonce: Option<String>,
    pub in_app_ownership_type: Option<String>,
    pub signed_date: Option<i64>,
    pub environment: Option<String>,
    pub transaction_reason: Option<String>,
    pub storefront: Option<String>,
    pub storefront_id: Option<String>,
    pub price: Option<i32>,
    pub currency: Option<String>,
    pub app_transaction_id: Option<String>,
}

impl DecodedReceipt {
    pub fn new(payload: &Value) -> Self {
        DecodedReceipt {
            transaction_id: payload
                .get("transactionId")
                .and_then(|v| v.as_str().map(String::from)),
            original_transaction_id: payload
                .get("originalTransactionId")
                .and_then(|v| v.as_str().map(String::from)),
            web_order_line_item_id: payload
                .get("webOrderLineItemId")
                .and_then(|v| v.as_str().map(String::from)),
            bundle_id: payload
                .get("bundleId")
                .and_then(|v| v.as_str().map(String::from)),
            product_id: payload
                .get("productId")
                .and_then(|v| v.as_str().map(String::from)),
            subscription_group_identifier: payload
                .get("subscriptionGroupIdentifier")
                .and_then(|v| v.as_str().map(String::from)),
            purchase_date: payload.get("purchaseDate").and_then(|v| v.as_i64()),
            original_purchase_date: payload.get("originalPurchaseDate").and_then(|v| v.as_i64()),
            expires_date: payload.get("expiresDate").and_then(|v| v.as_i64()),
            quantity: payload
                .get("quantity")
                .and_then(|v| v.as_i64().map(|n| n as i32)),
            type_: payload
                .get("type")
                .and_then(|v| v.as_str().map(String::from)),
            device_verification: payload
                .get("deviceVerification")
                .and_then(|v| v.as_str().map(String::from)),
            device_verification_nonce: payload
                .get("deviceVerificationNonce")
                .and_then(|v| v.as_str().map(String::from)),
            in_app_ownership_type: payload
                .get("inAppOwnershipType")
                .and_then(|v| v.as_str().map(String::from)),
            signed_date: payload.get("signedDate").and_then(|v| v.as_i64()),
            environment: payload
                .get("environment")
                .and_then(|v| v.as_str().map(String::from)),
            transaction_reason: payload
                .get("transactionReason")
                .and_then(|v| v.as_str().map(String::from)),
            storefront: payload
                .get("storefront")
                .and_then(|v| v.as_str().map(String::from)),
            storefront_id: payload
                .get("storefrontId")
                .and_then(|v| v.as_str().map(String::from)),
            price: payload
                .get("price")
                .and_then(|v| v.as_i64().map(|n| n as i32)),
            currency: payload
                .get("currency")
                .and_then(|v| v.as_str().map(String::from)),
            app_transaction_id: payload
                .get("appTransactionId")
                .and_then(|v| v.as_str().map(String::from)),
        }
    }
}
