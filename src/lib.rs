use spin_sdk::http::{IntoResponse, Request, Response};
use spin_sdk::{
    http::{Params, Router},
    http_component,
    key_value::Store,
};

const DRAFTERS: &str = "drafters";
const STILL_ALIVE: &str = "still_alive";
const PLAYERS: &str = "players";

#[http_component]
async fn handle_survivor_draft(req: Request) -> anyhow::Result<impl IntoResponse> {
    let mut router = Router::new();
    router.get("/api/drafters", get_drafters);
    router.get("/api/drafted/:drafter", get_drafted_for_drafter);
    router.post("/api/join/:drafter", join_draft);
    router.post("/api/leave/:drafter", leave_draft);
    router.post("/api/players", set_players);
    router.post("/api/vote-out/:player", vote_out);

    Ok(router.handle(req))
}
// /players
pub fn set_players(req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let body = req.body();
    let players = String::from_utf8(body.into())?;
    // Ensure at least one player is drafted
    if players.is_empty() {
        return Ok(Response::new(400, "No added players".to_string()));
    }
    let store = Store::open_default()?;
    // ensure only characters and commas in the string
    for c in players.chars() {
        if !c.is_alphabetic() && c != ',' {
            return Ok(Response::new(400, "Invalid player name".to_string()));
        }
    }
    store.set(PLAYERS, body)?;
    store.set(STILL_ALIVE, body)?;
    Ok(Response::new(200, "Players added".to_string()))
}

// /vote-out/:player
pub fn vote_out(_req: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    let player = params.get("player").expect("PLAYER");
    let store = Store::open_default()?;
    let still_alive = String::from_utf8(store.get(STILL_ALIVE)?.unwrap_or_default())?;
    let still_alive = still_alive
        .split(',')
        .filter(|p| p != &player)
        .map(|x| x.to_string() + ",")
        .collect::<String>();
    let store = Store::open_default()?;
    store.set(STILL_ALIVE, still_alive.as_bytes())?;
    Ok(Response::new(
        200,
        format!("Player {} voted out", player).to_string(),
    ))
}

// /drafters
pub fn get_drafters(_req: Request, _params: Params) -> anyhow::Result<impl IntoResponse> {
    let store = Store::open_default()?;
    let drafters = String::from_utf8(store.get(DRAFTERS)?.unwrap_or_default())?;
    Ok(Response::new(200, drafters))
}

// /drafted/:drafter
pub fn get_drafted_for_drafter(_req: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    let drafter = params.get("drafter").expect("DRAFTER");
    let store = Store::open_default()?;
    let drafted = store.get(drafter)?;
    match drafted {
        Some(drafted) => {
            let alive = String::from_utf8(store.get(STILL_ALIVE)?.unwrap_or_default())?;
            let alive = alive.split(',').collect::<Vec<&str>>();
            let drafted_status: String = String::from_utf8(drafted)?
                .split(',')
                .map(|d| {
                    if alive.contains(&d) {
                        format!("{}+", d)
                    } else {
                        format!("{}-", d)
                    }
                })
                .map(|s| s.to_string() + ",")
                .collect();
            Ok(Response::new(200, drafted_status))
        }
        None => Ok(Response::new(
            404,
            format!("Drafter {} not found", drafter).to_string(),
        )),
    }
}

// /join/:drafter
pub fn join_draft(req: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    let drafter = params.get("drafter").expect("DRAFTER");
    let store = Store::open_default()?;
    let body = req.body();
    let drafted = String::from_utf8(body.into())?;
    if drafted.is_empty() {
        return Ok(Response::new(400, "No drafted players".to_string()));
    }
    let players = String::from_utf8(store.get(PLAYERS)?.unwrap_or_default())?;
    let players = players.split(',').collect::<Vec<&str>>();
    let drafted = drafted.split(',').collect::<Vec<&str>>();
    for d in drafted.iter() {
        if !players.contains(d) {
            return Ok(Response::new(
                400,
                format!("Player {} not found", d).to_string(),
            ));
        }
    }
    store.set(drafter, body)?;
    let drafters = store.get(DRAFTERS)?.unwrap_or_default();
    if drafters.is_empty() {
        store.set(DRAFTERS, drafter.as_bytes())?;
    } else {
        store.set(
            DRAFTERS,
            format!("{},{}", String::from_utf8(drafters)?, drafter).as_bytes(),
        )?;
    }
    Ok(Response::new(
        200,
        format!("Drafter {} joined", drafter).to_string(),
    ))
}

// /leave/:drafter
pub fn leave_draft(_req: Request, params: Params) -> anyhow::Result<impl IntoResponse> {
    let drafter = params.get("drafter").expect("DRAFTER");
    let store = Store::open_default()?;
    if store.get(drafter)?.is_none() {
        return Ok(Response::new(
            404,
            format!("Drafter {} not found", drafter).to_string(),
        ));
    }
    store.delete(drafter)?;
    let drafters = store.get(DRAFTERS)?.unwrap_or_default();
    let drafters = String::from_utf8(drafters)?
        .split(',')
        .filter(|d| d != &drafter && !d.is_empty())
        .map(|x| x.to_string() + ",")
        .collect::<String>();
    store.set(DRAFTERS, drafters.as_bytes())?;
    Ok(Response::new(
        200,
        format!("Drafter {} removed", drafter).to_string(),
    ))
}
