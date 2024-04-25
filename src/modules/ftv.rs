// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{core::*, modules::poise::Poise};
use poise::{
  serenity_prelude::{ChannelId, Colour, GuildId, User},
  BoxFuture, Event,
};

pub const GUILD: GuildId = GuildId(1038789193113014333);
const GENERAL: ChannelId = ChannelId(1178857392033759262);

/// Module with femboy.tv discord server functionality
pub struct FemboyTV;

impl Module for FemboyTV {
  fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>()?;
    poise.event_handlers.push(welcomer);
    Ok(())
  }
}

fn welcomer<'a>(c: &'a poise::serenity_prelude::Context, event: &'a Event<'a>) -> BoxFuture<'a, R> {
  Box::pin(async move {
    use Event::*;
    match event {
      GuildMemberAddition { new_member: m } => {
        if !m.user.bot && m.guild_id == GUILD {
          let u = &m.user;
          GENERAL
            .send_message(c, |m| {
              m.embed(|e| {
                e.author(|a| {
                  a.icon_url(get_avatar(&u));
                  a.name(get_name(&u));
                  a.url(format!("https://discord.com/users/{}", u.id))
                });
                e.colour(Colour::from_rgb(139, 195, 74));
                e.description(format!("Welcome <@{}> to the server!", u.id))
              })
            })
            .await?;
        }
      }

      GuildMemberRemoval {
        guild_id: g,
        user: u,
        member_data_if_available: _,
      } => {
        if !u.bot && *g == GUILD {
          GENERAL
            .send_message(c, |m| {
              m.embed(|e| {
                e.author(|a| {
                  a.icon_url(get_avatar(&u));
                  a.name(get_name(&u));
                  a.url(format!("https://discord.com/users/{}", u.id))
                });
                e.colour(Colour::from_rgb(244, 67, 54));
                e.description(format!("<@{}> has left the server!", u.id))
              })
            })
            .await?;
        }
      }
      _ => {}
    }
    Ok(())
  })
}

fn get_avatar(u: &User) -> String {
  if let Some(avatar) = u.avatar_url() {
    return avatar;
  }
  u.default_avatar_url()
}

fn get_name(u: &User) -> String {
  if u.discriminator != 0 {
    format!("{}#{:0>4}", u.name, u.discriminator)
  } else {
    u.name.clone()
  }
}
