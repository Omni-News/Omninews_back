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
