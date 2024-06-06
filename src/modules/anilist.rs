// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{core::*, modules::poise::Poise};

pub struct AniList;

impl Module for AniList {
  async fn init(&mut self, fw: &mut Framework) -> R {
    let poise = fw.req_module::<Poise>().await?;
    
    Ok(())
  }
}
