// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::ops::Add;

use crate::{core::*, modules::poise::Poise};
use chrono::{Utc, Duration};
use itertools::Itertools;
use poise::serenity_prelude::{GuildId, RoleId, UserId};

const GUILD: GuildId = GuildId(1232659990993702943);
const ROLE: RoleId = RoleId(1232821172983562333);

pub mod query;
pub mod schema;

/// Temporary shitcoded moderation module while v2 is still being written
pub struct WarnSystem;

impl Module for WarnSystem {
  async fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>().await?;
    poise.commands.push(warn());
    poise.commands.push(rm_warn());
    poise.commands.push(warns());
    Ok(())
  }
}

#[poise::command(slash_command)]
async fn warns(ctx: crate::modules::poise::Ctx<'_>, user: UserId) -> R {
  if ctx.guild_id() != Some(GUILD) {
    ctx.reply("This command is only permitted in femboy.tv").await?;
    return Ok(())
  }
  let mut warns = query::active_user_warnings(user.0 as i64).await?;
  warns.sort_by_key(|(_, x, _)| x.clone());
  let content = warns.iter().map(|(id, ts, res)| format!("{id} | {ts} | {res}")).join("\n");
  ctx.reply(format!("```\n{content}\n```")).await?;
  Ok(())
}

#[poise::command(slash_command)]
async fn rm_warn(ctx: crate::modules::poise::Ctx<'_>, id: String) -> R {
  if ctx.guild_id() != Some(GUILD) {
    ctx.reply("This command is only permitted in femboy.tv").await?;
    return Ok(())
  }
  if let Some(m) = ctx.author_member().await {
    let roles = GUILD.member(ctx, m.user.id).await?.roles;
    if !roles.contains(&ROLE) {
      ctx.reply("You are not a moderator").await?;
      return Ok(())
    }
    query::rm_warn(id.parse()?).await?;
    ctx.reply(format!("Removed warn with id {id}")).await?;
  }
  Ok(())
}

#[poise::command(slash_command)]
async fn warn(ctx: crate::modules::poise::Ctx<'_>, user: UserId, reason: String, ) -> R {
  if ctx.guild_id() != Some(GUILD) {
    ctx.reply("This command is only permitted in femboy.tv").await?;
    return Ok(())
  }
  if let Some(m) = ctx.author_member().await {
    let roles = GUILD.member(ctx, m.user.id).await?.roles;
    if !roles.contains(&ROLE) {
      ctx.reply("You are not a moderator").await?;
      return Ok(())
    }
    query::add_user_warning(user.0 as i64, &reason).await?;
    let warns = query::active_user_warnings(user.0 as i64).await?.len();
    let (time, _) = match_warn(warns);
    let (_, future) = match_warn(warns + 1);
    let time = Utc::now().add(time);
    let ts = time.timestamp();
    ctx.reply(format!("**Warned** <@{}> with `{reason}`\nThey are now at **{warns} warnings**, and timed out until <t:{ts}>\nA future timeout will last for **{future}**\nThey will be able to speak again <t:{ts}:R>", user.0)).await?;
    GUILD.member(ctx, user).await?.disable_communication_until_datetime(ctx, time.into()).await?;
  }
  Ok(())
}

fn match_warn<'a>(warns: usize) -> (Duration, &'a str) {
  match warns {
    0 => {// Should never happen, but what if something goes wrong
      (Duration::try_minutes(1).unwrap(), "1 minute")
    }
    1 => {
      (Duration::try_days(1).unwrap(), "1 day")
    }
    2 => {
      (Duration::try_days(7).unwrap(), "1 week")
    }
    _ => {
      (Duration::try_days(24).unwrap(), "24 days")
    }
  }
}