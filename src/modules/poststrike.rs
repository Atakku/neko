// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::sqlx::Postgres;
use crate::{core::*, modules::poise::Poise};
use poise::{
  serenity_prelude::{Context, GatewayIntents},
  BoxFuture, Event,
};
use crate::query::poststrike::*;

/// Module with femboy.tv discord server functionality
pub struct PostStrike;

impl Module for PostStrike {
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
      Message { new_message:  m } => {
        let user = m.author.id.0 as i64;
        let (prev_ts, strike) = get_strike(user).await?.unwrap_or((0, 0));
        let new_ts = m.timestamp.unix_timestamp();
        if new_ts < prev_ts + MIN_TRESH {
          // too early, do nothing
        } else if new_ts > prev_ts + MIN_TRESH && new_ts < prev_ts + MAX_TRESH  {
          // add one
          update_timestamp(user, new_ts, strike + 1).await?;
          m.reply(&c, format!("🔥 **Your strike is now at {}** 🔥", strike + 1)).await?;
        } else {
          // reset
          update_timestamp(user, new_ts, 1).await?;
          m.reply(&c, format!("🔥 **Your strike has been reset to 1** 🔥")).await?;
        }
      }
      _ => {}
    }
    Ok(())
  })
}
