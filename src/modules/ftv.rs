// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use itertools::Itertools;
use crate::{core::*, modules::poise::Poise};
use poise::{
  serenity_prelude::{ChannelId, Colour, GuildId, ReactionType, User, RoleId, InteractionResponseType, EmojiId},
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

const ROLES: &[(&str, &str, &[(u64, &str, &str, bool)])] = &[
  (
    "pick_badge", "# Pick your badge roles:",
    &[
      (1142188267643600907, "🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️🏳️‍⚧️", "🏳️‍⚧️", false),
      (1142188265835868244, "🫑🫑🫑🫑🫑🫑🫑🫑🫑🫑🫑", "🫑", false),
    ],
  ),
  (
    "pick_color", "# Pick your color role:",
    &[
      (1122082509493121084, "Blossom", "🌸", false),
      (1122082527956439081, "Carnation", "🌺", false),
      (1122082529797734420, "Watermelon", "🍉", false),
      (1122082515356745808, "Apricot", "🍑", false),
      (1122082535032225866, "Chocolate", "🍫", false),
      (1122082536101777412, "Tangerine", "🍊", false),
      (1122082579256983623, "Amber", "🔥", false),
      (1122082516464042068, "Sunny", "☀️", false),
      (1122082533329354762, "Sunflower", "🌻", false),
      (1122082527297941544, "Creamy", "🍦", false),
      (1122082522277351485, "Lime", "🍃", false),
      (1122082518473121802, "Mint", "🌿", false),
      (1122082531630661743, "Teal", "🧪", false),
      (1122082519580413953, "Turquoise", "💎", false),
      (1122082520910012487, "Oceanic", "🌊", false),
      (1122082537922109440, "Sky", "☁️", false),
      (1122082526299688961, "Royal", "👑", false),
      (1122082523615346698, "Lavender", "🌸", false),
      (1122082524944945203, "Grape", "🍇", false),
      (1122082581895184404, "Cherry", "🍒", false),
    ],
  ),
  (
    "pick_country", "# Pick your country roles:",
    &[
      (1062671646915297330, "United Kingdom", "🇬🇧", false),
      (1062671650342060053, "Netherlands", "🇳🇱", false),
      (1062671867015610388, "Italy", "🇮🇹", false),
      (1062671639436865557, "Spain", "🇪🇸", false),
      (1062671151903547432, "Russia", "🇷🇺", false),
      (1062671883935428628, "Serbia", "🇷🇸", false),
      (1062671879015497789, "France", "🇫🇷", false),
      (1123962805922562098, "United States", "🇺🇸", false),
      (1123962799958282360, "Germany", "🇩🇪", false),
      (1123962798616096818, "Bosnia & Herzegovina", "🇧🇦", false),
      (1123962803317903380, "Poland", "🇵🇱", false),
      (1123962795692671008, "Portugal", "🇵🇹", false),
      (1123962807155695646, "Denmark", "🇩🇰", false),
      (1123962810922180648, "Turkey", "🇹🇷", false),
      (1123962797458468924, "Czechia", "🇨🇿", false),
      (1123962809328353440, "Lithuania", "🇱🇹", false),
      (1123962804601360384, "Canada", "🇨🇦", false),
      (1123962802298704014, "Ireland", "🇮🇪", false),
    ],
  ),
  (
    "pick_interest", "# Pick your interest roles:",
    &[
      (1123962812276936724, "Femboys", "🌸", false),
      (1123962816949391360, "Programming", "💻", false),
      (1123962819197554789, "Gaming", "🎮", false),
      (1123962821454086175, "Drawing", "🎨", false),
    ],
  ),
  //(
  //  "pick_hmd", "# Pick your VR headset roles:",
  //  &[
  //    (1041462150297825351, "No HMD", "❌", false),
  //  ],
  //),
  //(
  //  "pick_fbt", "# Pick your VR FBT roles:",
  //  &[
  //    (1124283639514026097, "No FBT", "❌", false),
  //  ],
  //),
];

#[poise::command(prefix_command, hide_in_help, owners_only)]
async fn spawn_roles(ctx: crate::modules::poise::Ctx<'_>) -> R {
  for category in ROLES {
    ctx
      .send(|b| {
        b.content(category.1).components(|b| {
          b.create_action_row(|b| {
            b.create_select_menu(|m| {
              m.custom_id(category.0).min_values(0).max_values(category.2.len().min(25) as u64).options(|f| {
                let mut f = f;
                for role in category.2 {
                  f = f.create_option(|o| {
                    o.emoji({
                      if role.3 {
                        EmojiId(role.2.parse().unwrap_or(1049347516346400858)).into()
                      } else {  
                        ReactionType::Unicode(role.2.to_string())
                      }
                    })
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
      InteractionCreate { interaction } => {
        if let poise::serenity_prelude::Interaction::MessageComponent(i) = interaction {
          if let Some(g) = &i.guild_id {
            for r in ROLES {
              if i.data.custom_id == r.0 {
                if let Ok(mut m) = g.member(c, i.user.id).await {
                  i.defer(c).await?;
                  let all_roles: Vec<RoleId> = r.2.iter().map(|rr| RoleId::from(rr.0)).collect();
                  let current_roles: Vec<RoleId> = m.roles.iter().filter(|rr| all_roles.contains(rr)).cloned().collect();
                  let target_roles: Vec<RoleId> = all_roles.iter().filter(|rr| i.data.values.contains(&rr.0.to_string())).cloned().collect();

                  let rem_roles: Vec<RoleId> = current_roles.iter().filter(|x|!target_roles.contains(x)).cloned().collect();
                  m.remove_roles(&c, rem_roles.as_slice()).await?;

                  let add_roles: Vec<RoleId> = target_roles.iter().filter(|x|!current_roles.contains(x)).cloned().collect();
                  m.add_roles(&c, add_roles.as_slice()).await?;

                  let rem = rem_roles.iter().map(|x|format!("<@&{}>", x.0)).join(", ");
                  let add = add_roles.iter().map(|x|format!("<@&{}>", x.0)).join(", ");
                  i.create_interaction_response(c, |a| {
                    a.kind(InteractionResponseType::ChannelMessageWithSource).interaction_response_data(|d| {
                      d.ephemeral(true).content({
                        let mut msg = String::new();
                        if rem_roles.len() > 0 {
                          msg = format!("**Removed roles:** {rem}\n");
                        }
                        if add_roles.len() > 0 {
                          msg += &format!("**Added roles:** {add}\n");
                        }
                        msg
                      })
                    })
                  }).await?;
                }
              }
            }
          }
        }
      },
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
