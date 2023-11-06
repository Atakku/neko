// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*, plugins::steam::{sapi_key, wrapper::{get_app_list, get_owned_games, get_player_summaries}},
};
use crate::plugins::*;
use chrono::Utc;
use poise::ChoiceParameter;
use sea_query::{Alias, Expr, Func, OnConflict, Order, Query, SelectStatement, WindowStatement};
use sqlx::FromRow;
use std::collections::HashMap;

pub async fn update_apps() -> R {
  use super::schema::SteamApps::*;
  log::info!("Updating Steam apps");
  let apps = get_app_list().await?.applist.apps;
  for apps_chunk in apps.chunks(10000) {
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([AppId, AppName]);
    qb.on_conflict(OnConflict::column(AppId).update_column(AppName).to_owned());
    for app in apps_chunk {
      qb.values([(app.id as i64).into(), app.name.clone().into()])?;
    }
    sql!(Execute, &qb)?;
    log::info!("Updated {} apps", apps_chunk.len());
  }
  log::info!("Finished updating Steam apps");
  Ok(())
}

pub async fn update_users(user_list: &Vec<(i64,)>) -> R {
  log::info!("Updating Steam users");
  let mut profiles = vec![];
  for chunk in user_list.chunks(100) {
    match get_player_summaries(
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
    use super::schema::SteamUsers::*;
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([SteamId, Username]);
    qb.on_conflict(OnConflict::column(SteamId).update_column(Username).to_owned());
    for v in chunk {
      qb.values([v.0.into(), v.1.clone().into()])?;
    }
    sql!(Execute, &qb)?;
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
    if let Ok(res) = 
      get_owned_games(sapi_key(), user.0 as u64, true, true)
      .await
    {
      for game in res.response.games {
        games.insert(game.id as i64, game.name);
        playdata.push((user.0, game.id as i64, game.playtime as i32));
      }
    } else {
      log::warn!("Failed to get playdata of user '{}'", user.0);
    }
  }
  for chunk in games.into_iter().collect::<Vec<_>>().chunks(10000) {
    use super::schema::SteamApps::*;
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([AppId, AppName]);
    qb.on_conflict(OnConflict::column(AppId).update_column(AppName).to_owned());
    for v in chunk {
      qb.values([v.0.into(), v.1.clone().into()])?;
    }
    sql!(Execute, &qb)?;
    log::info!("Updated {} apps", chunk.len());
  }
  for chunk in playdata.chunks(10000) {
    let updates = {
      use super::schema::SteamPlaydata::*;
      let mut qb = Query::insert();
      qb.into_table(Table);
      qb.columns([UserId, AppId, Playtime]);
      qb.on_conflict(
        OnConflict::columns([UserId, AppId])
          .update_column(Playtime)
          .to_owned(),
      );
      qb.returning(Query::returning().columns([PlaydataId, Playtime]));
      for v in chunk {
        qb.values([v.0.into(), v.1.into(), v.2.into()])?;
      }
      sql!(FetchAll, &qb, (i64, i32))?
    };
    log::trace!("Updated {} playdata rows", chunk.len());
    {
      use super::schema::SteamPlayhist::*;
      let mut qb = Query::insert();
      qb.into_table(Table);
      qb.columns([PlaydataId, UtcDay, Playtime]);
      qb.on_conflict(
        OnConflict::columns([PlaydataId, UtcDay])
          .update_column(Playtime)
          .to_owned(),
      );
      for v in updates {
        qb.values([v.0.into(), day.into(), v.1.into()])?;
      }
      sql!(Execute, &qb)?;
    }
    log::trace!("Updated {} playdata history rows", chunk.len());
  }
  log::info!("Finished updating Steam playdata");
  Ok(())
}

#[derive(ChoiceParameter)]
pub enum By {
  Playtime,
  Ownership, // TODO: TopCompleteon (achievemetns)
}

pub enum At {
  User(i64),
  Guild(i64),
  App(i32),
  None,
}

#[derive(ChoiceParameter)]
pub enum Of {
  Apps,   // Top apps in user, guild, or global, by hours or count
  Guilds, // Top guilds by app hours or app count, in app or global
  Users,  // Top users by app hours or app count, in guild or global
}

// Top of (Apps, Guilds, Users) by (Playtime, Ownership) at (User, Guild, App, Global)
pub fn build_top_query(of: Of, by: By, at: At) -> SelectStatement {
  let mut qb = Query::select();
  qb.from(super::schema::SteamPlaydata::Table);
  nekoid_eq(&mut qb, &of, &at);
  member_eq(&mut qb, &of, &at);
  match of {
    Of::Apps => {
      use super::schema::{SteamApps, SteamPlaydata};
      qb.from(SteamApps::Table);
      qb.and_where(ex_col!(SteamApps, AppId).equals(col!(SteamPlaydata, AppId)));
      qb.group_by_col(col!(SteamApps, AppId));
      qb.columns([col!(SteamApps, AppId), col!(SteamApps, AppName)]);
    }
    Of::Guilds => {
      use discord_cache::schema::{DiscordGuilds, DiscordMembers};
      qb.from(DiscordGuilds::Table);
      qb.and_where(ex_col!(DiscordGuilds, Id).equals(col!(DiscordMembers, GuildId)));
      qb.group_by_col(col!(DiscordGuilds, Id));
      qb.columns([col!(DiscordGuilds, Id), col!(DiscordGuilds, Name)]);
    }
    Of::Users => {
      use discord_cache::schema::DiscordUsers;
      use neko::schema::NekoUsersDiscord;
      qb.from(DiscordUsers::Table);
      qb.and_where(ex_col!(DiscordUsers, Id).equals(col!(NekoUsersDiscord, DiscordId)));
      qb.group_by_col(col!(DiscordUsers, Id));
      qb.columns([col!(DiscordUsers, Id), col!(DiscordUsers, Name)]);
    }
  }
  {
    use steam::schema::SteamPlaydata;
    qb.expr_as(
      match by {
        By::Playtime => Func::sum(ex_col!(SteamPlaydata, Playtime)),
        By::Ownership => Func::count(ex_col!(SteamPlaydata, AppId)),
      },
      Alias::new("sum_count"),
    );
    qb.expr_window_as(
      Func::cust(Alias::new("ROW_NUMBER")),
      WindowStatement::new()
        .order_by_expr(
          match by {
            By::Playtime => Expr::sum(ex_col!(SteamPlaydata, Playtime)),
            By::Ownership => Expr::count(ex_col!(SteamPlaydata, AppId)),
          },
          Order::Desc,
        )
        .to_owned(),
      Alias::new("row_num"),
    );
  }
  match at {
    At::User(id) => qb.and_where(ex_col!(neko::schema::NekoUsersDiscord, DiscordId).eq(id)),
    At::Guild(id) => qb.and_where(ex_col!(discord_cache::schema::DiscordMembers, GuildId).eq(id)),
    At::App(id) => qb.and_where(ex_col!(steam::schema::SteamPlaydata, AppId).eq(id)),
    At::None => &qb,
  };
  qb.order_by(Alias::new("sum_count"), Order::Desc);
  qb
}

fn nekoid_eq(qb: &mut SelectStatement, of: &Of, at: &At) {
  not_match!(of, Of::Guilds | Of::Users, {
    not_match!(at, At::Guild(_) | At::User(_), {
      return;
    });
  });
  use neko::schema::*;
  qb.from(NekoUsersSteam::Table);
  qb.from(NekoUsersDiscord::Table);
  qb.and_where(ex_col!(NekoUsersSteam, NekoId).equals(col!(NekoUsersDiscord, NekoId)));
  qb.and_where(ex_col!(NekoUsersSteam, SteamId).equals(col!(steam::schema::SteamPlaydata, UserId)));
}

fn member_eq(qb: &mut SelectStatement, of: &Of, at: &At) {
  not_match!(of, Of::Guilds, {
    not_match!(at, At::Guild(_), {
      return;
    });
  });
  use discord_cache::schema::*;
  use neko::schema::*;
  qb.from(DiscordMembers::Table);
  qb.and_where(
    Expr::col((DiscordMembers::Table, DiscordMembers::UserId))
      .equals((NekoUsersDiscord::Table, NekoUsersDiscord::DiscordId)),
  );
}

#[derive(FromRow, Clone)]
pub struct QueryOutput {
  pub row_num: i64,
  pub sum_count: i64,
  pub id: i64,
  pub name: String,
}
