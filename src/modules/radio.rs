// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*, interface::radio::RadioApi, modules::{poise::Poise, reqwest::{req, Reqwest}}
};

pub struct Radio;

impl Module for Radio {
  async fn init(&mut self, fw: &mut Framework) -> R {
    fw.req_module::<Reqwest>().await?;
    let poise = fw.req_module::<Poise>().await?;
    poise.commands.push(radio());
    Ok(())
  }
}
use crate::modules::poise::Ctx;

#[poise::command(slash_command)]
pub async fn radio(ctx: Ctx<'_>) -> R {
  let m = ctx.reply("Fetching information...").await?;
  let data = req().get_nowplaying().await?;
  let st = data.first().ok_or("No station data available")?;
  let np = &st.now_playing;
  m.edit(ctx, |b| b.content(format!("Now playing:\nTitle: {}\nAlbum: {}\nArtist: {}\nPlayed: {}:{}/{}:{}", np.song.title, np.song.album, np.song.artist, np.elapsed/60, np.elapsed%60, np.duration/60, np.duration%60)))
    .await?;
  Ok(())
}
