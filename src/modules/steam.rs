// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::{axum::Axum, cron::Cron};
use crate::{
  core::*,
  interface::steam::{IPlayerService, ISteamApps},
  modules::{
    poise::{Ctx, Poise},
    reqwest::req,
    sqlx::Postgres,
  },
  schema::{
    discord::{Guilds, Members},
    steam::*,
  }, query::{steam::{update_apps, update_users, update_playdata}, neko::all_steam_connections}
};
use axum::routing::get;
use poise::{
  serenity_prelude::UserId,
  ChoiceParameter,
};
use sea_query::{Alias, Expr, Func, OnConflict, Order, Query, WindowStatement};
use tokio_cron_scheduler::Job;

pub struct Steam;

once_cell!(sapi_key, APIKEY: String);

async fn root() -> &'static str {
  "Hello from steam module!"
}

impl Module for Steam {
  fn init(&mut self, fw: &mut Framework) -> R {
    APIKEY.set(expect_env!("STEAMAPI_KEY"))?;
    fw.req_module::<Postgres>()?;
    let axum = fw.req_module::<Axum>()?;
    axum.routes.push(|r| r.route("/", get(root)));
    let poise = fw.req_module::<Poise>()?;
    poise.commands.push(steam());
    let cron = fw.req_module::<Cron>()?;
    cron.jobs.push(Job::new_async("0 0 */6 * * *", |_id, _jsl| {
      Box::pin(async move {
        let conns = all_steam_connections().await.unwrap();
        update_users(&conns).await.unwrap();
        update_playdata(&conns).await.unwrap();
      })
    })?);
    cron.jobs.push(Job::new_async("0 0 0 */7 * *", |_id, _jsl| {
      Box::pin(async move {
        update_apps().await.unwrap()
      })
    })?);
    Ok(())
  }
}

#[poise::command(prefix_command, slash_command, subcommand_required, subcommands("top"))]
pub async fn steam(_: Ctx<'_>) -> R {
  Ok(())
}

#[poise::command(prefix_command, slash_command, subcommands("user", "app", "guild"))]
pub async fn top(ctx: Ctx<'_>, by: By) -> R {
  ctx.reply(format!("Global top: {by}")).await?;
  Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn user(ctx: Ctx<'_>, by: By, user: UserId) -> R {
  ctx.reply(format!("Top in user: {user} {by}")).await?;
  Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn app(
  ctx: Ctx<'_>,
  by: By,
  #[autocomplete = "crate::query::autocomplete::steam_apps"] app: i32,
) -> R {
  ctx.reply(format!("Top in app: {app} {by}")).await?;
  Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn guild(
  ctx: Ctx<'_>,
  by: By,
  #[autocomplete = "crate::query::autocomplete::discord_guilds"] guild: String,
) -> R {
  ctx.reply(format!("Top in guild: {guild} {by}")).await?;
  Ok(())
}

#[derive(ChoiceParameter)]
pub enum By {
  Playtime,
  Ownership, // TODO: TopCompleteon (achievemetns)
}

pub enum Target {
  User(i64),
  Guild(i64),
  App(i32),
  Global,
}

pub enum Mode {
  Apps,   // Top apps in user, guild, or global, by hours or count
  Guilds, // Top guilds by app hours or app count, in app or global
  Users,  // Top users by app hours or app count, in guild or global
}

pub struct Top {
  mode: Mode,
  by: By,
  target: Target,
}

pub fn query_builder(top: Top) {
  let mut qb = Query::select();
  qb.from(Playdata::Table);
  qb.expr_as(
    Func::count(Expr::col((
      Playdata::Table,
      match top.by {
        By::Playtime => Playdata::Playtime,
        By::Ownership => Playdata::AppId,
      },
    ))),
    Alias::new("sum_count"),
  );
  qb.expr_window_as(
    Func::cust(Alias::new("ROW_NUMBER")),
    WindowStatement::new()
      .order_by_expr(
        Expr::sum(Expr::col((
          Playdata::Table,
          match top.by {
            By::Playtime => Playdata::Playtime,
            By::Ownership => Playdata::AppId,
          },
        ))),
        Order::Desc,
      )
      .to_owned(),
    Alias::new("row_num"),
  );
  qb.order_by(Alias::new("sum_count"), Order::Desc);
  match top.mode {
    Mode::Apps => {
      qb.from(Apps::Table);
      qb.columns([(Apps::Table, Apps::Id), (Apps::Table, Apps::Name)]);
    }
    Mode::Guilds => {
      qb.from(Guilds::Table);
      qb.columns([(Guilds::Table, Guilds::Id), (Guilds::Table, Guilds::Name)]);
    }
    Mode::Users => {
      // TODO: members context
      qb.from(Users::Table);
      qb.columns([(Users::Table, Users::Id), (Users::Table, Users::Name)]);
    }
  }
  match top.target {
    Target::User(id) => {
      qb.and_where(Expr::col((Members::Table, Members::UserId)).eq(id));
    }
    Target::Guild(id) => {
      qb.and_where(Expr::col((Members::Table, Members::GuildId)).eq(id));
    }
    Target::App(id) => {
      qb.and_where(Expr::col((Apps::Table, Apps::Id)).eq(id));
    }
    Target::Global => {}
  };
}
