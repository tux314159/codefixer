pub mod auth;
pub mod problems;

use axum::Json;
use utoipa::{OpenApi, openapi};

#[derive(OpenApi)]
#[openapi(
    paths(
        openapi,
        problems::get::problems,
        problems::get::problems_id,
        auth::login::get::login,
        auth::login::post::logout,
        auth::oauth::get::authenticate,
    ),
    components(schemas(problems::get::ProblemSort))
)]
struct ApiDoc;

pub const API_DOC_URI: &str = "/api-dics/openapi.json";

#[utoipa::path(
    get,
    path = API_DOC_URI,
    responses(
        (status = 200, description = "JSON file", body = ())
    )
)]
pub async fn openapi() -> Json<openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
