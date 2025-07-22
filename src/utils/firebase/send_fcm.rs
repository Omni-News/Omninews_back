use std::error::Error;

use reqwest::Client;
use serde_json::json;

use super::firebase_util::get_fcm_access_token_with_expiry;

pub async fn send_fcm_message(
    device_token: &str,
    title: &str,
    body: &str,
) -> Result<(), Box<dyn Error>> {
    let access_token = get_fcm_access_token_with_expiry().await?;

    let payload = json!({
        "message": {
            "token": device_token,
            "notification": {
                "title": title,
                "body": body,
            },
        }
    });

    let url = "https://fcm.googleapis.com/v1/projects/kdh-omninews/messages:send";
    let client = Client::new();
    let resp = client
        .post(url)
        .bearer_auth(&access_token.0)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;

    let status = resp.status();
    let text = resp.text().await?;
    info!("FCM 전송 응답: {status} {text}");
    if !status.is_success() {
        return Err(format!("FCM 전송 실패: {status} {text}").into());
    }
    Ok(())
}
