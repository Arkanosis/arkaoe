use actix_files::NamedFile;

use actix_web::{
    get,
    error::ErrorInternalServerError,
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

use reqwest::{
    header::ACCEPT,
    Error,
    Client,
};

use serde_derive::{
    Deserialize,
    Serialize,
};

use serde_json::Value;

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

fn make_client() -> Result<Client, Error> {
  Client::builder()
        .user_agent(format!("arkaoe/{}", crate::version()))
        .cookie_store(true)
        .build()
}

#[derive(Deserialize)]
struct ClanRequest {
    clan: String,
}

#[derive(Serialize)]
struct ClanMember {
    alias: String,
    current_rating: u64,
    highest_rating: u64,
}

async fn get_clan_members_ratings(clan_name: &str) -> Result<String, Error> {
     let client = make_client()?;

    let clan: Value = client.get(format!("https://aoe-api.reliclink.com/community/clan/getClanInfoFull?title=age2&name={}", &clan_name))
        .header(ACCEPT, mime::APPLICATION_JSON.essence_str())
        .send()
        .await?
        .json()
        .await?;

    let mut clan_members = Vec::new();

    if let Some(members_list) = clan["clan"]["members"].as_array() {

        let members_names_list: Vec<String> = members_list.iter().map(|member| {
            member["avatar"]["name"].to_string()
        })
            .collect();

        let stats: Value = client.get(format!("https://aoe-api.reliclink.com/community/leaderboard/GetPersonalStat?title=age2&profile_names=[{}]", members_names_list.join(",")))
            .header(ACCEPT, mime::APPLICATION_JSON.essence_str())
            .send()
            .await?
            .json()
            .await?;

        if let Some(members_stats_groups) = stats["statGroups"].as_array() {
            if let Some(leaderboards_stats) = stats["leaderboardStats"].as_array() {
                for member_stats_group in members_stats_groups {
                    let member = &member_stats_group["members"][0];
                    if let Some(member_id) = member["personal_statgroup_id"].as_u64() {
                        let member_alias = member["alias"].as_str().unwrap_or("?");
                        for member_stats in leaderboards_stats.iter().filter(|leaderboard_stats| {
                            leaderboard_stats["leaderboard_id"] == 3 &&
                            leaderboard_stats["statgroup_id"] == member_id
                        }) {
                           clan_members.push(ClanMember {
                                alias: member_alias.to_string(),
                                current_rating: member_stats["rating"].as_u64().unwrap_or(0),
                                highest_rating: member_stats["highestrating"].as_u64().unwrap_or(0),
                           });
                       }
                    }
                }
            }
        }
    }

    clan_members.sort_by(|first, second| {
        first.alias.cmp(&second.alias)
    });

    let mut buffer = Vec::new();
    {
        let mut csv_writer = csv::Writer::from_writer(&mut buffer);
        for clan_member in clan_members {
            csv_writer.serialize(clan_member).unwrap_or_else(|err| {
                eprintln!("{}", err);
            });
        }
        csv_writer.flush().unwrap_or_else(|err| {
            eprintln!("{}", err);
        });
    }
    Ok(String::from_utf8(buffer).unwrap_or("Error producing CSV".to_string()))
}

#[get("/clan")]
async fn serve_clan(clan_request: Query<ClanRequest>, _data: Data<AppState>) -> impl Responder {
    let clan_members_csv = get_clan_members_ratings(&clan_request.clan)
        .await;

    match clan_members_csv {
        Ok(clan_members_csv) => {
            HttpResponse::Ok()
                .content_type(ContentType(TEXT_CSV_UTF_8))
                .body(clan_members_csv)
        }
        Err(err) => {
            eprintln!("{}", err);
            HttpResponse::InternalServerError()
                .body("Internal server error (it's my fault, not yours!)")
        }
    }
}


#[derive(Deserialize)]
struct MatchesRequest {
    members: String,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct PlayerInfo {
    team: u64,
    alias: String,
    current_rating: u64,
    highest_rating: u64,
    wins: u64,
    losses: u64,
}

#[allow(non_snake_case)]
#[derive(Serialize)]
struct MatchesResponse {
    players: Vec<PlayerInfo>,
}

async fn get_current_matches(members_ids: &Vec<String>) -> Result<Json<MatchesResponse>, Error> {
    let client = make_client()?;

    let recent_matches: Value = client.get(format!("https://aoe-api.reliclink.com/community/leaderboard/getRecentMatchHistory?title=age2&profile_ids=[{}]", members_ids.join(",")))
        .header(ACCEPT, mime::APPLICATION_JSON.essence_str())
        .send()
        .await?
        .json()
        .await?;

    if let Some(matches_stats) = recent_matches["matchHistoryStats"].as_array() {
        let mut matches_stats: Vec<&Value> = matches_stats.iter().collect();
        matches_stats.sort_by(|first, second| {
            first["startgametime"].as_u64().unwrap_or(0).cmp(&second["startgametime"].as_u64().unwrap_or(0))
        });

        let mut players_list = Vec::new();
        if let Some(last_match) = matches_stats.last() {
            if let Some(players) = last_match["matchhistorymember"].as_array() {
                let profiles_ids: Vec<String> = players.iter().map(|player| {
                    player["profile_id"].as_u64().unwrap_or(0).to_string()
                })
                    .collect();
                let stats: Value = client.get(format!("https://aoe-api.reliclink.com/community/leaderboard/GetPersonalStat?title=age2&profile_ids=[{}]", profiles_ids.join(",")))
                    .header(ACCEPT, mime::APPLICATION_JSON.essence_str())
                    .send()
                    .await?
                    .json()
                    .await?;
                if let Some(profiles) = recent_matches["profiles"].as_array() {
                    if let Some(members_stats_groups) = stats["statGroups"].as_array() {
                        if let Some(leaderboards_stats) = stats["leaderboardStats"].as_array() {
                            for player in players {
                                for profile in profiles.iter().filter(|profile| {
                                    profile["profile_id"].as_u64().unwrap_or(0) == player["profile_id"].as_u64().unwrap_or(0)
                                }) {
                                    for member_stats_group in members_stats_groups.iter().filter(|member_stats_group| {
                                        let member = &member_stats_group["members"][0];
                                        if let Some(profile_id) = member["profile_id"].as_u64() {
                                            profile_id == player["profile_id"].as_u64().unwrap_or(0)
                                        } else {
                                            false
                                        }
                                    }) {
                                        let member = &member_stats_group["members"][0];
                                        if let Some(member_id) = member["personal_statgroup_id"].as_u64() {
                                            for player_stats in leaderboards_stats.iter().filter(|leaderboard_stats| {
                                                leaderboard_stats["leaderboard_id"] == 3 &&
                                                leaderboard_stats["statgroup_id"].as_u64().unwrap_or(0) == member_id
                                            }) {
                                                players_list.push(PlayerInfo {
                                                    team: player["teamid"].as_u64().unwrap_or(0),
                                                    alias: profile["alias"].as_str().unwrap_or("?").to_string(),
                                                    current_rating: player_stats["rating"].as_u64().unwrap_or(0),
                                                    highest_rating: player_stats["highestrating"].as_u64().unwrap_or(0),
                                                    wins: player["wins"].as_u64().unwrap_or(0),
                                                    losses: player["losses"].as_u64().unwrap_or(0),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        players_list.sort_by(|first, second| {
            first.team.cmp(&second.team)
                .then(second.current_rating.cmp(&first.current_rating))
        });

        return Ok(Json(MatchesResponse {
            players: players_list,
        }));
    }

    Ok(Json(MatchesResponse {
        players: Vec::new(),
    }))
}

#[get("/matches")]
async fn serve_matches(matches_request: Query<MatchesRequest>, _data: Data<AppState>) -> WebResult<Json<MatchesResponse>> {
    let members_ids: Vec<String> = matches_request.members
        .split(',')
        .map(|user| {
            format!(r#""{}""#, user)
        })
        .collect();

    let current_matches = get_current_matches(&members_ids)
        .await;

    match current_matches {
        Ok(current_matches) => {
           Ok(current_matches)
        }
        Err(err) => {
            eprintln!("{}", err);
            Err(ErrorInternalServerError("Internal server error (it's my fault, not yours!)"))
        }
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
