// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use futures::StreamExt;

use super::poise::Poise;
use crate::{
  core::*,
  modules::{poise::Ctx, steam::{minor_update, get_roles, filter_roles}, beatleader::update_scores},
};

// Util module for maintenance commands
pub struct Atakku;

impl Module for Atakku {
  fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>()?;
    poise.commands.push(register_commands());
    poise.commands.push(update_steam());
    poise.commands.push(update_roles());
    poise.commands.push(update_beatleader());
    Ok(())
  }
}

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn register_commands(ctx: Ctx<'_>) -> R {
  poise::samples::register_application_commands_buttons(ctx).await?;
  Ok(())
}

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn update_steam(ctx: Ctx<'_>) -> R {
  let m = ctx.reply("Updating steam data...").await?;

  minor_update().await?;

  m.edit(ctx, |m| m.content("Done updating steam data!"))
    .await?;
  Ok(())
}

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn update_beatleader(ctx: Ctx<'_>) -> R {
  let m = ctx.reply("Updating beatleader data...").await?;

  update_scores().await?;

  m.edit(ctx, |m| m.content("Done updating beatleader data!"))
    .await?;
  Ok(())
}

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn update_roles(ctx: Ctx<'_>) -> R {
  let m = ctx.reply("Updating steam roles...").await?;

  if let Some(g) = ctx.guild_id() {
    let mut members = g.members_iter(&ctx).boxed();
    while let Some(member_result) = members.next().await {
      match member_result {
        Ok(mut m) => if !m.user.bot {
          let roles = filter_roles(
            m.roles(ctx).unwrap_or_default(),
            get_roles(&m).await?,
          );
          if !roles.is_empty() {
            m.add_roles(ctx, roles.as_slice()).await?;
          }
        },
        _ => {}
      }
    }
  }

  m.edit(ctx, |m| m.content("Done updating steam roles!"))
    .await?;
  Ok(())
}
