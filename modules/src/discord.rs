// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use futures::StreamExt;
use neko_core::*;
use neko_db::{discord::*, execute};
use poise::{
  serenity_prelude::{Context, GatewayIntents, GuildId, Member, User, UserId},
  Event,
};
use sea_query::{Expr, OnConflict, Query};

use crate::{poise::EventHandler, Poise};

/// Discord scraper module, populates the database with user data (users, guilds, members)
pub struct Discord;

impl Module for Discord {
  fn init(&self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>()?;
    poise.event_handlers.push(event_handler());
    poise.intents.insert(GatewayIntents::GUILDS);
    poise.intents.insert(GatewayIntents::GUILD_MEMBERS);
    Ok(())
  }
}

fn event_handler() -> EventHandler {
  |c, e, _f, _| {
    Box::pin(async move {
      match e {
        Event::GuildCreate {
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
        Event::GuildUpdate {
          old_data_if_available: _,
          new_but_incomplete: g,
        } => {
          update_guild(&c, g.id).await?;
        }
        Event::GuildDelete {
          incomplete: g,
          full: _,
        } => {
          if !g.unavailable {
            remove_guild(g.id).await?;
          }
        }
        Event::GuildMemberAddition { new_member: m } => {
          if !m.user.bot {
            update_users(vec![m.user.clone()]).await?;
            update_members(vec![m.clone()]).await?;
          }
        }
        Event::GuildMemberUpdate {
          old_if_available: _,
          new: m,
        } => {
          if !m.user.bot {
            update_users(vec![m.user.clone()]).await?;
            update_members(vec![m.clone()]).await?;
          }
        }
        Event::GuildMemberRemoval {
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
  log::trace!("Requesting {id} information");
  let info = id.get_preview(ctx).await?;
  log::trace!("Upserting {id} information into db");
  let mut qb = Query::insert();
  qb.into_table(Guilds::Table);
  qb.columns([Guilds::Id, Guilds::Name, Guilds::Icon]);
  qb.on_conflict(
    OnConflict::column(Guilds::Id)
      .update_columns([Guilds::Name, Guilds::Icon])
      .to_owned(),
  );
  qb.values([info.id.0.into(), info.name.into(), info.icon.into()])?;
  execute(qb).await?;
  Ok(())
}

async fn remove_guild(id: GuildId) -> R {
  log::trace!("Removing {id} information from db");
  let mut qb = Query::delete();
  qb.from_table(Guilds::Table);
  qb.cond_where(Expr::col(Guilds::Id).eq(id.0));
  execute(qb).await?;
  Ok(())
}

const CHUNK_SIZE: usize = 10000;

async fn update_users(users: Vec<User>) -> R {
  log::trace!("Updating {} users", users.len());
  for chunk in users.chunks(CHUNK_SIZE) {
    let mut qb = Query::insert();
    qb.into_table(Users::Table);
    qb.columns([Users::Id, Users::Username, Users::Nickname, Users::Avatar]);
    qb.on_conflict(
      OnConflict::column(Users::Id)
        .update_columns([Users::Username, Users::Nickname, Users::Avatar])
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
    execute(qb).await?;
  }
  Ok(())
}

async fn prune_members(id: GuildId) -> R {
  log::trace!("Pruning members from {id}");
  let mut qb = Query::delete();
  qb.from_table(Members::Table);
  qb.cond_where(Expr::col(Members::GuildId).eq(id.0));
  execute(qb).await?;
  Ok(())
}

async fn update_members(members: Vec<Member>) -> R {
  log::trace!("Updating {} members", members.len());
  for chunk in members.chunks(CHUNK_SIZE) {
    let mut qb = Query::insert();
    qb.into_table(Members::Table);
    qb.columns([
      Members::GuildId,
      Members::UserId,
      Members::Nickname,
      Members::Avatar,
    ]);
    qb.on_conflict(
      OnConflict::columns([Members::GuildId, Members::UserId])
        .update_columns([Members::Nickname, Members::Avatar])
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
    execute(qb).await?;
  }
  Ok(())
}

async fn remove_member(g: GuildId, u: UserId) -> R {
  let mut qb = Query::delete();
  qb.from_table(Members::Table);
  qb.cond_where(Expr::col(Members::GuildId).eq(g.0));
  qb.cond_where(Expr::col(Members::UserId).eq(u.0));
  execute(qb).await?;
  Ok(())
}
