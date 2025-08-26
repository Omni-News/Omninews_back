use chrono::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct NewOmniNewsSubscription {
    pub user_subscription_receipt_data: Option<String>,
    pub user_subscription_product_id: Option<String>,
    pub user_subscription_platform: Option<String>,
    pub user_subscription_plan: Option<bool>,
    pub user_subscription_is_test: Option<bool>,
    pub user_subscription_start_date: Option<NaiveDateTime>,
    pub user_subscription_end_date: Option<NaiveDateTime>,
    pub user_subscription_auto_renew: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct Receipt {
    pub receipt_data: Option<String>,
    pub platform: Option<String>,
}

/*
    {
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
#[derive(Debug, Clone)]
pub struct DecodedReceipt {
    pub purchase_date: NaiveDateTime,
    pub transaction_id: String,
    pub original_transaction_id: String,
    pub auto_renew: bool,
    pub expires_date: NaiveDateTime,
    pub product_id: String,
}
