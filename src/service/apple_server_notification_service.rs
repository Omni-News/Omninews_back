use crate::{
    dto::apple_server_notification::request::AppleServerNotificationRequestDto,
    model::{apple_server_notification::DecodeAppleServerNotification, error::OmniNewsError},
    service::omninews_subscription_service::decode_jwt_data,
};

pub async fn decode_apple_signed_payload(
    signed_payload_dto: AppleServerNotificationRequestDto,
) -> Result<(), OmniNewsError> {
    let signed_payload = signed_payload_dto.signed_payload.unwrap_or_default();

    let payload_value = decode_jwt_data::<DecodeAppleServerNotification>(&signed_payload)?;

    // did_renew, expired, ... 전부 같은 형태로 나오는지 확인 필요.
    info!("payload_value: {:?}", payload_value);

    // 1. notification_type에 따라 처리 분기
    //  DID_RENEW, EXPIRED, ..
    // 1-1. DID_RENEW: 구독 갱신됨 ->  구독 갱신 처리
    // 1-2. EXPIRED: 구독 만료됨 -> 구독 만료 처리
    // 1-3. 그 외: 필요시 처리

    Ok(())
}
