// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use neko::{core::*, modules::*};

#[tokio::main]
async fn main() -> R {
  if !std::env::var("RUST_LOG").is_ok_and(|f| !f.is_empty()) {
    std::env::set_var("RUST_LOG", "warn,neko=trace");
  }
  pretty_env_logger::init();
  let mut fw = Framework::new();
  fw.init_module(atakku::Atakku)?;
  fw.init_module(ftv::FemboyTV)?;
  fw.init_module(discord::Discord)?;
  fw.init_module(steam::Steam)?;
  fw.init_module(anilist::AniList)?;
  fw.init_module(drg::DeepRockGalactic)?;
  fw.run().await?;
  Ok(())
}
