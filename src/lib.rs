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

use mime::TEXT_CSV_UTF_8;

use serde_derive::{
    Deserialize,
    Serialize,
};

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
async fn serve_index(_data: Data<AppState>) -> impl Responder {
    IndexTemplate {
        version: version(),
    }
}


#[derive(Deserialize)]
struct ClanRequest {
    clan: String,
}

#[derive(Serialize)]
struct ClanMember<'a> {
    name: &'a str,
    current_elo: u32,
    highest_elo: u32,
}

#[get("/clan")]
async fn serve_clan(clan_request: Query<ClanRequest>, _data: Data<AppState>) -> impl Responder {
    let mut buffer = Vec::new();

    {
        let mut csv_writer = csv::Writer::from_writer(&mut buffer);

        let rec = ClanMember {
            name: &clan_request.clan,
            current_elo: 0,
            highest_elo: 1,
        };

        csv_writer.serialize(rec).unwrap();
        csv_writer.flush().unwrap();
    }

    HttpResponse::Ok()
        .content_type(ContentType(TEXT_CSV_UTF_8))
        .body(String::from_utf8(buffer).unwrap())
}


#[derive(Deserialize)]
struct MatchesRequest {
    members: String,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct MatchesResponse {
    placeholder: String,
}

#[get("/matches")]
async fn serve_matches(matches_request: Query<MatchesRequest>, _data: Data<AppState>) -> WebResult<Json<MatchesResponse>> {
    let members: Vec<String> = matches_request.members.split(',').map(|user| user.to_string()).collect();

    Ok(Json(MatchesResponse {
        placeholder: members.join("|"),
    }))
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
async fn serve_version(_data: Data<AppState>) -> impl Responder {
    HttpResponse::Ok().body(format!("Running arkaoe v{}\n", version()))
}

#[actix_web::main]
pub async fn serve(hostname: String, port: u16) -> std::io::Result<()> {
    let data = Data::new(AppState {

    });
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
