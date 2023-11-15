// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  modules::{
    poise::{EventHandler, Poise},
    sqlx::Postgres,
  }, plugins::discord_cache::schema::DiscordCacheGuilds,
};
use futures::StreamExt;
use poise::{
  serenity_prelude::{Context, GatewayIntents, GuildId, Member, User, UserId},
  Event,
};
use sea_query::{Expr, OnConflict, Query};

autocomplete!(discord_guilds, DiscordCacheGuilds, GuildId, Name);

module!{
  /// In database cache of all known Discord guilds/users/members for usage in database queries
  DiscordCache;

  fn init(fw) {
    let pg = fw.req::<Postgres>()?;
    pg.create_tables(&mut super::schema::create_tables());
    let poise = fw.req::<Poise>()?;
    poise.add_event_handler(event_handler());
    poise.add_intent(GatewayIntents::GUILDS);
    poise.add_intent(GatewayIntents::GUILD_MEMBERS);
  }
}

fn event_handler() -> EventHandler {
  |c, event| {
    Box::pin(async move {
      use Event::*;
      match event {
        Ready { data_about_bot: _ } => {
          prune_all_guilds().await?;
          // Removing guild cascades to removing all guild members
        }
        GuildCreate {
          guild: g,
          is_new: _,
        } => {
          if check_guild_whitelist(g.id).await? {
            update_guild(c, g.id).await?;
            let res: Vec<_> = g.id.members_iter(c).collect().await;
            let members: Vec<_> = res
              .into_iter()
              .filter_map(Result::ok)
              .filter(|m| !m.user.bot)
              .collect();
            let users: Vec<_> = members.clone().into_iter().map(|m| m.user).collect();
            update_users(users).await?;
            // No need to prune members, as bot does that on GuildDelete and Ready
            update_members(members).await?;
          }
        }
        GuildUpdate {
          old_data_if_available: _,
          new_but_incomplete: g,
        } => {
          if check_guild_whitelist(g.id).await? {
            update_guild(&c, g.id).await?;
          }
        }
        GuildDelete {
          incomplete: g,
          full: _,
        } => {
          if !g.unavailable && check_guild_whitelist(g.id).await? {
            remove_guild(g.id).await?;
            // Removing guild cascades to removing all guild members
          }
        }
        GuildMemberAddition { new_member: m } => {
          if !m.user.bot && check_guild_whitelist(m.guild_id).await? {
            update_users(vec![m.user.clone()]).await?;
            update_members(vec![m.clone()]).await?;
          }
        }
        GuildMemberUpdate {
          old_if_available: _,
          new: m,
        } => {
          if !m.user.bot && check_guild_whitelist(m.guild_id).await? {
            update_users(vec![m.user.clone()]).await?;
            update_members(vec![m.clone()]).await?;
          }
        }
        GuildMemberRemoval {
          guild_id: g,
          user: u,
          member_data_if_available: _,
        } => {
          if !u.bot && check_guild_whitelist(*g).await? {
            remove_member(*g, u.id).await?;
          }
        }
        _ => {}
      }
      Ok(())
    })
  }
}

async fn check_guild_whitelist(id: GuildId) -> Res<bool> {
  use crate::plugins::neko::schema::NekoWhitelistDiscord::*;
  let mut qb = Query::select();
  qb.from(Table);
  qb.column(GuildId);
  qb.and_where(Expr::col((Table, GuildId)).eq(id.0 as i64));
  Ok(sql!(FetchOpt, &qb, (i64,))?.is_some())
}

async fn update_guild(ctx: &Context, id: GuildId) -> R {
  use super::schema::DiscordCacheGuilds::*;
  log::trace!("Requesting {id} information");
  let info = id.get_preview(ctx).await?;
  log::trace!("Upserting {id} information into db");
  let mut qb = Query::insert();
  qb.into_table(Table);
  qb.columns([GuildId, Name, Icon]);
  qb.on_conflict(
    OnConflict::column(GuildId)
      .update_columns([Name, Icon])
      .to_owned(),
  );
  qb.values([info.id.0.into(), info.name.into(), info.icon.into()])?;
  sql!(Execute, &qb)?;
  Ok(())
}

async fn remove_guild(id: GuildId) -> R {
  use super::schema::DiscordCacheGuilds::*;
  log::trace!("Removing {id} information from db");
  let mut qb = Query::delete();
  qb.from_table(Table);
  qb.cond_where(Expr::col(GuildId).eq(id.0));
  sql!(Execute, &qb)?;
  Ok(())
}

async fn prune_all_guilds() -> R {
  use super::schema::DiscordCacheGuilds::*;
  log::trace!("Pruning all guilds");
  let mut qb = Query::delete();
  qb.from_table(Table);
  sql!(Execute, &qb)?;
  Ok(())
}

const CHUNK_SIZE: usize = 10000;

async fn update_users(users: Vec<User>) -> R {
  use super::schema::DiscordCacheUsers::*;
  log::trace!("Updating {} users", users.len());
  for chunk in users.chunks(CHUNK_SIZE) {
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([UserId, Name, Nick, Avatar]);
    qb.on_conflict(
      OnConflict::column(UserId)
        .update_columns([Name, Nick, Avatar])
        .to_owned(),
    );
    for row in chunk.into_iter().map(|u| {
      [
        u.id.0.into(),
        u.name.clone().into(),
        None::<String>.into(),
        u.avatar.clone().into(),
      ]
    }) {
      qb.values(row)?;
    }
    sql!(Execute, &qb)?;
  }
  Ok(())
}

async fn update_members(members: Vec<Member>) -> R {
  use super::schema::DiscordCacheMembers::*;
  log::trace!("Updating {} members", members.len());
  for chunk in members.chunks(CHUNK_SIZE) {
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([GuildId, UserId, Nick, Avatar]);
    qb.on_conflict(
      OnConflict::columns([GuildId, UserId])
        .update_columns([Nick, Avatar])
        .to_owned(),
    );
    for row in chunk.into_iter().map(|m| {
      [
        m.guild_id.0.into(),
        m.user.id.0.into(),
        m.nick.clone().into(),
        m.avatar.clone().into(),
      ]
    }) {
      qb.values(row)?;
    }
    sql!(Execute, &qb)?;
  }
  Ok(())
}

async fn remove_member(g: GuildId, u: UserId) -> R {
  use super::schema::DiscordCacheMembers::*;
  let mut qb = Query::delete();
  qb.from_table(Table);
  qb.cond_where(Expr::col(GuildId).eq(g.0));
  qb.cond_where(Expr::col(UserId).eq(u.0));
  sql!(Execute, &qb)?;
  Ok(())
}
