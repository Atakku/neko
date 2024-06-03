// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::sqlx::Postgres;
use crate::query::streak::*;
use crate::{core::*, modules::poise::Poise};
use poise::{
  serenity_prelude::{ChannelId, Context, GatewayIntents},
  BoxFuture, Event,
};

/// Module with femboy.tv discord server functionality
pub struct PostStreak;

impl Module for PostStreak {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>()?;
    let poise = fw.req_module::<Poise>()?;
    poise.intents.insert(GatewayIntents::GUILD_MESSAGES);
    poise.event_handlers.push(event_handler);
    Ok(())
  }
}

const MIN_TRESH: i64 = 18 * 60 * 60;
const MAX_TRESH: i64 = 36 * 60 * 60;

fn event_handler<'a>(c: &'a Context, event: &'a Event<'a>) -> BoxFuture<'a, R> {
  Box::pin(async move {
    use Event::*;
    match event {
      Message { new_message: m } => {
        if m.channel_id != ChannelId(1232829261279264829) || m.author.bot {
          return Ok(());
        }
        if m.attachments.len() == 0 {
          return Ok(());
        }
        let user = m.author.id.0 as i64;
        let (prev_ts, streak) = get_streak(user).await?.unwrap_or((0, 0));
        let new_ts = m.timestamp.unix_timestamp();
        let diff = new_ts - prev_ts;
        if diff < MIN_TRESH {
          // too early, do nothing
          m.reply(
            &c,
            format!("🔥 **You posted again too early, streak is still at {streak}** 🔥"),
          )
          .await?;
        } else if diff >= MIN_TRESH && diff < MAX_TRESH {
          // add one
          update_timestamp(user, new_ts, streak + 1).await?;
          m.reply(
            &c,
            format!("🔥 **Your streak is now at {}** 🔥", streak + 1),
          )
          .await?;
        } else {
          // reset
          update_timestamp(user, new_ts, 1).await?;
          m.reply(&c, format!("🔥 **Your streak has been reset to 1** 🔥"))
            .await?;
        }
      }
      _ => {}
    }
    Ok(())
  })
}
