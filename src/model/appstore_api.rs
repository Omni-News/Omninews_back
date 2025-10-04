pub enum SandboxApi {
    VerifyTransaction,
    VerifySubscription,
}
impl SandboxApi {
    pub fn url(&self, transaction_id: &str) -> String {
        match self {
            SandboxApi::VerifyTransaction => format!(
                "https://api.storekit-sandbox.itunes.apple.com/inApps/v1/transactions/{transaction_id}",
            ),
            SandboxApi::VerifySubscription => format!(
                "https://api.storekit-sandbox.itunes.apple.com/inApps/v1/subscriptions/{transaction_id}",
            ),
        }
    }
}

pub enum ProductionApi {
    VerifyTransaction,
    VerifySubscription,
}
impl ProductionApi {
    pub fn url(&self, transaction_id: &str) -> String {
        match self {
            ProductionApi::VerifyTransaction => format!(
                "https://api.storekit.itunes.apple.com/inApps/v1/transactions/{transaction_id}",
            ),
            ProductionApi::VerifySubscription => format!(
                "https://api.storekit.itunes.apple.com/inApps/v1/subscriptions/{transaction_id}",
            ),
        }
    }
}
