// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use self::query::{build_top_query, update_playdata, update_users, At, By, Of, QueryOutput};
use crate::{
  core::*,
  modules::{
    cron::Cron,
    poise::{Ctx, EventHandler, Poise},
    sqlx::Postgres,
  },
  plugins::neko::query::all_steam_connections,
};
use poise::{
  serenity_prelude::{
    ButtonStyle, CollectComponentInteraction, CreateActionRow, InteractionResponseType, Member,
    ReactionType, Role, RoleId, UserId,
  },
  Event,
};
use sea_query::Query;
use tokio_cron_scheduler::Job;

pub mod interface;
pub mod query;
pub mod schema;

pub struct Steam;

once_cell!(sapi_key, APIKEY: String);

impl Module for Steam {
  async fn init(&mut self, fw: &mut Framework) -> R {
    APIKEY.set(expect_env!("STEAMAPI_KEY"))?;
    fw.req_module::<Postgres>().await?;
    let poise = fw.req_module::<Poise>().await?;
    poise.commands.push(steam());
    poise.event_handlers.push(roles());
    cron!(fw, "0 0 */1 * * *", || { minor_update().await.unwrap() });
    //cron!(fw, "0 0 0 */7 * *", || { update_apps().await.unwrap() });
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
            let roles = filter_roles(&m.roles(c).unwrap_or_default(), get_roles(m).await?);
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
  use crate::plugins::{neko::schema as neko, steam::schema as steam};
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

pub fn filter_roles(og: &Vec<Role>, add: Vec<RoleId>) -> Vec<RoleId> {
  let mapped: Vec<u64> = og.into_iter().map(|r| r.id.0).collect();
  add.into_iter().filter(|r| !mapped.contains(&r.0)).collect()
}

pub async fn minor_update() -> R {
  let c = all_steam_connections().await?;
  update_users(&c).await?;
  update_playdata(&c).await?;
  Ok(())
}

cmd_group!(steam, "user", "app", "top");

cmd_group!(user, "user::top");
cmd_group!(app, "app::top");

#[poise::command(prefix_command, slash_command)]
pub async fn top(ctx: Ctx<'_>, by: By, of: Of) -> R {
  handle(ctx, format!("Top of {of} by {by}"), of, by, At::None).await
}
//context_menu_command = "gwaa"

mod user {
  use super::{
    handle,
    query::{At, By, Of},
  };
  use crate::{core::R, modules::poise::Ctx};
  use poise::serenity_prelude::UserId;

  #[poise::command(prefix_command, slash_command)]
  pub async fn top(ctx: Ctx<'_>, by: By, user: Option<UserId>) -> R {
    let user = user.unwrap_or(ctx.author().id);
    let title = format!("Users's ({user}) top of apps by {by}");
    handle(ctx, title, Of::Apps, by, At::User(user.0 as i64)).await
  }
}
mod app {
  use sea_query::Query;
use sqlx::Column;

use crate::{
    core::R,
    modules::poise::Ctx,
    plugins::{
      neko::autocomplete::steam_apps,
      steam::{
        handle,
        query::{At, By, Of},
      },
    },
  };

  #[poise::command(prefix_command, slash_command)]
  pub async fn top(ctx: Ctx<'_>, by: By, #[autocomplete = "steam_apps"] app: i32) -> R {
    
    use super::schema::Apps::{self, *};
    let mut qb = Query::select();
    qb.from(Table);
    qb.columns([Name]);
    qb.and_where(ex_col!(Apps, Id).eq(app));
    let (name,) = fetch_one!(&qb, (String,)).unwrap();

    let title = format!("Top {name} gamers by {by}");
    handle(ctx, title, Of::Users, by, At::App(app)).await
  }
}

const SIZE: u64 = 15;
const PAGES: u64 = 100; //todo

fn divdec(f: i64, s: i64) -> (i64, i64) {
  (f / s, f * 10 / s % 10)
}

fn fmt_sec(num: i64) -> String {
  let (mf, md) = divdec(num, 60);
  let (hf, hd) = divdec(num, 3600);
  let (df, dd) = divdec(num, 86400);
  let (wf, wd) = divdec(num, 604800);
  let (yf, yd) = divdec(num, 31556926);

  if mf < 1 {
    format!("{num}s")
  } else if mf < 10 {
    format!("{mf}.{md}m")
  } else if hf < 1 {
    format!("{mf}m")
  } else if hf < 10 {
    format!("{hf}.{hd}h")
  } else if df < 1 {
    format!("{hf}h")
  } else if df < 10 {
    format!("{df}.{dd}d")
  } else if wf < 1 {
    format!("{df}d")
  } else if wf < 10 {
    format!("{wf}.{wd}w")
  } else if yf < 1 {
    format!("{wf}w")
  } else if yf < 10 {
    format!("{yf}.{yd}y")
  } else {
    format!("{yf}y")
  }
}


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

  let isuser = Of::Users == of;
  let divider = By::Playtime == by;
  let qb = build_top_query(of, by, at);

  let get_page = async move |page: u64| -> Res<String> {
    let mut pb = qb.clone();
    pb.limit(SIZE);
    pb.offset(page * SIZE);
    //if page == 0 {
    //  pb.limit(SIZE + 1);
    //} else {
    //  pb.limit(SIZE + 2);
    //  pb.offset(page * SIZE - 1);
    //}
    let data = fetch_all!(&pb, QueryOutput)?;
    let mut output = String::new();
    let data: Vec<_> = data.into_iter().map(|a| (if divider { fmt_sec(a.sum_count * 60) } else { format!("{}", a.sum_count)}, a.id, a.name) ).collect();
    let max = data.iter().map(|(a,b,c)|a.len()).max().unwrap_or(5);
    for (c, id, name) in data {
      if (isuser) {

        output += &format!("`{c: >max$}` <@{id}>\n");
      } else {
        output += &format!("`{c: >max$} | {name}` \n");
      }
    }
    Ok(output)
  };

  let mut page = 0;
  let firstpage = get_page(page).await?;

  msg
    .edit(ctx, |b| {
      b.content(format!("{input}\n{firstpage}\nTo add your steam to this list, head over to https://link.neko.rs"))
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

    let pageee = get_page(page).await.unwrap();

    let mut msg = press.get_interaction_response(ctx).await?;
    msg
      .edit(ctx, |b| {
        b.content(format!("{input}\n{pageee}\nTo add your steam to this list, head over to https://link.neko.rs"))
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

pub fn pagination_buttons(
  b: &mut CreateActionRow,
  page: u64,
  pages: u64,
  loading: bool,
  event: String,
) -> &mut CreateActionRow {
  let l = ReactionType::Custom {
    animated: true,
    id: poise::serenity_prelude::EmojiId(1233072462527332363),
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
