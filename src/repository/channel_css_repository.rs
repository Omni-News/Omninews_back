use sqlx::{query, MySqlPool};

use crate::{model::rss::ChannelCssElement, utils::db_util::get_db};

pub async fn insert_channel_css_element(
    pool: &MySqlPool,
    channel_css_el: ChannelCssElement,
) -> Result<i32, sqlx::Error> {
    let mut conn = get_db(pool).await?;
    let result = query!(
    "INSERT INTO rss_css_channel (channel_id, item_title_css, item_description_css, item_link_css, item_author_css, item_pub_date_css, item_image_css) VALUES (?, ?, ?, ?, ?, ?, ?);",
        channel_css_el.channel_id,
        channel_css_el.item_title_css,
        channel_css_el.item_description_css,
        channel_css_el.item_link_css,
        channel_css_el.item_author_css,
        channel_css_el.item_pub_date_css,
        channel_css_el.item_image_css,
    ).execute(&mut *conn).await;

    match result {
        Ok(res) => Ok(res.last_insert_id() as i32),
        Err(e) => Err(e),
    }
}
