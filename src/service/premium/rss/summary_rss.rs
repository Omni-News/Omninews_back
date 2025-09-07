use scraper::{Html, Selector};
use sqlx::MySqlPool;

use crate::{
    dto::premium::rss::{request::RssSummaryRequestDto, response::RssSummaryResponseDto},
    model::error::OmniNewsError,
    utils::gemini::gemini_summarize,
};

pub async fn summary(
    pool: &MySqlPool,
    item_link_data: RssSummaryRequestDto,
) -> Result<RssSummaryResponseDto, OmniNewsError> {
    let item_link = item_link_data.rss_link;

    let body = {
        let client = reqwest::Client::new();
        let res = client.get(item_link).send().await?.text().await?;
        let doc = Html::parse_document(&res);
        let sel = Selector::parse("body").map_err(|_| OmniNewsError::FetchUrl)?;

        match doc.select(&sel).next() {
            Some(body_el) => body_el.text().collect::<String>(),
            None => {
                error!("Failed to select body element");
                return Err(OmniNewsError::FetchUrl);
            }
        }
    };

    let summarized_body = gemini_summarize(60, &body).await;
    Ok(RssSummaryResponseDto {
        text: summarized_body,
    })
    // TODO: 한번 요약한거 db저장해서 글당 한번만 ㄱㄴ하도록 하기.
    // 요약은 잘함 ㅇㅇ.
}
