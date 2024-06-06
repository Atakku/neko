// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use neko::{core::*, modules::*};

#[tokio::main]
async fn main() -> R {
  //if !std::env::var("RUST_LOG").is_ok_and(|f| !f.is_empty()) {
  //  std::env::set_var("RUST_LOG", "warn,neko=trace");
  //}
  pretty_env_logger::init();
  let mut fw = Framework::new();
  //#[cfg(not(debug_assertions))]
  //{
    fw.init_module(atakku::Atakku).await?;
    fw.init_module(discord::Discord).await?;
    fw.init_module(steam::Steam).await?;
    fw.init_module(anilist::AniList).await?;
    fw.init_module(drg::DeepRockGalactic).await?;
    fw.init_module(gwaaa::Gwaaa {}).await?;
    fw.init_module(warnsys::WarnSystem {}).await?;
    fw.init_module(beatleader::BeatLeader {}).await?;
    fw.init_module(radio::Radio {}).await?;
    fw.init_module(welcomer::Welcomer).await?;
    fw.init_module(ftvroles::FTVRoles).await?;
    fw.init_module(starboard::Starboard).await?;
  //}
  fw.init_module(streak::PostStreak).await?;
  fw.run().await?;
  Ok(())
}
