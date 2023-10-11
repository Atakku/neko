// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use neko::{core::*, plugins::*};

#[tokio::main]
async fn main() -> R {
  if !std::env::var("RUST_LOG").is_ok_and(|f| !f.is_empty()) {
    std::env::set_var("RUST_LOG", "warn,neko=trace");
  }
  pretty_env_logger::init();
  let mut fw = Framework::new();
  fw.init_module(Atakku)?;
  fw.init_module(FemboyTV)?;
  fw.init_module(Discord)?;
  fw.init_module(Steam)?;
  fw.init_module(DeepRockGalactic)?;
  fw.init_module(Gwaaa {})?;
  fw.run().await?;
  Ok(())
}
