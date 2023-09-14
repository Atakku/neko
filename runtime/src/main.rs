// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use neko_core::*;

#[tokio::main]
async fn main() -> R {
  if !std::env::var("RUST_LOG").is_ok_and(|f| !f.is_empty()) {
    std::env::set_var("RUST_LOG", "warn");
    #[cfg(debug_assertions)]
    std::env::set_var("RUST_LOG", "debug");
  }
  pretty_env_logger::init();
  let mut fw = Framework::new();
  fw.init_module(neko_discord::DiscordPlugin {})?;
  fw.run().await?;
  Ok(())
}
