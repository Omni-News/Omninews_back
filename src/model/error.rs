use thiserror::Error;

#[derive(Debug, Error)]
pub enum OmniNewsError {
    #[error("Failed to fetch : {0}")]
    Request(#[from] reqwest::Error),

    #[error("Failed to parse RSS feed")]
    Parse,

    #[error("Morpheme Error: {0}")]
    Morpheme(#[from] MorphemeError),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Failed to fetch News")]
    FetchNews,
}

#[derive(Debug, Error)]
pub enum MorphemeError {
    #[error("Failed to initialize Mecab")]
    MecabInitialization,

    #[error("Failed to convert Mecab result to UTF-8")]
    MecabConvertUTF8,

    #[error("Failed to extract key words")]
    KeywordsExtraction,
}
