use sqlx::MySqlPool;

use crate::{
    config::webdriver::DriverPool,
    dto::{premium::rss::request::RssGenerateRequestDto, rss::response::RssChannelResponseDto},
    model::{error::OmniNewsError, premium::rss_generate::SiteType},
    service::channel_service,
    utils::embedding_util::EmbeddingService,
};

use super::site::{default, instagram, medium, naver, tistory};

pub async fn generate_rss(
    pool: &MySqlPool,
    embedding_service: &EmbeddingService,
    driver_pool: &DriverPool,
    data: RssGenerateRequestDto,
) -> Result<RssChannelResponseDto, OmniNewsError> {
    //let generated_channel = generate_rss_channel(data.channel_link, data.kind).await?;
    let link = data.channel_link;

    // TODO: 이미 등록되어 있는 Rss를 또 등록하려 할 때, 문제 발생. 이거 어떻게 해결할지 방안 강구!!
    if let Ok(channel) = channel_service::find_rss_channel_by_channel_link(pool, &link).await {
        return Ok(RssChannelResponseDto::from_model(channel));
    };

    let channel_id = match data.kind {
        SiteType::Naver => naver::generate_rss(pool, embedding_service, &link).await?,
        SiteType::Tistory => tistory::generate_rss(pool, embedding_service, &link).await?,
        //SiteType::Instagram => todo!(),
        SiteType::Medium => medium::generate_rss(pool, embedding_service, &link).await?,
        SiteType::Instagram => {
            instagram::generate_rss(pool, embedding_service, driver_pool, &link).await?
        }
        SiteType::Default => {
            default::generate_rss(pool, embedding_service, driver_pool, &link).await?
        }
    };
    //    let channel_id = channel_service::create_rss_and_embedding_by_channel(
    //        pool,
    //        embedding_service,
    //        generated_channel,
    //        "omninews".into(),
    //    )
    //    .await?;

    channel_service::find_rss_channel_by_id(pool, channel_id).await
}
