// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{core::*, modules::poise::Poise};
use poise::{
  serenity_prelude::{ChannelId, Colour, GuildId, ReactionType, User},
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
    poise.commands.push(spawn_roles());
    Ok(())
  }
}

const ROLES: &[(&str, &[(u64, &str, &str)])] = &[
  (
    "Pick your color role:",
    &[(1122082509493121084, "Blossom", "🌸")],
  ),
  (
    "Pick your country role:",
    &[
      (1062671646915297330, "United Kingdom", "🇬🇧"),
      (1062671650342060053, "Netherlands", "🇳🇱"),
      (1062671867015610388, "Italy", "🇮🇹"),
      (1062671639436865557, "Spain", "🇪🇸"),
      (1062671151903547432, "Russia", "🇷🇺"),
      (1062671883935428628, "Serbia", "🇷🇸"),
      (1062671879015497789, "France", "🇫🇷"),
      (1123962805922562098, "United States", "🇺🇸"),
      (1123962799958282360, "Germany", "🇩🇪"),
      (1123962798616096818, "Bosnia & Herzegovina", "🇧🇦"),
      (1123962803317903380, "Poland", "🇵🇱"),
      (1123962795692671008, "Portugal", "🇵🇹"),
      (1123962807155695646, "Denmark", "🇩🇰"),
      (1123962810922180648, "Turkey", "🇹🇷"),
      (1123962797458468924, "Czechia", "🇨🇿"),
      (1123962809328353440, "Lithuania", "🇱🇹"),
      (1123962804601360384, "Canada", "🇨🇦"),
      (1123962802298704014, "Ireland", "🇮🇪"),
    ],
  ),
];

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn spawn_roles(ctx: crate::modules::poise::Ctx<'_>) -> R {
  for category in ROLES {
    ctx
      .send(|b| {
        b.content(category.0).components(|b| {
          b.create_action_row(|b| {
            b.create_select_menu(|m| {
              m.options(|f| {
                let mut f = f;
                for role in category.1 {
                  f = f.create_option(|o| {
                    o.emoji(ReactionType::Unicode(role.2.to_string()))
                      .label(role.1)
                      .value(role.0)
                  });
                }
                f
              })
            })
          })
        })
      })
      .await?
      .into_message()
      .await?;
  }
  Ok(())
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
