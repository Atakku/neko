// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use serde::Deserialize;

api!(RadioApi, "https://azuracast.atakku.dev/api/", {
  fn get_nowplaying("nowplaying/femboytv") -> Vec<StationData>;
});

#[derive(Deserialize)]
pub struct StationData {
  pub now_playing: NowPlaying,
}

#[derive(Deserialize)]
pub struct NowPlaying {
  pub song: Song,
  pub duration: u32,
  pub elapsed: u32,
  pub remaining: u32,
}

#[derive(Deserialize)]
pub struct Song {
  pub artist: String,
  pub title: String,
  pub album: String,
}