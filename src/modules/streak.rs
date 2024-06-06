// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::sqlx::Postgres;
use crate::{core::*, modules::poise::Poise, query::streak::*};
use poise::{
  serenity_prelude::{ChannelId, Context, GatewayIntents},
  BoxFuture, Event,
};

/// Module with femboy.tv discord server functionality
pub struct PostStreak;

impl Module for PostStreak {
  async fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>().await?;
    let poise = fw.req_module::<Poise>().await?;
    poise.intents.insert(GatewayIntents::GUILD_MESSAGES);
    poise.event_handlers.push(event_handler);
    poise.commands.push(streak());
    Ok(())
  }
}

const MIN_TRESH: i64 = 18 * 60 * 60;
const MAX_TRESH: i64 = 36 * 60 * 60;

use crate::modules::poise::Ctx;
#[poise::command(prefix_command, slash_command)]
pub async fn streak(ctx: Ctx<'_>) -> R {
  let (prev_ts, streak) = get_streak(ctx.author().id.0 as i64)
    .await?
    .unwrap_or((0, 0));
  let new_ts = ctx.created_at().unix_timestamp();
  let diff = new_ts - prev_ts;
  let msg = if diff < MIN_TRESH {
    format!("Your streak is {streak} and you need to wait {} to continue your streak", nicefmt(MIN_TRESH - diff))
  } else if diff >= MIN_TRESH && diff < MAX_TRESH {
    format!("Your streak is {streak} and you have {} left to continue your streak", nicefmt(MAX_TRESH - diff))
  } else {
    if prev_ts == 0 {
      format!("You haven't started a streak yet!")
    } else {
      format!("Your streak of {streak} expired {} ago", nicefmt(diff - MAX_TRESH))
    }
  };
  ctx.send(|m| m.ephemeral(true).reply(true).content(msg)).await?;
  Ok(())
}

fn nicefmt(raw_secs: i64) -> String {
  let mut out = String::new();
  let secs = raw_secs % 60;
  let raw_mins = raw_secs / 60;
  let mins = raw_mins % 60;
  let hours = raw_mins / 60;
  if hours > 0 {
    out += &format!("{hours}h ");
  }
  if hours > 0 || mins > 0 {
    out += &format!("{mins}m ");
  }
  out += &format!("{secs}s ");
  return out;
}

fn event_handler<'a>(c: &'a Context, event: &'a Event<'a>) -> BoxFuture<'a, R> {
  Box::pin(async move {
    use Event::*;
    match event {
      Message { new_message: m } => {
        if m.channel_id != ChannelId(1232829261279264829) || m.author.bot {
          return Ok(());
        }
        if m
          .channel(c)
          .await?
          .guild()
          .unwrap()
          .message(c, m)
          .await?
          .attachments
          .len()
          == 0
        {
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
          if prev_ts == 0 {
            m.reply(&c, format!("🔥 **You just started your first streak!** 🔥\nYou can post again in 18-36 hours to increase your streak.\nYou can also check your streak with the /streak command"))
            .await?;
          } else {
            m.reply(&c, format!("🔥 **Your streak has been reset to 1** 🔥"))
              .await?;
          }
        }
      }
      _ => {}
    }
    Ok(())
  })
}
