use std::fs;

use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};

fn main() {
    let doc = gen_my_openapi();
    fs::write("./what.html", doc);
}

fn gen_my_openapi() -> String {
    #[derive(OpenApi)]
    #[openapi()]
    struct ApiDoc;

    ApiDoc::openapi().to_pretty_json().unwrap()
}
