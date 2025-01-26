use crate::bindings::{mecab_destroy, mecab_new2, mecab_sparse_tostr};

pub fn mecab_test() {
    unsafe {
        let mecab = mecab_new2(b"\0".as_ptr() as *const i8);
        if mecab.is_null() {
            println!("Failed to initialize mecab.");
            return;
        }

        let input = r#"실시간 반응형 추천 개발 일지 #1. 프로젝트 소개" 편을 읽고 2편을 기다려주신 여러분, 반갑습니다. (아직 안 읽으셨다면 읽고 오셔도 됩니다. 여기서 기다리고 있을 테니까요.) 오늘은 1편에 이어 실시간 행동 이력을 활용한 실시간 반응형 추천 시스템 의 개발에 대해 더 깊이 들어가 보도록 하겠습니다. 제 소개를 드려야겠네요. 저는 우아한형제들 추천프로덕트팀에서 AI/ML, Data Engineer를 하고 있는 정현입니다."#;
        let result = mecab_sparse_tostr(mecab, input.as_ptr() as *const i8);
        let result_str = std::ffi::CStr::from_ptr(result).to_str().unwrap();

        // Extract NNG, NNP Tag, NNG -> 일반 명사, NNP -> 고유 명사
        let nng_keywords = extract_nngp_keywords(result_str);

        //println!("NNG/P Keywords : {:?}", nng_keywords);

        mecab_destroy(mecab);
    }
}

pub fn extract_nngp_keywords(text: &str) -> Vec<String> {
    let mut keywords = Vec::new();

    for word in text.split_whitespace() {
        let parts: Vec<&str> = word.split('/').collect();
        if parts.len() != 1 {
            continue;
        }
        let part = parts[0];

        if part.contains("NNG") || part.contains("NNP") {
            let part: Vec<&str> = part.split(",").collect();
            keywords.push(part[3].to_string());
        } else {
            let part: Vec<&str> = part.split(",").collect();

            if part.len() == 1 && part[0].as_bytes()[0].is_ascii_alphabetic() && part[0] != "EOS" {
                keywords.push(part[0].to_string());
            }
        }
    }
    keywords
}
