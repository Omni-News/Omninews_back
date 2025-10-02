use chrono::NaiveDateTime;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct NewOmniNewsSubscription {
    pub user_id: Option<i32>,
    pub omninews_subscription_transaction_id: Option<String>,
    pub omninews_subscription_status: Option<bool>,
    pub omninews_subscription_product_id: Option<String>,
    pub omninews_subscription_auto_renew: Option<bool>,
    pub omninews_subscription_platform: Option<String>,
    pub omninews_subscription_start_date: Option<NaiveDateTime>,
    pub omninews_subscription_renew_date: Option<NaiveDateTime>,
    pub omninews_subscription_end_date: Option<NaiveDateTime>,
    pub omninews_subscription_is_sandbox: Option<bool>,
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
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedReceipt {
    pub original_transaction_id: Option<String>,
    pub product_id: Option<String>,
    pub expires_date: Option<i64>,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub signed_date: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeSignedTransactionInfo {
    pub purchase_date: Option<i64>,
    pub original_purchase_date: Option<i64>,
    pub expires_date: Option<i64>,
    pub product_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeSignedRenewalInfo {
    pub auto_renew_status: Option<bool>,
}
