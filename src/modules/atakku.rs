// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::poise::Poise;
use crate::{
  core::*,
  modules::poise::Ctx,
  query::{
    neko::all_steam_connections,
    steam::{update_playdata, update_users},
  },
};

// Util module for maintenance commands
pub struct Atakku;

impl Module for Atakku {
  fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>()?;
    poise.commands.push(register_commands());
    poise.commands.push(update_steam());
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

  let conns = all_steam_connections().await?;
  update_users(&conns).await?;
  update_playdata(&conns).await?;

  m.edit(ctx, |m| m.content("Done updating steam data!"))
    .await?;
  Ok(())
}
