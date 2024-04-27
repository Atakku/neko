// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::{
  reqwest::{req, Reqwest},
  sqlx::Postgres,
};
use crate::{core::*, modules::poise::Poise, query::starboard::*};
use itertools::Itertools;
use log::{debug, error, info};
use poise::{
  serenity_prelude::{
    ChannelId, Context, GatewayIntents, GuildId, Message, MessageId, ReactionType, User,
  },
  BoxFuture, Event,
};
use reqwest::{Client, RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Module with femboy.tv discord server functionality
pub struct Starboard;

impl Module for Starboard {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>()?;
    fw.req_module::<Reqwest>()?;
    let poise = fw.req_module::<Poise>()?;
    poise
      .intents
      .insert(GatewayIntents::GUILD_MESSAGE_REACTIONS);
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

const CATEGORIES: &[(ChannelId, bool)] = &[
  (ChannelId(1232822999732719826), false), // chat
  (ChannelId(1232823234240577679), false), // gaming
  (ChannelId(1232821654200123392), false), // vc
  (ChannelId(1232824647834140712), true),  // nsfw
  (ChannelId(1232744932834410610), true),  // media - forum
  (ChannelId(1232744969996075079), true),  // nsfw - forum
];

async fn starboard_update<'a>(c: &'a Context, m: Message) -> Res<()> {
  let Some(ch) = m.channel(c).await?.guild() else {
    return Ok(());
  };
  let Some(cat) = ch.parent_id else {
    return Ok(());
  };
  let Some((_, spoiler)) = CATEGORIES.iter().find(|(id, _)| id == &cat) else {
    error!("cat not listed");
    return Ok(());
  };

  let Some((react, count)) = m
    .reactions
    .iter()
    .map(|r| (r.reaction_type.clone(), r.count))
    .max_by_key(|(_, c)| *c)
  else {
    return Ok(());
  };

  let mut msg = format!(
    "**{count}x** {react} in https://discord.com/channels/1232659990993702943/{}/{}\n",
    m.channel_id, m.id
  );
  if *spoiler {
    msg += "||"
  }
  if m.content != "" {
    msg += &m.content.replace("||", "");
    if m.attachments.len() > 0 {
      msg += "\n";
    }
  }
  msg += &m
      .attachments
      .iter()
      .take(5)
      .map(|a| {
        let mut att = String::new();
        if a.filename.contains("SPOILER") && !*spoiler {
          att += "||"
        }
        att += &format!("[{}]({})", a.filename, a.url.clone()).replace("||", "");

        if a.filename.contains("SPOILER") && !*spoiler {
          att += "||"
        }
        att
      })
      .join(" ");

  if *spoiler {
    msg += " ||"
  }

  match get_post_id(m.id.0 as i64).await? {
    Some((p,)) => {
      edit_post(p, msg, m.author).await?;
    }
    None => {
      if count >= 5 {
        let id = new_post(msg, m.author).await?.0;
        upsert_post(m.id.0 as i64, id as i64).await?;
      }
    }
  }
  Ok(())
}

#[derive(Serialize, Debug, Default)]
struct HookMsg {
  pub content: String,
  pub allowed_mentions: HashMap<String, Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub avatar_url: Option<String>,
  pub username: String,
}

async fn send(
  method: fn(&Client, String) -> RequestBuilder,
  suffix: String,
  text: String,
  u: User,
) -> Res<Response> {
  let mut hm: HashMap<String, Vec<String>> = HashMap::new();
  hm.insert("parse".into(), vec![]);

  Ok(
    (method)(req(), format!("{}{suffix}", expect_env!("STARBOARD_HOOK"),))
      .json(&HookMsg {
        content: text,
        allowed_mentions: hm,
        avatar_url: u.avatar_url(),
        username: u.name,
        ..Default::default()
      })
      .send()
      .await?,
  )
}

#[derive(Deserialize)]
struct HookRes {
  pub id: MessageId,
}

async fn new_post(text: String, u: User) -> Res<MessageId> {
  Ok(
    send(Client::post, "?wait=true".into(), text, u)
      .await?
      .json::<HookRes>()
      .await?
      .id,
  )
}

async fn edit_post(id: i64, text: String, u: User) -> Res<()> {
  send(Client::patch, format!("/messages/{id}"), text, u)
    .await?
    .text()
    .await?;
  Ok(())
}
