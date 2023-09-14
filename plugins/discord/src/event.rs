// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use neko_core::*;
use neko_poise::EH;
use poise::{
  futures_util::StreamExt,
  serenity_prelude::UserId,
  {
    serenity_prelude::{Context, GuildId, Member, User},
    Event,
  },
};
use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::PgPool;

use crate::schema::{Guilds, Members, Users};

pub fn event_handler() -> EH {
  |c, e, _f, s| {
    Box::pin(async move {
      let db = s.read().await.borrow::<PgPool>()?.clone();
      match e {
        Event::GuildCreate {
          guild: g,
          is_new: _,
        } => {
          update_guild(c, g.id, &db).await?;
          let res: Vec<_> = g.id.members_iter(c).collect().await;
          let members: Vec<_> = res
            .into_iter()
            .filter_map(Result::ok)
            .filter(|m| !m.user.bot)
            .collect();
          let users: Vec<_> = members.clone().into_iter().map(|m| m.user).collect();
          update_users(users, &db).await?;
          // Prune members (bot may have been offline and missed guild leaves)
          prune_members(g.id, &db).await?;
          update_members(members, &db).await?;
        }
        Event::GuildUpdate {
          old_data_if_available: _,
          new_but_incomplete: g,
        } => {
          update_guild(&c, g.id, &db).await?;
        }
        Event::GuildDelete {
          incomplete: g,
          full: _,
        } => {
          if !g.unavailable {
            remove_guild(g.id, &db).await?;
          }
        }
        Event::GuildMemberAddition { new_member: m } => {
          if !m.user.bot {
            update_users(vec![m.user.clone()], &db).await?;
            update_members(vec![m.clone()], &db).await?;
          }
        }
        Event::GuildMemberUpdate {
          old_if_available: _,
          new: m,
        } => {
          if !m.user.bot {
            update_users(vec![m.user.clone()], &db).await?;
            update_members(vec![m.clone()], &db).await?;
          }
        }
        Event::GuildMemberRemoval {
          guild_id: g,
          user: u,
          member_data_if_available: _,
        } => {
          if !u.bot {
            remove_member(*g, u.id, &db).await?;
          }
        }
        _ => {}
      }
      Ok(())
    })
  }
}

async fn update_guild(ctx: &Context, id: GuildId, db: &PgPool) -> R {
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
  let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
  sqlx::query_with(&q, v).execute(db).await?;
  Ok(())
}

async fn remove_guild(id: GuildId, db: &PgPool) -> R {
  log::trace!("Removing {id} information from db");
  let mut qb = Query::delete();
  qb.from_table(Guilds::Table);
  qb.cond_where(Expr::col(Guilds::Id).eq(id.0));
  let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
  sqlx::query_with(&q, v).execute(db).await?;
  Ok(())
}

const CHUNK_SIZE: usize = 10000;

async fn update_users(users: Vec<User>, db: &PgPool) -> R {
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
    let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&q, v).execute(db).await?;
  }
  Ok(())
}

async fn prune_members(id: GuildId, db: &PgPool) -> R {
  log::trace!("Pruning members from {id}");
  let mut qb = Query::delete();
  qb.from_table(Members::Table);
  qb.cond_where(Expr::col(Members::GuildId).eq(id.0));
  let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
  sqlx::query_with(&q, v).execute(db).await?;
  Ok(())
}

async fn update_members(members: Vec<Member>, db: &PgPool) -> R {
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
    let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
    sqlx::query_with(&q, v).execute(db).await?;
  }
  Ok(())
}

async fn remove_member(g: GuildId, u: UserId, db: &PgPool) -> R {
  let mut qb = Query::delete();
  qb.from_table(Members::Table);
  qb.cond_where(Expr::col(Members::GuildId).eq(g.0));
  qb.cond_where(Expr::col(Members::UserId).eq(u.0));
  let (q, v) = qb.build_sqlx(PostgresQueryBuilder);
  sqlx::query_with(&q, v).execute(db).await?;
  Ok(())
}
