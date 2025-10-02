pub enum SandboxApi {
    VerifySubscription,
}
impl SandboxApi {
    pub fn url(&self, transaction_id: &str) -> String {
        match self {
            SandboxApi::VerifySubscription => format!(
                "https://api.storekit-sandbox.itunes.apple.com/inApps/v1/subscriptions/{transaction_id}",
            ),
        }
    }
}

pub enum ProductionApi {
    VerifySubscription,
}
impl ProductionApi {
    pub fn url(&self, transaction_id: &str) -> String {
        match self {
            ProductionApi::VerifySubscription => format!(
                "https://api.storekit.itunes.apple.com/inApps/v1/subscriptions/{transaction_id}",
            ),
        }
    }
}
