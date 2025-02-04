use crate::bindings::{mecab_destroy, mecab_new2, mecab_sparse_tostr};

#[derive(Debug)]
pub enum MorphemeError {
    MecabInitialization,
    MecabConvertUTF8,
    KeywordsExtraction,
}

///
/// 주어진paragraph의 NNG(일반 명사), NNP(고유 명사), ENG를 추출해 배열로 반환함.
///
pub fn analyze_morpheme(paragraph: String) -> Result<Vec<String>, MorphemeError> {
    unsafe {
        let mecab = mecab_new2(b"\0".as_ptr() as *const i8);
        if mecab.is_null() {
            return Err(MorphemeError::MecabInitialization);
        }

        let parsed_paragraph = mecab_sparse_tostr(mecab, paragraph.as_ptr() as *const i8);
        let result = match std::ffi::CStr::from_ptr(parsed_paragraph).to_str() {
            Ok(str) => str,
            Err(err) => {
                eprintln!("Failed to convert Mecab result to UTF-8: {}", err);
                return Err(MorphemeError::MecabConvertUTF8);
            }
        };

        // Extract NNG, NNP Tag with ENG , NNG -> 일반 명사, NNP -> 고유 명사
        let nng_keywords = extract_nngp_keywords(result);
        if nng_keywords.is_empty() {
            return Err(MorphemeError::KeywordsExtraction);
        }

        mecab_destroy(mecab);
        Ok(nng_keywords)
    }
}

pub fn extract_nngp_keywords(text: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    for word in text.split_whitespace() {
        let parts: Vec<&str> = word.split('/').collect();
        // Extract word
        let part = parts[0];

        if part.contains("NNG") || part.contains("NNP") {
            let part: Vec<&str> = part.split(",").collect();
            if part.len() != 1 {
                // Extract NNG or NNP word
                keywords.push(part[3].to_string());
            }
        } else if !part.is_empty() {
            let part: Vec<&str> = part.split(",").collect();

            // Extract ENG word
            if part.len() == 1 && part[0].as_bytes()[0].is_ascii_alphabetic() && part[0] != "EOS" {
                keywords.push(part[0].to_string());
            }
        }
    }
    keywords
}
