// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  modules::poise::{Ctx, Poise},
  plugins::steam::{filter_roles, get_roles, minor_update},
};
use futures::StreamExt;

module! {
  // Util module for maintenance commands
  Atakku;

  fn init(fw) {
    let poise = fw.req::<Poise>()?;
    poise.add_commands(vec![register_commands(), update_steam(), update_roles()]);
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
async fn update_roles(ctx: Ctx<'_>) -> R {
  let m = ctx.reply("Updating steam roles...").await?;

  if let Some(g) = ctx.guild_id() {
    let mut members = g.members_iter(&ctx).boxed();
    while let Some(member_result) = members.next().await {
      match member_result {
        Ok(mut m) => {
          if !m.user.bot {
            let roles = filter_roles(m.roles(ctx).unwrap_or_default(), get_roles(&m).await?);
            if !roles.is_empty() {
              m.add_roles(ctx, roles.as_slice()).await?;
            }
          }
        }
        _ => {}
      }
    }
  }

  m.edit(ctx, |m| m.content("Done updating steam roles!"))
    .await?;
  Ok(())
}
