// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  modules::{poise::Poise, sqlx::Postgres},
  plugins::{findr::query::*, neko::autocomplete::findr_cities},
};
use geoutils::Location;
use itertools::Itertools;
use poise::{serenity_prelude::GuildId, ReplyHandle};

pub mod query;
pub mod schema;

/// Module with femboy.tv discord server functionality
pub struct Findr;

impl Module for Findr {
  async fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>().await?;
    let poise = fw.req_module::<Poise>().await?;
    poise.commands.push(findr());
    Ok(())
  }
}

use crate::modules::poise::Ctx;

cmd_group!(findr, "find", "set");

pub async fn ephrep<'a>(ctx: Ctx<'a>, text: &'a str) -> Res<ReplyHandle<'a>> {
  Ok(
    ctx
      .send(|m| m.ephemeral(true).reply(true).content(text))
      .await?,
  )
}

#[poise::command(slash_command)]
pub async fn find(ctx: Ctx<'_>) -> R {
  let Some(guild_id) = ctx.guild_id() else {
    ephrep(ctx, "This command is not available outside femboy.tv!").await?;
    return Ok(());
  };

  if guild_id != GuildId(1232659990993702943) {
    ephrep(ctx, "This command is not available outside femboy.tv!").await?;
    return Ok(());
  }

  //guild_id.members_iter(&ctx).filter(|m| m.unwrap().avatar_url()).collect();
  let mut data = get_all().await?;
  let Some((pos, (_, lat, lng, _name))) = data
    .clone()
    .into_iter()
    .find_position(|(id, _lat, _lng, _name)| *id == ctx.author().id.0 as i64)
  else {
    ctx
      .send(|m| {
        m.ephemeral(true)
          .reply(true)
          .content("You did not set your city, use /findr set")
      })
      .await?;
    return Ok(());
  };
  data.remove(pos);
  let origin = Location::new(lat, lng);

  let data = data
    .into_iter()
    .map(|(id, lat, lng, name)| {
      origin
        .distance_to(&Location::new(lat, lng))
        .and_then(|dist| Ok((id, (dist.meters() / 1000.0) as i64, name)))
    })
    .filter_map(Result::ok)
    .sorted_by_key(|(_, dist, _)| *dist)
    .collect::<Vec<_>>();

  let mut output = String::new();
  for (id, dist, name) in data.iter().take(25) {
    output += &format!("{dist}km | <@{id}> | {name}\n");
  }

  ctx
    .send(|m| m.ephemeral(true).reply(true).content(output))
    .await?;
  Ok(())
}

#[poise::command(slash_command)]
pub async fn set(ctx: Ctx<'_>, #[autocomplete = "findr_cities"] city_id: String) -> R {
  if ctx.guild_id() != Some(GuildId(1232659990993702943)) {
    ctx
      .send(|m| {
        m.ephemeral(true)
          .reply(true)
          .content("This command is not available outside femboy.tv!")
      })
      .await?;
    return Ok(());
  }
  update_city(ctx.author().id.0 as i64, city_id.parse().unwrap()).await?;
  ctx
    .send(|m| m.ephemeral(true).reply(true).content("Updated your city!"))
    .await?;
  Ok(())
}
