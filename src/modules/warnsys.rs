// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::ops::Add;

use crate::{core::*, modules::poise::Poise, query::warnsys};
use chrono::{Utc, Duration};
use poise::serenity_prelude::{GuildId, RoleId, UserId};

const GUILD: GuildId = GuildId(1038789193113014333);
const ROLE: RoleId = RoleId(1040686138878341231);

/// Temporary shitcoded moderation module while v2 is still being written
pub struct WarnSystem;

impl Module for WarnSystem {
  fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>()?;
    poise.commands.push(warn());
    Ok(())
  }
}

#[poise::command(slash_command)]
async fn warn(ctx: crate::modules::poise::Ctx<'_>, user: UserId, reason: String) -> R {
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
    warnsys::add_user_warning(user.0 as i64, &reason).await?;
    let warns = warnsys::active_user_warnings(user.0 as i64).await?.len();
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
      (Duration::minutes(1), "1 minute")
    }
    1 => {
      (Duration::minutes(5), "5 minutes")
    }
    2 => {
      (Duration::minutes(30), "30 minutes")
    }
    3 => {
      (Duration::hours(6), "6 hours")
    }
    4 => {
      (Duration::days(1), "1 day")
    }
    5 => {
      (Duration::days(3), "3 days")
    }
    6 => {
      (Duration::days(6), "6 days")
    }
    _ => {
      (Duration::days(24), "24 days")
    }
  }
}