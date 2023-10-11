// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  modules::{
    poise::{Ctx, Poise, EventHandler},
    sqlx::Postgres, svgui::{SvgUi, render_svg}, cron::Cron,
  },
};
use askama::Template;
use futures::future::join_all;
use poise::{
  serenity_prelude::{
    AttachmentType, ButtonStyle, CollectComponentInteraction, CreateActionRow,
    InteractionResponseType, Member, ReactionType, Role, RoleId,
  },
  Event,
};
use sea_query::Query;
use std::path::Path;
use tokio_cron_scheduler::Job;

use super::query::{build_top_query, By, Of, At, QueryOutput, update_users, update_playdata};

use crate::plugins::neko::query::all_steam_connections;

pub struct Steam;

autocomplete!(steam_apps, crate::plugins::steam::schema::Apps);

once_cell!(sapi_key, APIKEY: String);

impl Module for Steam {
  fn init(&mut self, fw: &mut Framework) -> R {
    APIKEY.set(env!("STEAMAPI_KEY"))?;
    fw.req::<SvgUi>()?;
    fw.req::<Postgres>()?;
    let poise = fw.req::<Poise>()?;
    poise.add_command(steam());
    poise.add_event_handler(roles());
    let cron = fw.req::<Cron>()?;
    cron.add_job(Job::new_async("0 0 */1 * * *", |_id, _jsl| {
      Box::pin(async move {
        minor_update().await.unwrap();
      })
    })?);
    cron.add_job(Job::new_async("0 0 0 */7 * *", |_id, _jsl| {
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
  use crate::plugins::*;
  let mut qb = Query::select();
  qb.from(steam::schema::DiscordRoles::Table);
  qb.from(steam::schema::Playdata::Table);
  qb.from(neko::schema::UsersSteam::Table);
  qb.from(neko::schema::UsersDiscord::Table);
  qb.column(col!(steam::schema::DiscordRoles, RoleId));

  qb.cond_where(ex_col!(steam::schema::DiscordRoles, GuildId).eq(m.guild_id.0 as i64));
  qb.cond_where(ex_col!(steam::schema::DiscordRoles, AppId).equals(col!(steam::schema::Playdata, AppId)));
  qb.cond_where(ex_col!(neko::schema::UsersSteam, SteamId).equals(col!(steam::schema::Playdata, UserId)));
  qb.cond_where(ex_col!(neko::schema::UsersSteam, NekoId).equals(col!(neko::schema::UsersDiscord, NekoId)));
  qb.cond_where(ex_col!(neko::schema::UsersDiscord, DiscordId).eq(m.user.id.0 as i64));
  qb.distinct();
  Ok(
    sql!(FetchAll, &qb, (i64,))?
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

cmd_group!(steam, "user", "app", "guild", "top");

cmd_group!(user, "user::top");
cmd_group!(app, "app::top");
cmd_group!(guild, "guild::top");

#[poise::command(prefix_command, slash_command)]
pub async fn top(ctx: Ctx<'_>, by: By, of: Of) -> R {
  handle(ctx, format!("Top of {of} by {by}"), of, by, At::None).await
}
//context_menu_command = "gwaa"

mod user {
  use crate::{
    core::R,
    modules::{poise::Ctx}, plugins::steam::{query::{At, Of, By}, module::handle},
  };
  use poise::serenity_prelude::UserId;

  #[poise::command(prefix_command, slash_command)]
  pub async fn top(ctx: Ctx<'_>, by: By, user: Option<UserId>) -> R {
    let user = user.unwrap_or(ctx.author().id);
    let of = Of::Apps;
    let title = format!("Users's ({user}) top of {of} by {by}");
    handle(ctx, title, of, by, At::User(user.0 as i64)).await
  }
}
mod app {
  use crate::{
    core::R,
    modules::{poise::Ctx}, plugins::steam::{query::{At, By, self}, steam_apps, module::handle}
  };
  use poise::ChoiceParameter;

use query::Of;
  #[derive(ChoiceParameter)]
  enum AppTop {
    Users,
    Guilds,
  }

  impl Into<Of> for AppTop {
    fn into(self) -> Of {
      match self {
        AppTop::Users => Of::Users,
        AppTop::Guilds => Of::Guilds,
      }
    }
  }

  #[poise::command(prefix_command, slash_command)]
  pub async fn top(ctx: Ctx<'_>, by: By, of: AppTop, #[autocomplete = "steam_apps"] app: i32) -> R {
    let title = format!("Apps's ({app}) top of {of} by {by}");
    handle(ctx, title, of.into(), by, At::App(app)).await
  }
}

mod guild {
  use crate::{
    core::R, plugins::{steam::{query::{At, By, self}, module::handle}, discord::discord_guilds}, modules::poise::Ctx,
  };
  use poise::ChoiceParameter;

use query::Of;

  #[derive(ChoiceParameter)]
  enum GuildTop {
    Users,
    Apps,
  }

  impl Into<Of> for GuildTop {
    fn into(self) -> Of {
      match self {
        GuildTop::Users => Of::Users,
        GuildTop::Apps => Of::Apps,
      }
    }
  }

  #[poise::command(prefix_command, slash_command)]
  pub async fn top(
    ctx: Ctx<'_>,
    by: By,
    of: GuildTop,
    //#[autocomplete = "steam_apps"] app: Option<i32>,
    #[autocomplete = "discord_guilds"] guild: Option<String>,
  ) -> R {
    let guild = guild.unwrap_or(
      ctx
        .guild_id()
        .unwrap_or(crate::plugins::ftv::GUILD)
        .0
        .to_string(),
    );
    let title = format!("Guild's ({guild}) top of {of} by {by}");
    handle(ctx, title, of.into(), by, At::Guild(guild.parse::<i64>()?)).await
  }
}
const SIZE: u64 = 10;
const PAGES: u64 = 100; //todo

#[derive(Template, Clone)]
#[template(path = "svgui/top.svg", escape = "html")]
pub struct TestUI {
  //pub title: &'a String,
  //pub appid: i32,
  pub data: Vec<QueryOutput>,
  //pub total: i32,
  pub page: u64,
  pub pages: u64,
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
  let divider = if let By::Playtime = by { 60 } else { 1 };
  let qb = build_top_query(of, by, at);

  let get_page = async move |page: u64| -> Res<Vec<u8>> {
    let mut pb = qb.clone();
    if page == 0 {
      pb.limit(SIZE + 1);
    } else {
      pb.limit(SIZE + 2);
      pb.offset(page * SIZE - 1);
    }
    let data = sql!(FetchAll, &pb, QueryOutput)?;

    join_all(
      data
        .iter()
        .map(|i| fetch_asset(i.id as u32, "library_600x900")),
    )
    .await;

    //let mut output = String::new();
    //for (i, d) in data.iter().enumerate() {
    //  if i == SIZE as usize && page == 0 || i == SIZE as usize + 1 && page != 0 {
    //    output += "-------------------\n"
    //  }
    //  output += &format!("{} | {} | {}\n", d.row_num, d.sum_count / divider, d.name);
    //  if i == 0 && page != 0 {
    //    output += "-------------------\n"
    //  }
    //}

    render_svg(TestUI {
      data,
      page,
      pages: PAGES,
    })
    .await
  };

  let mut page = 0;
  let firstpage = get_page.clone()(page).await?;

  msg
    .edit(ctx, |b| {
      b.remove_all_attachments()
        .attachment(AttachmentType::Bytes {
          data: firstpage.into(),
          filename: "page.webp".to_owned(),
        })
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
        b.remove_all_attachments()
          .attachment(AttachmentType::Bytes {
            data: pageee.into(),
            filename: "page.webp".to_owned(),
          })
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

pub async fn fetch_asset<'a>(appid: u32, asset: &'a str) -> R {
  let name = format!(".cache/steam/{}/{}.jpg", asset, appid);
  let path = Path::new(&name);
  std::fs::create_dir_all(format!(".cache/steam/{}", asset))?;
  if !path.exists() {
    let req = reqwest::get(format!(
      "https://cdn.cloudflare.steamstatic.com/steam/apps/{}/{}.jpg",
      appid, asset
    ))
    .await?;
    if req.status() == 200 {
      std::fs::write(path, Into::<Vec<u8>>::into(req.bytes().await?))?;
    }
  }
  Ok(())
}
