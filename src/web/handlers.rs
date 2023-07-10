use ntex::web;
use serde::Serialize;

#[derive(Serialize)]
struct Version {
    version: String,
}

#[web::get("/version")]
pub(crate) async fn version() -> impl web::Responder {
    web::HttpResponse::Ok().json(&Version {
        version: env!("CARGO_PKG_VERSION_MAJOR").to_string(),
    })
}
