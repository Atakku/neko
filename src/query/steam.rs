// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::collections::HashMap;

use crate::{
  core::*,
  interface::steam::{IPlayerService, ISteamApps, ISteamUser},
  modules::{reqwest::req, steam::sapi_key},
  schema::steam::{Apps, Playdata, PlaydataHistory, Users},
};
use chrono::Utc;
use sea_query::{OnConflict, Query};

pub async fn update_apps() -> R {
  log::info!("Updating Steam apps");
  let apps = req().get_app_list().await?.applist.apps;
  for apps_chunk in apps.chunks(10000) {
    let mut qb = Query::insert();
    qb.into_table(Apps::Table);
    qb.columns([Apps::Id, Apps::Name]);
    qb.on_conflict(
      OnConflict::column(Apps::Id)
        .update_column(Apps::Name)
        .to_owned(),
    );
    for app in apps_chunk {
      qb.values([(app.id as i32).into(), app.name.clone().into()])?;
    }
    execute!(&qb)?;
    log::info!("Updated {} apps", apps_chunk.len());
  }
  log::info!("Finished updating Steam apps");
  Ok(())
}

pub async fn update_users(user_list: &Vec<(i64,)>) -> R {
  log::info!("Updating Steam users");
  let mut profiles = vec![];
  for chunk in user_list.chunks(100) {
    match req()
      .get_player_summaries(
        sapi_key(),
        &chunk
          .into_iter()
          .map(|i| i.0.to_string())
          .collect::<Vec<String>>()
          .join(","),
      )
      .await
    {
      Ok(res) => {
        for user in res.response.players {
          profiles.push((user.id.parse::<i64>()?, user.name));
        }
      }
      Err(err) => log::warn!("Failed to get '{}' profile summaries: {err}", chunk.len()),
    }
  }
  for chunk in profiles.chunks(10000) {
    let mut qb = Query::insert();
    qb.into_table(Users::Table);
    qb.columns([Users::Id, Users::Name]);
    qb.on_conflict(
      OnConflict::column(Users::Id)
        .update_column(Users::Name)
        .to_owned(),
    );
    for v in chunk {
      qb.values([v.0.into(), v.1.clone().into()])?;
    }
    execute!(&qb)?;
  }
  log::info!("Finished updating Steam users");
  Ok(())
}

pub async fn update_playdata(user_list: &Vec<(i64,)>) -> R {
  // Yes a day, is never exactly the same, but I just need to round the timestamp to current day
  let day = (Utc::now().timestamp() / 86400) as i32;
  log::info!("Updating Steam playdata");
  let mut games = HashMap::new();
  let mut playdata = vec![];
  for user in user_list {
    if let Ok(res) = req()
      .get_owned_games(sapi_key(), user.0 as u64, true, true)
      .await
    {
      for game in res.response.games {
        games.insert(game.id as i32, game.name);
        playdata.push((user.0, game.id as i32, game.playtime as i32));
      }
    } else {
      log::warn!("Failed to get playdata of user '{}'", user.0);
    }
  }
  for chunk in games.into_iter().collect::<Vec<_>>().chunks(10000) {
    let mut qb = Query::insert();
    qb.into_table(Apps::Table);
    qb.columns([Apps::Id, Apps::Name]);
    qb.on_conflict(
      OnConflict::column(Apps::Id)
        .update_column(Apps::Name)
        .to_owned(),
    );
    for v in chunk {
      qb.values([v.0.into(), v.1.clone().into()])?;
    }
    execute!(&qb)?;
    log::info!("Updated {} apps", chunk.len());

  }
  for chunk in playdata.chunks(10000) {
    let mut qb = Query::insert();
    qb.into_table(Playdata::Table);
    qb.columns([Playdata::UserId, Playdata::AppId, Playdata::Playtime]);
    qb.on_conflict(
      OnConflict::columns([Playdata::UserId, Playdata::AppId])
        .update_column(Playdata::Playtime)
        .to_owned(),
    );
    qb.returning(Query::returning().columns([Playdata::Id, Playdata::Playtime]));
    for v in chunk {
      qb.values([v.0.into(), v.1.into(), v.2.into()])?;
    }
    let updates = fetch_all!(&qb, (i32, i32))?;
    log::trace!("Updated {} playdata rows", chunk.len());
    let mut qb = Query::insert();
    qb.into_table(PlaydataHistory::Table);
    qb.columns([
      PlaydataHistory::PlaydataId,
      PlaydataHistory::UtcDay,
      PlaydataHistory::Playtime,
    ]);
    qb.on_conflict(
      OnConflict::columns([PlaydataHistory::PlaydataId, PlaydataHistory::UtcDay])
        .update_column(PlaydataHistory::Playtime)
        .to_owned(),
    );
    for v in updates {
      qb.values([v.0.into(), day.into(), v.1.into()])?;
    }
    execute!(&qb)?;
    log::trace!("Updated {} playdata history rows", chunk.len());
  }
  log::info!("Finished updating Steam playdata");
  Ok(())
}
