// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::cron::Cron;
use crate::{
  core::*,
  modules::{poise::Poise, reqwest::req, sqlx::Postgres, steam::pagination_buttons},
  query::{neko::all_steam_connections, steam::ratelimit}, schema::{neko::{UsersSteam, UsersDiscord}, discord::Users},
};
use poise::serenity_prelude::{CollectComponentInteraction, InteractionResponseType};
use serde::Deserialize;
use sqlx::FromRow;
use tokio_cron_scheduler::Job;

pub struct BeatLeader;

impl Module for BeatLeader {
  fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>()?;
    let poise = fw.req_module::<Poise>()?;
    poise.commands.push(beetleader());
    let cron = fw.req_module::<Cron>()?;
    cron.jobs.push(Job::new_async("0 0 */1 * * *", |_id, _jsl| {
      Box::pin(async move {
        update_scores().await.unwrap();
      })
    })?);
    Ok(())
  }
}
use crate::modules::poise::Ctx;


const SIZE: u64 = 15;
const PAGES: u64 = 100; //todo

#[poise::command(slash_command)]
pub async fn beetleader(ctx: Ctx<'_>) -> R {
  let input = "BeetLeader top:";

  let mut msg = ctx
    .send(|b| {
      b.content(input).components(|b| {
        b.create_action_row(|b| pagination_buttons(b, 0, 0, true, "pg_disp".into()))
      })
    })
    .await?
    .into_message()
    .await?;

  let qb = {
    let mut qb = Query::select();
    qb.from(UsersSteam::Table);
    qb.from(UsersDiscord::Table);
    qb.from(Users::Table);
    qb.from(BeetleaderLB::Table);
    qb.and_where(ex_col!(Users, Id).equals(col!(UsersDiscord, DiscordId)));
    qb.and_where(ex_col!(UsersSteam, NekoId).equals(col!(UsersDiscord, NekoId)));
    qb.and_where(ex_col!(UsersSteam, SteamId).equals(col!(BeetleaderLB, SteamId)));
    qb.column(col!(Users, Name));
    qb.column(col!(BeetleaderLB, Pp));
    {
      qb.expr_window_as(
        Func::cust(Alias::new("ROW_NUMBER")),
        WindowStatement::new()
          .order_by(col!(BeetleaderLB, Pp), Order::Desc)
          .to_owned(),
        Alias::new("row_num"),
      );
    }
    qb.order_by(Alias::new("row_num"), Order::Desc);
    qb
  };

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
    for (i, d) in data.iter().enumerate() {
      if i == SIZE as usize && page == 0 || i == SIZE as usize + 1 && page != 0 {
        output += "-------------------\n"
      }
      output += &format!("{} | {} | {}\n", d.row_num, d.pp, d.name);
      if i == 0 && page != 0 {
        output += "-------------------\n"
      }
    }
    Ok(output)
  };

  let mut page = 0;
  let firstpage = get_page.clone()(page).await?;

  msg
    .edit(ctx, |b| {
      b.content(format!("{input}\n```\n# | pp | name \n{firstpage}```\nTo add your steam to this list, head over to https://link.neko.rs\nThis bot is still in early development, so bear with the bad design, feedback is appreciated\nDebug locale: {}", ctx.locale().unwrap_or("none")))
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
        b.content(format!("{input}\n```\n# | pp | name \n{pageee}```\nTo add your steam to this list, head over to https://link.neko.rs\nThis bot is still in early development, so bear with the bad design, feedback is appreciated\nDebug locale: {}", ctx.locale().unwrap_or("none")))
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


#[derive(FromRow)]
pub struct QueryOutput {
  pub row_num: i64,
  pub name: String,
  pub pp: f32,
}


pub async fn update_scores() -> R {
  let c = all_steam_connections().await?;
  let mut push: Vec<(i64, f32)> = vec![];
  for (acc,) in c {
    if let Ok(scores) = get_scores(acc, 0).await {
      log::info!("Got all scores for {acc}");
      let mut pps = scores.iter().map(|a| a.pp).collect::<Vec<f32>>();
      pps.sort_floats();
      pps.reverse();
      pps.truncate(100);
      push.push((
        acc,
        pps
          .iter()
          .enumerate()
          .map(|(n, x)| x * 0.965_f32.powi(n as i32))
          .sum::<f32>(),
      ))
    }
  }
  use BeetleaderLB::*;
  let mut qb = Query::insert();
  qb.into_table(Table);
  qb.columns([SteamId, Pp]);
  qb.on_conflict(OnConflict::column(SteamId).update_column(Pp).to_owned());
  for v in push {
    qb.values([v.0.into(), v.1.into()])?;
  }
  execute!(&qb)?;
  Ok(())
}

use sea_query::{Iden, OnConflict, Query, WindowStatement, Func, Alias, Order};

#[derive(Iden)]
#[iden(rename = "beetleader_lb")]
pub enum BeetleaderLB {
  Table,
  SteamId,
  Pp,
}

pub async fn get_scores(id: i64, t: i64) -> Res<Vec<PlayerScoresData>> {
  let mut page = futures::join!(get_scores_paginated(id, t, 1), ratelimit()).0?;
  let mut data: Vec<PlayerScoresData> = vec![];
  data.append(&mut page.data);
  if page.metadata.total > 100 {
    for i in 2..=(page.metadata.total as f64 / 100_f64).ceil() as u64 {
      let mut page = futures::join!(get_scores_paginated(id, t, i), ratelimit()).0?;
      data.append(&mut page.data);
    }
  }
  Ok(data)
}

async fn get_scores_paginated(id: i64, t: i64, page: u64) -> Res<PlayerScores> {
  Ok(
    req()
      .get(format!(
        "https://api.beatleader.xyz/player/{id}/scores?time_from={t}&count=100&page={page}"
      ))
      .send()
      .await?
      .json()
      .await?,
  )
}

#[derive(Deserialize, Debug)]
pub struct PlayerScores {
  metadata: PlayerScoresMetadata,
  data: Vec<PlayerScoresData>,
}

#[derive(Deserialize, Debug)]
pub struct PlayerScoresMetadata {
  total: i64,
}

#[derive(Deserialize, Debug)]
pub struct PlayerScoresData {
  pub leaderboardId: String,
  pub timepost: u64,
  pub pp: f32,
}
