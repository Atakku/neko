// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  interface::steam::{IPlayerService, ISteamApps, ISteamUser},
  modules::{reqwest::req, steam::sapi_key},
  schema::*,
};
use chrono::Utc;
use poise::ChoiceParameter;
use sea_query::{Alias, Expr, Func, OnConflict, Order, Query, SelectStatement, WindowStatement};
use sqlx::FromRow;
use std::collections::HashMap;

pub async fn update_apps() -> R {
  use steam::Apps::*;
  log::info!("Updating Steam apps");
  let apps = req().get_app_list().await?.applist.apps;
  for apps_chunk in apps.chunks(10000) {
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([Id, Name]);
    qb.on_conflict(OnConflict::column(Id).update_column(Name).to_owned());
    for app in apps_chunk {
      qb.values([(app.id as i64).into(), app.name.clone().into()])?;
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
    match futures::join!(req()
      .get_player_summaries(
        sapi_key(),
        chunk
          .into_iter()
          .map(|i| i.0.to_string())
          .collect::<Vec<String>>()
          .join(","),
      ), ratelimit())
    {
      (Ok(res), _) => {
        for user in res.response.players {
          profiles.push((user.id.parse::<i64>()?, user.name));
        }
      }
      (Err(err), _) => log::warn!("Failed to get '{}' profile summaries: {err}", chunk.len()),
    }
  }
  for chunk in profiles.chunks(10000) {
    use steam::Users::*;
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([Id, Name]);
    qb.on_conflict(OnConflict::column(Id).update_column(Name).to_owned());
    for v in chunk {
      qb.values([v.0.into(), v.1.clone().into()])?;
    }
    execute!(&qb)?;
  }
  log::info!("Finished updating Steam users");
  Ok(())
}

pub async fn ratelimit() {
  std::thread::sleep(std::time::Duration::from_millis(1600));
}

pub async fn update_playdata(user_list: &Vec<(i64,)>) -> R {
  // Yes a day, is never exactly the same, but I just need to round the timestamp to current day
  let day = (Utc::now().timestamp() / 86400) as i32;
  log::info!("Updating Steam playdata");
  let mut games = HashMap::new();
  let mut playdata = vec![];
  for user in user_list {
    if let (Ok(res), _) = futures::join!(req()
      .get_owned_games(sapi_key(), user.0 as u64, true, true, false), ratelimit())
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
    use steam::Apps::*;
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([Id, Name]);
    qb.on_conflict(OnConflict::column(Id).update_column(Name).to_owned());
    for v in chunk {
      qb.values([v.0.into(), v.1.clone().into()])?;
    }
    execute!(&qb)?;
    log::info!("Updated {} apps", chunk.len());
  }
  for chunk in playdata.chunks(10000) {
    let updates = {
      use steam::Playdata::*;
      let mut qb = Query::insert();
      qb.into_table(Table);
      qb.columns([UserId, AppId, Playtime]);
      qb.on_conflict(
        OnConflict::columns([UserId, AppId])
          .update_column(Playtime)
          .to_owned(),
      );
      qb.returning(Query::returning().columns([Id, Playtime]));
      for v in chunk {
        qb.values([v.0.into(), v.1.into(), v.2.into()])?;
      }
      fetch_all!(&qb, (i64, i32))?
    };
    log::trace!("Updated {} playdata rows", chunk.len());
    {
      use steam::PlaydataHistory::*;
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
      execute!(&qb)?;
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
  qb.from(steam::Playdata::Table);
  nekoid_eq(&mut qb, &of, &at);
  member_eq(&mut qb, &of, &at);
  match of {
    Of::Apps => {
      use steam::{Apps, Playdata};
      qb.from(Apps::Table);
      qb.and_where(ex_col!(Apps, Id).equals(col!(Playdata, AppId)));
      qb.group_by_col(col!(Apps, Id));
      qb.columns([col!(Apps, Id), col!(Apps, Name)]);
    }
    Of::Guilds => {
      use discord::{Guilds, Members};
      qb.from(Guilds::Table);
      qb.and_where(ex_col!(Guilds, Id).equals(col!(Members, GuildId)));
      qb.group_by_col(col!(Guilds, Id));
      qb.columns([col!(Guilds, Id), col!(Guilds, Name)]);
    }
    Of::Users => {
      use discord::Users;
      use neko::UsersDiscord;
      qb.from(Users::Table);
      qb.and_where(ex_col!(Users, Id).equals(col!(UsersDiscord, DiscordId)));
      qb.group_by_col(col!(Users, Id));
      qb.columns([col!(Users, Id), col!(Users, Name)]);
    }
  }
  {
    use steam::Playdata;
    qb.expr_as(
      match by {
        By::Playtime => Func::sum(ex_col!(Playdata, Playtime)),
        By::Ownership => Func::count(ex_col!(Playdata, AppId)),
      },
      Alias::new("sum_count"),
    );
    qb.expr_window_as(
      Func::cust(Alias::new("ROW_NUMBER")),
      WindowStatement::new()
        .order_by_expr(
          match by {
            By::Playtime => Expr::sum(ex_col!(Playdata, Playtime)),
            By::Ownership => Expr::count(ex_col!(Playdata, AppId)),
          },
          Order::Desc,
        )
        .to_owned(),
      Alias::new("row_num"),
    );
  }
  match at {
    At::User(id) => qb.and_where(ex_col!(neko::UsersDiscord, DiscordId).eq(id)),
    At::Guild(id) => qb.and_where(ex_col!(discord::Members, GuildId).eq(id)),
    At::App(id) => qb.and_where(ex_col!(steam::Playdata, AppId).eq(id)),
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
  use neko::*;
  qb.from(UsersSteam::Table);
  qb.from(UsersDiscord::Table);
  qb.and_where(ex_col!(UsersSteam, NekoId).equals(col!(UsersDiscord, NekoId)));
  qb.and_where(ex_col!(UsersSteam, SteamId).equals(col!(steam::Playdata, UserId)));
}

fn member_eq(qb: &mut SelectStatement, of: &Of, at: &At) {
  not_match!(of, Of::Guilds, {
    not_match!(at, At::Guild(_), {
      return;
    });
  });
  use discord::*;
  use neko::*;
  qb.from(Members::Table);
  qb.and_where(
    Expr::col((Members::Table, Members::UserId))
      .equals((UsersDiscord::Table, UsersDiscord::DiscordId)),
  );
}

#[derive(FromRow)]
pub struct QueryOutput {
  pub row_num: i64,
  pub sum_count: i64,
  pub id: i64,
  pub name: String,
}
