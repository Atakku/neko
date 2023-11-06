// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::plugins::{
  discord_cache::discord_guilds,
  steam::{
    handle,
    query::{At, By, Of},
    steam_apps,
  },
};
use poise::serenity_prelude::UserId;

lim_choice!(GuildTop, Of, [Users, Apps]);
lim_choice!(AppTop, Of, [Users, Guilds]);

cmd_group!(steam, "user", "app", "guild", "top");

cmd_group!(user, "user_top");
cmd_group!(app, "app_top");
cmd_group!(guild, "guild_top");

commands! {
  fn top(ctx, by: By, of: Of) -> BasicCommand {
    handle(ctx, format!("Top of {of} by {by}"), of, by, At::None).await?;
  }

  fn user_top(ctx, by: By, user: Option<UserId>) -> BasicCommand {
    let user = user.unwrap_or(ctx.author().id);
    let of = Of::Apps;
    let title = format!("Users's ({user}) top of {of} by {by}");
    handle(ctx, title, of, by, At::User(user.0 as i64)).await?;
  }

  fn app_top(ctx, by: By, of: AppTop, #[autocomplete = "steam_apps"] app: i32) -> BasicCommand {
    let title = format!("Apps's ({app}) top of {of} by {by}");
    handle(ctx, title, of.into(), by, At::App(app)).await?;
  }

  fn guild_top(ctx, by: By, of: GuildTop, #[autocomplete = "discord_guilds"] guild: Option<String>) -> BasicCommand {
    let guild = guild.unwrap_or(ctx.guild_id().unwrap_or(crate::plugins::ftv::GUILD).0.to_string());
    let title = format!("Guild's ({guild}) top of {of} by {by}");
    handle(ctx, title, of.into(), by, At::Guild(guild.parse::<i64>()?)).await?;
  }
}
