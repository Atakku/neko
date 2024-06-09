// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::sqlx::Postgres;
use crate::query::autocomplete::finder_cities;
use crate::{core::*, modules::poise::Poise, query::finder::*};

/// Module with femboy.tv discord server functionality
pub struct Finder;

impl Module for Finder {
  async fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>().await?;
    let poise = fw.req_module::<Poise>().await?;
    poise.commands.push(finder());
    Ok(())
  }
}

use crate::modules::poise::Ctx;

cmd_group!(finder, "find", "set");

#[poise::command(slash_command)]
pub async fn find(ctx: Ctx<'_>) -> R {
  ctx
    .send(|m| m.ephemeral(true).reply(true).content("work in poggers"))
    .await?;
  Ok(())
}

#[poise::command(slash_command)]
pub async fn set(ctx: Ctx<'_>, #[autocomplete = "finder_cities"] city_id: String) -> R {
  update_city(ctx.author().id.0 as i64, city_id.parse().unwrap()).await?;
  ctx
    .send(|m| m.ephemeral(true).reply(true).content("Updated your city!"))
    .await?;
  Ok(())
}
