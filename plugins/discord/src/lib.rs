// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use neko_core::*;
use neko_poise::PoiseModule;
use neko_sqlx::SqlxModule;
use poise::serenity_prelude::GatewayIntents;
use sqlx::Postgres;

mod event;
mod schema;

pub struct DiscordPlugin {}

impl Module for DiscordPlugin {
  fn init(&self, fw: &mut Framework) -> R {
    fw.req_module::<SqlxModule<Postgres>>()?;
    let poise = fw.req_module::<PoiseModule>()?;
    poise.event_handlers.push(event::event_handler());
    poise
      .intents
      .insert(GatewayIntents::GUILDS | GatewayIntents::GUILD_MEMBERS);
    Ok(())
  }
}
