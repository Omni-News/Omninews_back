use bindgen;

fn main() {
    // 빌드가 변경될 때마다 다시 빌드
    println!("cargo:rerun-if-changed=build.rs");

    // mecab 라이브러리 경로 설정
    println!("cargo:rustc-link-search=native=/opt/homebrew/lib");

    // mecab 라이브러리 링크
    println!("cargo:rustc-link-lib=dylib=mecab");

    // mecab.h 파일로부터 바인딩 생성
    let bindings = bindgen::Builder::default()
        .header("/opt/homebrew/include/mecab.h")
        .clang_args(&["-w"])
        .generate()
        .expect("Unable to generate bindings");

    // 생성된 바인딩을 src/bindings.rs에 저장
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}
