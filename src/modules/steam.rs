// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::{axum::Axum, cron::Cron, poise::EventHandler};
use crate::{
  core::*,
  modules::{
    poise::{Ctx, Poise},
    sqlx::Postgres,
  },
  query::{
    autocomplete::*,
    neko::all_steam_connections,
    steam::{build_top_query, update_apps, update_playdata, update_users, At, By, Of, QueryOutput},
  },
};
use axum::routing::get;
use poise::{
  serenity_prelude::{
    ButtonStyle, CollectComponentInteraction, CreateActionRow, InteractionResponseType, Member,
    ReactionType, Role, RoleId, UserId,
  },
  Event,
};
use sea_query::Query;
use tokio_cron_scheduler::Job;

pub struct Steam;

once_cell!(sapi_key, APIKEY: String);

impl Module for Steam {
  fn init(&mut self, fw: &mut Framework) -> R {
    APIKEY.set(expect_env!("STEAMAPI_KEY"))?;
    fw.req_module::<Postgres>()?;
    let poise = fw.req_module::<Poise>()?;
    poise.commands.push(steam());
    poise.event_handlers.push(roles());
    let cron = fw.req_module::<Cron>()?;
    cron.jobs.push(Job::new_async("0 0 */6 * * *", |_id, _jsl| {
      Box::pin(async move {
        minor_update().await.unwrap();
      })
    })?);
    cron.jobs.push(Job::new_async("0 0 0 */7 * *", |_id, _jsl| {
      Box::pin(async move {
        //update_apps().await.unwrap()
      })
    })?);
    Ok(())
  }
}

fn roles() -> EventHandler {
  |c, event| {
    Box::pin(async move {
      use Event::*;
      match event {
        GuildMemberAddition { new_member: m } => {
          if !m.user.bot {
            let roles = filter_roles(m.roles(c).unwrap_or_default(), get_roles(m).await?);
            let mut member = m.guild_id.member(c, m.user.id).await?;
            member.add_roles(c, roles.as_slice()).await?;
          }
        }
        _ => {}
      }
      Ok(())
    })
  }
}

pub async fn get_roles(m: &Member) -> Res<Vec<RoleId>> {
  use crate::schema::*;
  let mut qb = Query::select();
  qb.from(steam::DiscordRoles::Table);
  qb.from(steam::Playdata::Table);
  qb.from(neko::UsersSteam::Table);
  qb.from(neko::UsersDiscord::Table);
  qb.column(col!(steam::DiscordRoles, RoleId));

  qb.cond_where(ex_col!(steam::DiscordRoles, GuildId).eq(m.guild_id.0 as i64));
  qb.cond_where(ex_col!(steam::DiscordRoles, AppId).equals(col!(steam::Playdata, AppId)));
  qb.cond_where(ex_col!(neko::UsersSteam, SteamId).equals(col!(steam::Playdata, UserId)));
  qb.cond_where(ex_col!(neko::UsersSteam, NekoId).equals(col!(neko::UsersDiscord, NekoId)));
  qb.cond_where(ex_col!(neko::UsersDiscord, DiscordId).eq(m.user.id.0 as i64));
  qb.distinct();
  Ok(
    fetch_all!(&qb, (i64,))?
      .into_iter()
      .map(|r| RoleId(r.0 as u64))
      .collect(),
  )
}

pub fn filter_roles(og: Vec<Role>, add: Vec<RoleId>) -> Vec<RoleId> {
  let mapped: Vec<u64> = og.into_iter().map(|r| r.id.0).collect();
  add.into_iter().filter(|r| !mapped.contains(&r.0)).collect()
}

pub async fn minor_update() -> R {
  let c = all_steam_connections().await?;
  update_users(&c).await?;
  update_playdata(&c).await?;
  Ok(())
}

cmd_group!(steam, "user_top", "app_top", "guild_top", "top");

#[poise::command(prefix_command, slash_command)]
pub async fn top(ctx: Ctx<'_>, of: Of, by: By) -> R {
  handle(ctx, format!("Top of {of} by {by}"), of, by, At::None).await
}

#[poise::command(prefix_command, slash_command)]
pub async fn user_top(ctx: Ctx<'_>, of: Of, by: By, user: UserId) -> R {
  handle(
    ctx,
    format!("Users's ({user}) top of {of} by {by}"),
    of,
    by,
    At::User(user.0 as i64),
  )
  .await
}

#[poise::command(prefix_command, slash_command)]
pub async fn app_top(ctx: Ctx<'_>, of: Of, by: By, #[autocomplete = "steam_apps"] app: i32) -> R {
  handle(
    ctx,
    format!("Apps's ({app}) top of {of} by {by}"),
    of,
    by,
    At::App(app),
  )
  .await
}

#[poise::command(prefix_command, slash_command)]
pub async fn guild_top(
  ctx: Ctx<'_>,
  of: Of,
  by: By,
  #[autocomplete = "discord_guilds"] guild: String,
) -> R {
  handle(
    ctx,
    format!("Guild's ({guild}) top of {of} by {by}"),
    of,
    by,
    At::Guild(guild.parse::<i64>()?),
  )
  .await
}

const SIZE: u64 = 10;
const PAGES: u64 = 100; //todo

async fn handle(ctx: Ctx<'_>, input: String, of: Of, by: By, at: At) -> R {
  let mut msg = ctx
    .send(|b| {
      b.content(input.clone()).components(|b| {
        b.create_action_row(|b| pagination_buttons(b, 0, 0, true, "pg_disp".into()))
      })
    })
    .await?
    .into_message()
    .await?;

  let bys = match by {
    By::Playtime => "hours",
    By::Ownership => "copies",
  };
  let divider = if let By::Playtime = by { 60 } else { 1 };
  let qb = build_top_query(of, by, at);

  let get_page = async move |page: u64| -> Res<String> {
    let mut pb = qb.clone();
    if page == 0 {
      pb.limit(SIZE + 1);
    } else {
      pb.limit(SIZE + 2);
      pb.offset(page * SIZE - 1);
    }
    let data = fetch_all!(&pb, QueryOutput)?;
    let mut output = String::new();
    for d in data.into_iter().take(10) {
      output += &format!("{} | {} | {}\n", d.row_num, d.sum_count / divider, d.name);
    }
    Ok(output)
  };

  let mut page = 0;
  let firstpage = get_page.clone()(page).await?;

  msg
    .edit(ctx, |b| {
      b.content(format!("{input}\n```\n# | {bys} | name \n{}```", firstpage))
        .components(|b| {
          b.create_action_row(|b| pagination_buttons(b, page, PAGES, false, "".into()))
        })
    })
    .await?;

  let mut id = msg.id.0;

  while let Some(press) = CollectComponentInteraction::new(ctx)
    .message_id(msg.id)
    .timeout(std::time::Duration::from_secs(300))
    .await
  {
    match press.data.custom_id.as_str() {
      "pg_prev" => page -= 1,
      "pg_next" => page += 1,
      _ => {}
    }

    press
      .create_interaction_response(ctx, |f| {
        f.kind(InteractionResponseType::UpdateMessage)
          .interaction_response_data(|b| {
            b.components(|b| {
              b.create_action_row(|b| {
                pagination_buttons(b, page, PAGES, true, press.data.custom_id.clone())
              })
            })
          })
      })
      .await?;

    let pageee = get_page.clone()(page).await.unwrap();

    let mut msg = press.get_interaction_response(ctx).await?;
    msg
      .edit(ctx, |b| {
        b.content(format!("{input}\n```\n# | {bys} | name \n{}```", pageee))
          .components(|b| {
            b.create_action_row(|b| {
              pagination_buttons(b, page, PAGES, false, press.data.custom_id.clone())
            })
          })
      })
      .await?;

    id = msg.id.0;
  }
  ctx
    .http()
    .get_message(msg.channel_id.0, id)
    .await?
    .edit(ctx, |m| {
      m.components(|b| b.create_action_row(|b| pagination_buttons(b, page, PAGES, true, "".into())))
    })
    .await?;
  Ok(())
}


fn pagination_buttons(
  b: &mut CreateActionRow,
  page: u64,
  pages: u64,
  loading: bool,
  event: String,
) -> &mut CreateActionRow {
  let l = ReactionType::Custom {
    animated: true,
    id: poise::serenity_prelude::EmojiId(1110725977069326346),
    name: None,
  };
  b.create_button(|b| {
    if event == "pg_prev" && loading {
      b.emoji(l.clone())
    } else {
      b.label("<")
    }
    .custom_id(format!("pg_prev"))
    .style(ButtonStyle::Secondary)
    .disabled(loading || page == 0)
  });
  b.create_button(|b| {
    if event == "pg_disp" && loading {
      b.emoji(l.clone())
    } else {
      b.label(format!("{}/{pages}", page + 1))
    }
    .custom_id(format!("pg_disp"))
    .style(ButtonStyle::Secondary)
    .disabled(true)
  });
  b.create_button(|b| {
    if event == "pg_next" && loading {
      b.emoji(l)
    } else {
      b.label(">")
    }
    .custom_id(format!("pg_next"))
    .style(ButtonStyle::Secondary)
    .disabled(loading || page + 1 == pages)
  });

  b
}
