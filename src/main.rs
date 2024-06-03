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
  #[cfg(not(debug_assertions))]
  {
    fw.init_module(atakku::Atakku)?;
    fw.init_module(discord::Discord)?;
    fw.init_module(steam::Steam)?;
    fw.init_module(anilist::AniList)?;
    fw.init_module(drg::DeepRockGalactic)?;
    fw.init_module(gwaaa::Gwaaa {})?;
    fw.init_module(warnsys::WarnSystem {})?;
    fw.init_module(beatleader::BeatLeader {})?;
    fw.init_module(radio::Radio {})?;
    fw.init_module(welcomer::Welcomer)?;
    fw.init_module(ftvroles::FTVRoles)?;
    fw.init_module(starboard::Starboard)?;
  }
  fw.init_module(streak::PostStreak)?;
  fw.run().await?;
  Ok(())
}
