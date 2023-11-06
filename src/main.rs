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
  fw.init(Maintenance)?;
  fw.init(FemboyTV)?;
  fw.init(Discord)?;
  fw.init(Steam)?;
  fw.init(DeepRockGalactic)?;
  fw.init(Gwaaa {})?;
  fw.run().await?;
  Ok(())
}
