// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  modules::{
    poise::{EventHandler, Poise},
    sqlx::Postgres,
  },
  schema::discord::*,
};
use futures::StreamExt;
use poise::{
  serenity_prelude::{Context, GatewayIntents, GuildId, Member, User, UserId},
  Event,
};
use sea_query::{Expr, OnConflict, Query};

/// Discord scraper module, populates the database with user data (users, guilds, members)
pub struct Discord;

impl Module for Discord {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>()?;
    let poise = fw.req_module::<Poise>()?;
    poise.event_handlers.push(event_handler());
    poise.intents.insert(GatewayIntents::GUILDS);
    poise.intents.insert(GatewayIntents::GUILD_MEMBERS);
    Ok(())
  }
}

fn event_handler() -> EventHandler {
  |c, event| {
    Box::pin(async move {
      use Event::*;
      match event {
        GuildCreate {
          guild: g,
          is_new: _,
        } => {
          update_guild(c, g.id).await?;
          let res: Vec<_> = g.id.members_iter(c).collect().await;
          let members: Vec<_> = res
            .into_iter()
            .filter_map(Result::ok)
            .filter(|m| !m.user.bot)
            .collect();
          let users: Vec<_> = members.clone().into_iter().map(|m| m.user).collect();
          update_users(users).await?;
          // Prune members (bot may have been offline and missed guild leaves)
          prune_members(g.id).await?;
          update_members(members).await?;
        }
        GuildUpdate {
          old_data_if_available: _,
          new_but_incomplete: g,
        } => {
          update_guild(&c, g.id).await?;
        }
        GuildDelete {
          incomplete: g,
          full: _,
        } => {
          if !g.unavailable {
            remove_guild(g.id).await?;
          }
        }
        GuildMemberAddition { new_member: m } => {
          if !m.user.bot {
            update_users(vec![m.user.clone()]).await?;
            update_members(vec![m.clone()]).await?;
          }
        }
        GuildMemberUpdate {
          old_if_available: _,
          new: m,
        } => {
          if !m.user.bot {
            update_users(vec![m.user.clone()]).await?;
            update_members(vec![m.clone()]).await?;
          }
        }
        GuildMemberRemoval {
          guild_id: g,
          user: u,
          member_data_if_available: _,
        } => {
          if !u.bot {
            remove_member(*g, u.id).await?;
          }
        }
        _ => {}
      }
      Ok(())
    })
  }
}

async fn update_guild(ctx: &Context, id: GuildId) -> R {
  use Guilds::*;
  log::trace!("Requesting {id} information");
  let info = id.get_preview(ctx).await?;
  log::trace!("Upserting {id} information into db");
  let mut qb = Query::insert();
  qb.into_table(Table);
  qb.columns([Id, Name, Icon]);
  qb.on_conflict(
    OnConflict::column(Id)
      .update_columns([Name, Icon])
      .to_owned(),
  );
  qb.values([info.id.0.into(), info.name.into(), info.icon.into()])?;
  execute!(&qb)?;
  Ok(())
}

async fn remove_guild(id: GuildId) -> R {
  use Guilds::*;
  log::trace!("Removing {id} information from db");
  let mut qb = Query::delete();
  qb.from_table(Table);
  qb.cond_where(Expr::col(Id).eq(id.0));
  execute!(&qb)?;
  Ok(())
}

const CHUNK_SIZE: usize = 10000;

async fn update_users(users: Vec<User>) -> R {
  use Users::*;
  log::trace!("Updating {} users", users.len());
  for chunk in users.chunks(CHUNK_SIZE) {
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([Id, Username, Nickname, Avatar]);
    qb.on_conflict(
      OnConflict::column(Id)
        .update_columns([Username, Nickname, Avatar])
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
    execute!(&qb)?;
  }
  Ok(())
}

async fn prune_members(id: GuildId) -> R {
  use Members::*;
  log::trace!("Pruning members from {id}");
  let mut qb = Query::delete();
  qb.from_table(Table);
  qb.cond_where(Expr::col(GuildId).eq(id.0));
  execute!(&qb)?;
  Ok(())
}

async fn update_members(members: Vec<Member>) -> R {
  use Members::*;
  log::trace!("Updating {} members", members.len());
  for chunk in members.chunks(CHUNK_SIZE) {
    let mut qb = Query::insert();
    qb.into_table(Table);
    qb.columns([GuildId, UserId, Nickname, Avatar]);
    qb.on_conflict(
      OnConflict::columns([GuildId, UserId])
        .update_columns([Nickname, Avatar])
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
    execute!(&qb)?;
  }
  Ok(())
}

async fn remove_member(g: GuildId, u: UserId) -> R {
  use Members::*;
  let mut qb = Query::delete();
  qb.from_table(Table);
  qb.cond_where(Expr::col(GuildId).eq(g.0));
  qb.cond_where(Expr::col(UserId).eq(u.0));
  execute!(&qb)?;
  Ok(())
}
