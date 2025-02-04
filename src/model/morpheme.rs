use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use super::rss::NewticleType;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NewMorpheme {
    pub morpheme_word: Option<String>,
    pub morpheme_rank: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Morpheme {
    pub morpheme_id: Option<i32>,
    pub morpheme_word: Option<String>,
    pub morpheme_rank: Option<i32>,
}

#[derive(Debug, Clone, FromRow)]
pub struct MorphemeLinkMapping {
    pub morpheme_id: Option<i32>,
    pub channel_id: Option<i32>,
    pub rss_id: Option<i32>,
    pub source_link: Option<String>,
    pub source_rank: Option<i32>,
}

impl MorphemeLinkMapping {
    pub fn new(
        newticle_type: NewticleType,
        newticle_id: Option<i32>,
        morpheme_id: Option<i32>,
        source_link: Option<String>,
        source_rank: Option<i32>,
    ) -> Self {
        let (channel_id, rss_id) = match newticle_type {
            NewticleType::Channel => (newticle_id, Some(0)),
            NewticleType::Rss => (Some(0), newticle_id),
        };

        MorphemeLinkMapping {
            morpheme_id,
            channel_id,
            rss_id,
            source_link,
            source_rank,
        }
    }
}
