// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::query::{build_top_query, update_playdata, update_users, At, By, Of, QueryOutput};
use crate::{
  core::*,
  modules::{
    cron::Cron,
    poise::{Ctx, EventHandler, Poise},
    sqlx::Postgres,
    svgui::{render_svg, SvgUi},
  },
  plugins::{neko::query::all_steam_connections, steam::{poise::steam, schema::SteamApps}},
};
use askama::Template;
use futures::future::join_all;
use poise::{
  serenity_prelude::{
    AttachmentType, ButtonStyle, CollectComponentInteraction, CreateActionRow,
    InteractionResponseType, Member, ReactionType, Role, RoleId,
  }, Event,
};
use sea_query::Query;
use std::path::Path;
use tokio_cron_scheduler::Job;

autocomplete!(steam_apps, SteamApps, AppId, AppName);

once_cell!(sapi_key, APIKEY: String);

module! {
  Steam;

  fn init(fw) {
    APIKEY.set(env!("STEAMAPI_KEY"))?;
    fw.req::<SvgUi>()?;
    fw.req::<Postgres>()?;
    let poise = fw.req::<Poise>()?;
    poise.add_command(steam());
    poise.add_event_handler(roles());
    job!(fw, "0 0 */1 * * *", {
      minor_update().await.unwrap();
    });
    job!(fw, "0 0 0 */7 * *", {
      //update_apps().await.unwrap()
    });
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
  qb.from(steam::schema::SteamDiscordRoles::Table);
  qb.from(steam::schema::SteamPlaydata::Table);
  qb.from(neko::schema::NekoUsersSteam::Table);
  qb.from(neko::schema::NekoUsersDiscord::Table);
  qb.column(col!(steam::schema::SteamDiscordRoles, RoleId));

  qb.cond_where(ex_col!(steam::schema::SteamDiscordRoles, GuildId).eq(m.guild_id.0 as i64));
  qb.cond_where(
    ex_col!(steam::schema::SteamDiscordRoles, AppId)
      .equals(col!(steam::schema::SteamPlaydata, AppId)),
  );
  qb.cond_where(
    ex_col!(neko::schema::NekoUsersSteam, SteamId)
      .equals(col!(steam::schema::SteamPlaydata, UserId)),
  );
  qb.cond_where(
    ex_col!(neko::schema::NekoUsersSteam, NekoId)
      .equals(col!(neko::schema::NekoUsersDiscord, NekoId)),
  );
  qb.cond_where(ex_col!(neko::schema::NekoUsersDiscord, DiscordId).eq(m.user.id.0 as i64));
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


//context_menu_command = "gwaa"

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

pub async fn handle(ctx: Ctx<'_>, input: String, of: Of, by: By, at: At) -> R {
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
