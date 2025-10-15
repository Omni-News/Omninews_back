use rocket::{form::Form, fs::NamedFile, State};
use rocket_dyn_templates::{context, Template};
use sqlx::MySqlPool;

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

#[get("/ads.txt")]
pub async fn ads() -> Option<NamedFile> {
    NamedFile::open("static/app-ads.txt").await.ok()
}

#[derive(FromForm)]
pub struct DeleteAccountForm {
    email: String,
    confirm: bool,
}
#[get("/delete_account")]
pub async fn delete_account() -> Template {
    Template::render(
        "delete_account",
        context! {
            title: "Omninews - 계정 삭제",
            app_name: "Omninews",
            current_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        },
    )
}

#[post("/delete_account", data = "<form>")]
pub async fn process_delete_account(
    form: Form<DeleteAccountForm>,
    db_pool: &State<MySqlPool>,
) -> Template {
    if !form.confirm {
        return Template::render(
            "delete_account",
            context! {
                title: "Omninews - 계정 삭제",
                app_name: "Omninews",
                error: "확인란을 체크해주세요.",
                current_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
            },
        );
    }

    // TODO: 사용자가 요청하면 처리하기 직전 확인 이메일 보내고 처리하기.

    Template::render(
        "delete_account_confirmation",
        context! {
            title: "Omninews - 계정 삭제 요청 완료",
            app_name: "Omninews",
            email: form.email.clone(),
            current_time: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        },
    )
}
