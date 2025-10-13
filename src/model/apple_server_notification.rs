use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NotificationType {
    DidRenew,
    Expired,
    // ...
}

/*
*{
  "notificationType": "DID_RENEW",
  "notificationUUID": "90420247-3bdc-4f6c-9c9c-bbbb110f892d",
  "data": {
    "appAppleId": 6746567181,
    "bundleId": "com.kdh.omninews",
    "bundleVersion": "19",
    "environment": "Sandbox",
    "signedTransactionInfo": "eyJhbG...",
    "signedRenewalInfo": "eyJhbGciOi...",
    "status": 1
  },
  "version": "2.0",
  "signedDate": 1760060422199
}
*/
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeAppleServerNotification {
    // DID_RENEW, EXPIRED, ...
    pub notification_type: Option<NotificationType>,
    pub notification_u_u_i_d: Option<String>,
    pub data: Option<DecodeAppleServerNotificationData>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodeAppleServerNotificationData {
    pub bundle_id: Option<String>,
    pub bundle_version: Option<String>,
    pub environment: Option<String>,
    pub signed_transaction_info: Option<String>,
    pub signed_renewal_info: Option<String>,
    // 1, ... 뭐있는지 확인.
    pub status: Option<i32>,
}
