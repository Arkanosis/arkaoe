use actix_files::NamedFile;

use actix_web::{
    get,
    http::header::ContentType,
    web::{
        Data,
        Json,
        Query,
    },
    App,
    HttpResponse,
    HttpServer,
    Responder,
    Result as WebResult,
};

use askama_actix::Template;

use mime::TEXT_PLAIN_UTF_8;

use serde_derive::{
    Deserialize,
    Serialize,
};

use std::{
    cmp::Reverse,
    collections::{
        BinaryHeap,
        BTreeMap,
        HashMap,
        HashSet,
    },
    fs::File,
    io::{
        BufRead,
        Cursor,
        Read,
        Seek,
        SeekFrom,
        Write,
    },
    path::Path,
    sync::Mutex,
    time::Instant,
};

enum Tag {
    Title,
    UserName,
    Other,
}

pub fn version() -> &'static str {
    if env!("CARGO_PKG_VERSION").ends_with("-dev") {
        concat!(env!("CARGO_PKG_VERSION"), "+", env!("VERGEN_GIT_SHA_SHORT"))
    } else {
        env!("CARGO_PKG_VERSION")
    }
}

struct AppState {

}

#[derive(Template)]
#[template(path = "index.htm")]
struct IndexTemplate<'a> {
    version: &'a str
}

#[get("/")]
async fn serve_index(data: Data<AppState>) -> impl Responder {
    IndexTemplate {
        version: version(),
    }
}

#[get("/clan")]
async fn serve_clan(data: Data<AppState>) -> impl Responder {
    IndexTemplate {
        version: version(),
    }
}

#[get("/matches")]
async fn serve_matches(data: Data<AppState>) -> impl Responder {
    IndexTemplate {
        version: version(),
    }
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct BadgeResponse {
    label: String,
    message: String,
    schemaVersion: u32,
}

#[get("/badge")]
async fn serve_badge(_data: Data<AppState>) -> WebResult<Json<BadgeResponse>> {
    Ok(Json(BadgeResponse {
        label: "arkaoe".to_string(),
        message: version().to_string(),
        schemaVersion: 1,
    }))
}

#[get("/logo.svg")]
async fn serve_logo(_data: Data<AppState>) -> WebResult<NamedFile> {
    Ok(NamedFile::open("static/logo.svg")?)
}


#[get("/favicon.ico")]
async fn serve_favicon(_data: Data<AppState>) -> WebResult<NamedFile> {
    Ok(NamedFile::open("static/favicon.ico")?)
}

#[derive(Deserialize)]
struct QueryRequest {

}

#[get("/version")]
async fn serve_version(data: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("Running arkaoe v{}\n", version()))
}

#[actix_web::main]
pub async fn serve(hostname: String, port: u16) -> std::io::Result<()> {
    let data = Data::new(AppState {

    });
    let initial_data = data.clone();
    println!("Listening on {}:{}...", hostname, port);
    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(serve_index)
            .service(serve_clan)
            .service(serve_matches)
            .service(serve_badge)
            .service(serve_logo)
            .service(serve_favicon)
            .service(serve_version)
    })
        .bind((hostname, port))?
        .run()
        .await
}
