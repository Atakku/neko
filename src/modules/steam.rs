// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::{axum::Axum, cron::Cron, poise::EventHandler};
use crate::{
  core::*,
  modules::{
    poise::{Ctx, Poise},
    sqlx::Postgres,
  },
  query::{
    autocomplete::*,
    neko::all_steam_connections,
    steam::{build_top_query, update_apps, update_playdata, update_users, At, By, Of, QueryOutput},
  },
};
use axum::routing::get;
use poise::{
  serenity_prelude::{Member, Role, RoleId, UserId},
  Event,
};
use sea_query::{Query};
use tokio_cron_scheduler::Job;

pub struct Steam;

once_cell!(sapi_key, APIKEY: String);

impl Module for Steam {
  fn init(&mut self, fw: &mut Framework) -> R {
    APIKEY.set(expect_env!("STEAMAPI_KEY"))?;
    fw.req_module::<Postgres>()?;
    let poise = fw.req_module::<Poise>()?;
    poise.commands.push(steam());
    poise.event_handlers.push(roles());
    let cron = fw.req_module::<Cron>()?;
    cron.jobs.push(Job::new_async("0 0 */6 * * *", |_id, _jsl| {
      Box::pin(async move {
        minor_update().await.unwrap();
      })
    })?);
    cron.jobs.push(Job::new_async("0 0 0 */7 * *", |_id, _jsl| {
      Box::pin(async move {
        //update_apps().await.unwrap()
      })
    })?);
    Ok(())
  }
}

fn roles() -> EventHandler {
  |c, event| {
    Box::pin(async move {
      use Event::*;
      match event {
        GuildMemberAddition { new_member: m } => {
          if !m.user.bot {
            let roles = filter_roles(
              m.roles(c).unwrap_or_default(),
              get_roles(m).await?,
            );
            let mut member = m.guild_id.member(c, m.user.id).await?;
            member.add_roles(c, roles.as_slice()).await?;
          }
        }
        _ => {}
      }
      Ok(())
    })
  }
}

pub async fn get_roles( m: &Member) -> Res<Vec<RoleId>> {
  use crate::schema::*;
  let mut qb = Query::select();
  qb.from(steam::DiscordRoles::Table);
  qb.from(steam::Playdata::Table);
  qb.from(neko::UsersSteam::Table);
  qb.from(neko::UsersDiscord::Table);
  qb.column(col!(steam::DiscordRoles, RoleId));

  qb.cond_where(ex_col!(steam::DiscordRoles, GuildId).eq(m.guild_id.0 as i64));
  qb.cond_where(ex_col!(steam::DiscordRoles, AppId).equals(col!(steam::Playdata, AppId)));
  qb.cond_where(ex_col!(neko::UsersSteam, SteamId).equals(col!(steam::Playdata, UserId)));
  qb.cond_where(ex_col!(neko::UsersSteam, NekoId).equals(col!(neko::UsersDiscord, NekoId)));
  qb.cond_where(ex_col!(neko::UsersDiscord, DiscordId).eq(m.user.id.0 as i64));
  qb.distinct();
  Ok(fetch_all!(&qb, (i64,))?.into_iter()
  .map(|r| RoleId(r.0 as u64))
  .collect())
}

pub fn filter_roles(og: Vec<Role>, add: Vec<RoleId>) -> Vec<RoleId> {
  let mapped: Vec<u64> = og.into_iter().map(|r| r.id.0).collect();
  add.into_iter().filter(|r| !mapped.contains(&r.0)).collect()
}

pub async fn minor_update() -> R {
  let c = all_steam_connections().await?;
  update_users(&c).await?;
  update_playdata(&c).await?;
  Ok(())
}

cmd_group!(steam, "user_top", "app_top", "guild_top", "top");

#[poise::command(prefix_command, slash_command)]
pub async fn top(ctx: Ctx<'_>, of: Of, by: By) -> R {
  let output = format!("Top of {of} by {by}");
  let m = ctx.reply(output.clone()).await?;
  let c = fetch(output, of, by, At::None).await?;
  m.edit(ctx, |m| m.content(c)).await?;
  Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn user_top(ctx: Ctx<'_>, of: Of, by: By, user: UserId) -> R {
  let output = format!("Users's ({user}) top of {of} by {by}");
  let m = ctx.reply(output.clone()).await?;
  let c = fetch(output, of, by, At::User(user.0 as i64)).await?;
  m.edit(ctx, |m| m.content(c)).await?;
  Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn app_top(ctx: Ctx<'_>, of: Of, by: By, #[autocomplete = "steam_apps"] app: i32) -> R {
  let output = format!("Apps's ({app}) top of {of} by {by}");
  let m = ctx.reply(output.clone()).await?;
  let c = fetch(output, of, by, At::App(app)).await?;
  m.edit(ctx, |m| m.content(c)).await?;
  Ok(())
}

#[poise::command(prefix_command, slash_command)]
pub async fn guild_top(
  ctx: Ctx<'_>,
  of: Of,
  by: By,
  #[autocomplete = "discord_guilds"] guild: String,
) -> R {
  let output = format!("Guild's ({guild}) top of {of} by {by}");
  let m = ctx.reply(output.clone()).await?;
  let c = fetch(output, of, by, At::Guild(guild.parse::<i64>()?)).await?;
  m.edit(ctx, |m| m.content(c)).await?;
  Ok(())
}

async fn fetch(input: String, of: Of, by: By, at: At) -> Res<String> {
  let mut output = String::new();
  let bys = match by {
    By::Playtime => "hours",
    By::Ownership => "copies",
  };
  let divider = if let By::Playtime = by { 60 } else { 1 };
  let data = fetch_all!(&build_top_query(of, by, at), QueryOutput)?;
  for d in data.into_iter().take(10) {
    output += &format!("{} | {} | {}\n", d.row_num, d.sum_count / divider, d.name);
  }
  Ok(format!("{input}\n```\n# | {bys} | name \n{output}```"))
}
