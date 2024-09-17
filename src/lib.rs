use std::collections::HashMap;

use anyhow::Context;
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::{
    http::{Params, Router},
    http_component,
    sqlite::Connection,
    variables,
};

// TODO make this a Spin application variable
const NUMBER_OF_DRAFTS: u32 = 3;
const SQLITE_DATE_FMT: &str = "%Y-%m-%d";
const DRAFT_DEADLINE_DATE_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

#[derive(Serialize, Debug, Clone)]
struct Player {
    id: u32,
    name: String,
    voted_out: bool,
}

#[derive(Serialize, Debug)]
struct Drafter {
    name: String,
    season: i32,
    players: Vec<Player>,
}

#[derive(Deserialize)]
struct DraftRequest {
    drafter: String,
    players: Vec<String>,
}

#[http_component]
async fn handle_survivor_draft(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::new();
    router.get("/api/drafters", get_drafters);
    router.get("/api/players", get_players);
    router.post("/api/join", join_draft);
    router.post("/api/vote-out", vote_out);

    Ok(router.handle(req))
}

// /vote-out/
pub fn vote_out(req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let player = String::from_utf8(req.body().into())?;
    let season = current_season()?;
    let conn = Connection::open_default()?;
    let date = Utc::now().naive_utc().date();
    let query = format!(
        "UPDATE players SET voted_out = '{date}' WHERE name = '{player}' AND season = {season};"
    );
    conn.execute(&query, &[])?;
    Ok(Response::new(
        200,
        format!("Player {} voted out", player).to_string(),
    ))
}

fn current_season() -> anyhow::Result<i32> {
    let season = variables::get("season").context("could not get season")?;
    season
        .parse()
        .map_err(|_| anyhow::anyhow!("Configured season cannot be parsed as i32"))
}

fn open_draft() -> anyhow::Result<bool> {
    let deadline = variables::get("draft_deadline").context("could not get draft_deadline")?;
    parse_date(Some(&deadline), DRAFT_DEADLINE_DATE_FORMAT)
        .map(|date| date.unwrap() > Utc::now().naive_utc().date())
}

// /players
pub fn get_players(_req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let season = current_season()?;
    let conn = Connection::open_default()?;
    let query = format!("SELECT * from players WHERE season = {season};");
    let rowset = conn.execute(&query, &[])?;

    let players: Vec<Player> = rowset
        .rows()
        .map(|row| Player {
            id: row.get::<u32>("id").unwrap(),
            name: row.get::<&str>("name").unwrap().to_owned(),
            voted_out: parse_date(row.get::<&str>("voted_out"), SQLITE_DATE_FMT)
                .expect("could not parse date")
                .is_some(),
        })
        .collect();
    let body = serde_json::to_string(&players)?;
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(body)
        .build())
}

// /drafters
pub fn get_drafters(_req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let season = current_season()?;
    let conn = Connection::open_default()?;
    let query = format!(
        "SELECT 
  d.name AS drafter_name,
  d.season AS drafter_season,
  d.id AS drafter_id,
  p.id AS player_id,
  p.name AS player_name,
  p.season AS player_season,
  p.voted_out AS player_voted_out
FROM drafters d
JOIN drafterDrafts dd ON d.id = dd.drafter_id
JOIN players p ON dd.player_id = p.id
WHERE p.season = {season};"
    );
    let rowset = conn.execute(&query, &[])?;
    let mut map: HashMap<u32, Drafter> = HashMap::new();

    rowset.rows().for_each(|row| {
        let id = row.get::<u32>("drafter_id").unwrap();
        let player = Player {
            id: row.get::<u32>("player_id").unwrap(),
            name: row.get::<&str>("player_name").unwrap().to_owned(),
            voted_out: parse_date(row.get::<&str>("player_voted_out"), SQLITE_DATE_FMT)
                .expect("could not parse date")
                .is_some(),
        };
        map.entry(id)
            .and_modify(|f| f.players.push(player.clone()))
            .or_insert(Drafter {
                name: row.get::<&str>("drafter_name").unwrap().to_owned(),
                season,
                players: vec![player],
            });
    });
    let drafters = map.into_values().collect::<Vec<Drafter>>();
    let body = serde_json::to_string(&drafters)?;
    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(body)
        .build())
}

// /join
pub fn join_draft(req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    if !open_draft()? {
        return Ok(Response::new(400, "Draft is closed".to_string()));
    }
    let draft_request = serde_json::from_slice::<DraftRequest>(req.body())?;
    let drafted = draft_request.players;
    let drafter = draft_request.drafter;
    let season = current_season()?;
    let conn = Connection::open_default()?;
    if drafted.is_empty() {
        return Ok(Response::new(400, "No drafted players".to_string()));
    }
    let query = format!("SELECT * FROM players WHERE season = {season};");
    let rowset = conn.execute(&query, &[])?;
    let players = rowset
        .rows()
        .map(|row| Player {
            name: row.get::<&str>("name").unwrap().to_owned(),
            id: row.get::<u32>("id").unwrap(),
            voted_out: parse_date(row.get::<&str>("voted_out"), SQLITE_DATE_FMT)
                .expect("could not parse date")
                .is_some(),
        })
        .collect::<Vec<Player>>();
    let players = players
        .iter()
        .filter(|p| drafted.contains(&p.name))
        .collect::<Vec<&Player>>();
    if players.len() != NUMBER_OF_DRAFTS as usize {
        return Ok(Response::new(
            400,
            format!("Must draft exactly 3 players from season {season}"),
        ));
    }
    // First remove the drafter
    // TODO ensure people are using their full names
    let query = format!("DELETE FROM drafters WHERE name = '{drafter}' AND season = {season};");
    conn.execute(&query, &[])?;

    let insert_drafter_query =
        format!("INSERT OR IGNORE INTO drafters (name, season) VALUES ('{drafter}', {season});");
    let drafter_id_query =
        format!("SELECT id FROM drafters WHERE name = '{drafter}' AND season = {season};");
    conn.execute(&insert_drafter_query, &[])?;
    let drafter_id = conn
        .execute(&drafter_id_query, &[])
        .unwrap()
        .rows()
        .next()
        .unwrap()
        .get::<u32>("id")
        .unwrap();
    for player in players {
        conn.execute(&format!("INSERT OR IGNORE INTO drafterDrafts (drafter_id, player_id) VALUES ({drafter_id}, {});", player.id), &[])?;
    }
    Ok(Response::new(
        200,
        format!("Drafter {} joined", drafter).to_string(),
    ))
}

fn parse_date(date: Option<&str>, fmt: &str) -> anyhow::Result<Option<NaiveDate>> {
    match date {
        Some(date_str) => {
            if date_str.is_empty() {
                return Ok(None);
            }
            let parsed_date = NaiveDate::parse_from_str(date_str, fmt)?;
            Ok(Some(parsed_date))
        }
        None => Ok(None),
    }
}
