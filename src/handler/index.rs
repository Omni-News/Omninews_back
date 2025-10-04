use rocket::fs::NamedFile;
use rocket_dyn_templates::{context, Template};

#[get("/")]
pub async fn index() -> Template {
    Template::render(
        "index",
        context! {
            title: "OmniNews — 모든 뉴스를 한 곳에서",
            description: "뉴스와 RSS를 한 번에. 빠르고 간결한 뉴스 경험.",
            year: 2025,
        },
    )
}

#[get("/app-ads.txt")]
pub async fn app_ads() -> Option<NamedFile> {
    NamedFile::open("static/app-ads.txt").await.ok()
}
