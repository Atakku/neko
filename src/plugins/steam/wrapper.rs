// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use serde::Deserialize;

api!(ISteamApps, "https://api.steampowered.com/ISteamApps/", {
  fn get_app_list("GetAppList/v2") -> GetAppList;
});

#[derive(Deserialize)]
pub struct GetAppList {
  pub applist: Applist,
}

#[derive(Deserialize)]
pub struct Applist {
  pub apps: Vec<App>,
}

#[derive(Deserialize)]
pub struct App {
  #[serde(rename = "appid")]
  pub id: u32,
  pub name: String,
}

api!(ISteamUser, "https://api.steampowered.com/ISteamUser/", {
  fn get_player_summaries("GetPlayerSummaries/v2") -> Response<GetPlayerSummaries> {
    key: &String,
    steamids: &String,
  };
});

#[derive(Deserialize)]
pub struct Response<T> {
  pub response: T,
}

#[derive(Deserialize)]
pub struct GetPlayerSummaries {
  pub players: Vec<PlayerSummary>
}

#[derive(Deserialize)]
pub struct PlayerSummary {
  #[serde(rename = "steamid")]
  pub id: String,
  #[serde(rename = "personaname")]
  pub name: String,
}

api!(IPlayerService, "https://api.steampowered.com/IPlayerService/", {
  fn get_owned_games("GetOwnedGames/v1") -> Response<GetRecentlyPlayedGames> {
    key: &String,
    steamid: u64,
    include_appinfo: bool,
    include_played_free_games: bool,
  };
  fn get_steam_level("GetSteamLevel/v1") -> Response<GetSteamLevel> {
    key: String,
    steamid: u64,
  };
});

#[derive(Deserialize)]
pub struct GetRecentlyPlayedGames {
  pub games: Vec<OwnedApp>,
}

#[derive(Deserialize)]
pub struct OwnedApp {
  #[serde(rename = "appid")]
  pub id: u32,
  pub name: String,
  #[serde(rename = "playtime_forever")]
  pub playtime: u32,
}

#[derive(Deserialize)]
pub struct GetSteamLevel {
  pub player_level: u32,
}

//get_owned_games(get, "IPlayerService/GetOwnedGames", Vec<String>);
//get_friend_list(get, "ISteamUser/GetFriendList", Vec<String>);
