// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{core::*, modules::poise::Poise};
use poise::{
  serenity_prelude::{ButtonStyle, EmojiId, Interaction::MessageComponent, ReactionType, RoleId},
  BoxFuture, Event,
};

/// Module with femboy.tv discord server functionality
pub struct FTVRoles;

impl Module for FTVRoles {
  async fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>().await?;
    poise.event_handlers.push(event_handler);
    poise.commands.push(spawn_roles());
    Ok(())
  }
}

fn event_handler<'a>(
  c: &'a poise::serenity_prelude::Context,
  event: &'a Event<'a>,
) -> BoxFuture<'a, R> {
  Box::pin(async move {
    use Event::*;
    match event {
      InteractionCreate { interaction } => {
        let MessageComponent(i) = interaction else {
          return Ok(());
        };
        let Some(guild_id) = &i.guild_id else {
          return Ok(());
        };

        let all: Vec<_> = ROLES.iter().flat_map(|r| r.1).map(|r| r.0).collect();
        let Ok(id) = i.data.custom_id.parse::<u64>() else {
          return Ok(());
        };

        if all.contains(&id) {
          i.defer_ephemeral(c).await?;
          let Ok(mut m) = guild_id.member(c, i.user.id).await else {
            return Err("Failed to get member".into());
          };
          let role: RoleId = RoleId::from(id);

          let mut msg = String::new();
          if m.roles.contains(&role) {
            m.remove_role(&c, role).await?;
            msg = format!("**Removed role:** <@&{id}>");
          } else {
            m.add_role(&c, role).await?;
            msg = format!("**Added role:** <@&{id}>");
          }

          i.edit_original_interaction_response(c, |r| r.content(msg))
            .await?;
        }
      }
      _ => {}
    }
    Ok(())
  })
}

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn spawn_roles(ctx: crate::modules::poise::Ctx<'_>) -> R {
  for group in ROLES {
    let rows: Vec<_> = group.1.chunks(5).map(|row| row.to_vec()).collect();

    if rows.len() == 0 {
      ctx
        .send(|b| b.content(group.0))
        .await?
        .into_message()
        .await?;
    }
    for (i, msg) in rows.chunks(5).enumerate() {
      ctx
        .send(|b| {
          if i == 0 {
            b.content(group.0);
          }

          b.components(|b| {
            for row in msg {
              b.create_action_row(|b| {
                for role in row {
                  b.create_button(|b| {
                    b.custom_id(role.0.to_string())
                      .emoji({
                        if role.2 {
                          EmojiId(role.1.parse().unwrap_or(1233072462527332363)).into()
                        } else {
                          ReactionType::Unicode(role.1.to_string())
                        }
                      })
                      .style(ButtonStyle::Secondary);
                    if role.3 != "" {
                      b.label(role.3);
                    }
                    b
                  });
                }
                b
              });
            }

            b
          });

          b
        })
        .await?
        .into_message()
        .await?;
    }
  }
  Ok(())
}

const ROLES: &[(&str, &[(u64, &str, bool, &str)])] = &[
  (
    "# Notification Roles:\n\
    **📢 - Announcements** (Updates, rule changes, new cool things, etc)\n\
    **📅 - Events** (New event, starting event)\n\
    **📰 - News** (A big game update, free to keep or huge sale, or other non-server news)\n\
    **💩 - Shitpost** (For funny, or not so funny things)\n\
    **🍿 - Movie Night** (Pinged whenever we do a movie night :3)\n\
    **👋 - Welcome** (Get notified every time someone gets verified)\n\
    **🔌 - Pingplug** (Free for all, take at your own risk)",
    &[
      (1233042177408368652, "📢", false, ""),
      (1233042176217190450, "📅", false, ""),
      (1233042179191210024, "📰", false, ""),
      (1233042174904373248, "💩", false, ""),
      (1342096453425823805, "🍿", false, ""),
      (1277279750469062667, "👋", false, ""),
      (1233042180252237834, "🔌", false, ""),
    ],
  ),
  (
    "# Interest roles:",
    &[
      (1233042183439777884, "🌸", false, "Femboys"),
      (1233047149063962675, "💻", false, "Programming"),
      (1233042181309206629, "🎮", false, "Gaming"),
      (1233042182647320706, "🎨", false, "Drawing"),
      (1233047520700006501, "🎵", false, "Music"),
    ],
  )
];
