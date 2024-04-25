// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::{reqwest::{req, Reqwest}, sqlx::Postgres};
use crate::{core::*, modules::poise::Poise, query::starboard::*};
use log::{debug, error, info};
use poise::{
  serenity_prelude::{ChannelId, Context, GatewayIntents, GuildId, Message, MessageId, ReactionType},
  BoxFuture, Event,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Module with femboy.tv discord server functionality
pub struct Starboard;

impl Module for Starboard {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>()?;
    fw.req_module::<Reqwest>()?;
    let poise = fw.req_module::<Poise>()?;
    poise.intents.insert(GatewayIntents::GUILD_MESSAGE_REACTIONS);
    poise.event_handlers.push(event_handler);
    Ok(())
  }
}

fn event_handler<'a>(c: &'a Context, event: &'a Event<'a>) -> BoxFuture<'a, R> {
  Box::pin(async move {
    use Event::*;
    match event {
      ReactionAdd { add_reaction: e } => {
        if e.guild_id != Some(GuildId(1232659990993702943)) {
          return Ok(());
        }
        starboard_update(c, e.message(c).await?).await?;
      }
      ReactionRemove {
        removed_reaction: e,
      } => {
        if e.guild_id != Some(GuildId(1232659990993702943)) {
          return Ok(());
        }
        starboard_update(c, e.message(c).await?).await?;
      }
      _ => {}
    }
    Ok(())
  })
}
async fn starboard_update<'a>(c: &'a Context, m: Message) -> Res<()> {

  let ch = m.channel(c).await?;
  let spoiler = ch.category().map(|c| c.id) == Some(ChannelId(1232824647834140712));

  let Some((react, count)) = m
    .reactions
    .iter()
    .map(|r| (r.reaction_type.clone(), r.count))
    .max_by_key(|(_, c)| *c)
  else {
    return Ok(());
  };

  match get_post_id(m.id.0 as i64).await? {
    Some((p,)) => {
      webhook_msg(Some(p), format!("{react} x {count}")).await?.id.0;
    }
    None => {
      //if count > 1 {
        info!("sending message");
        match webhook_msg(None, "test".into()).await {
            Ok(r) => upsert_post(m.id.0 as i64, r.id.0 as i64).await?,
            Result::Err(e) => error!("{e}"),
        }
      //}
    }
  }
  info!("starboard_update2");
  Ok(())
}

#[derive(Serialize, Debug, Default)]
struct HookMsg {
  pub content: String,
  pub allowed_mentions: HashMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct HookRes {
  pub id: MessageId,
}

async fn webhook_msg(srcid: Option<i64>, text: String) -> Res<HookRes> {
  let mut hm: HashMap<String, Vec<String>> = HashMap::new();
  hm.insert("parse".into(), vec![]);

  Ok(
    req()
      .post(format!(
        "{}{}?wait=true",
        expect_env!("STARBOARD_HOOK"),
        srcid
          .map(|id| format!("/messages/{id}"))
          .unwrap_or("".into())
      ))
      .json(&HookMsg {
        content: text,
        allowed_mentions: hm,
        ..Default::default()
      })
      .send()
      .await?
      .json()
      .await?,
  )
}
