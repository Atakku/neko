// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{core::*, modules::poise::Poise};
use poise::{
  serenity_prelude::{
    ButtonStyle, ChannelId, Colour, EmojiId, GuildId, InteractionResponseType, ReactionType,
    RoleId, User,
  },
  BoxFuture, Event,
};

pub const GUILD: GuildId = GuildId(1232659990993702943);
const GENERAL: ChannelId = ChannelId(1232666862148653147);

/// Module with femboy.tv discord server functionality
pub struct NewFemboyTV;

impl Module for NewFemboyTV {
  fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>()?;
    poise.event_handlers.push(welcomer);
    poise.commands.push(new_spawn_roles());
    Ok(())
  }
}

const ROLES: &[(&str, &[&[(u64, &str, &str, bool)]])] = &[
  (
    "# Pick a badge role:",
    &[
      &[(1232717136770891868, "", "🏳️‍⚧️", false)],
      &[(1232717140285718590, "", "🫑", false)],
      &[(1232717142785523885, "", "🦀", false)],
      &[(1232717144702320692, "", "🌸", false)],
      &[(1232717147147472957, "", "🗿", false)],
    ],
  ),
  (
    "# Pick your country role:",
    &[
      &[
        (1232718028802883647, "<@&1232718028802883647>", "🇩🇪", false),
        (1232718029398474804, "Russia", "🇷🇺", false),
        (1232718030782333009, "United Kingdom", "🇬🇧", false),
      ],
      &[(1232718031617265805, "Poland", "🇵🇱", false)],
    ],
  ),
];

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn new_spawn_roles(ctx: crate::modules::poise::Ctx<'_>) -> R {
  for category in ROLES {
    ctx
      .send(|b| {
        b.content(category.0).components(|b| {
          for row in category.1 {
            b.create_action_row(|b| {
              for but in *row {
                b.create_button(|b| {
                  b.custom_id(but.0)
                    .emoji({
                      if but.3 {
                        EmojiId(but.2.parse().unwrap_or(1049347516346400858)).into()
                      } else {
                        ReactionType::Unicode(but.2.to_string())
                      }
                    })
                    .style(ButtonStyle::Secondary);

                  if but.1 != "" {
                    b.label(but.1);
                  }

                  b
                });
              }
              b
            });
          }

          b
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
      InteractionCreate { interaction } => {
        if let poise::serenity_prelude::Interaction::MessageComponent(i) = interaction {
          if let Some(g) = &i.guild_id {
            let roles: Vec<(u64, &str, &str, bool)> = ROLES
              .iter()
              .flat_map(|x| x.1)
              .map(|a| a.to_vec())
              .flatten()
              .collect();
            let role_ids: Vec<u64> = roles.into_iter().map(|r| r.0).collect();
            let id: u64 = i.data.custom_id.parse().unwrap();
            if role_ids.contains(&id) {
              if let Ok(mut m) = g.member(c, i.user.id).await {
                i.defer(c).await?;

                let role: RoleId = RoleId::from(id);

                let mut msg = String::new();
                if m.roles.contains(&role) {
                  m.remove_role(&c, role).await?;
                  msg = format!("**Removed role:** <@&{id}>");
                } else {
                  m.add_role(&c, role).await?;
                  msg = format!("**Added role:** <@&{id}>");
                }
                i.create_interaction_response(c, |a| {
                  a.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|d| d.ephemeral(true).content(msg))
                })
                .await?;
              }
            }
          }
        }
      }
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
