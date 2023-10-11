// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![feature(trait_alias)]
#![feature(async_fn_in_trait)]
#![feature(async_closure)]

#[macro_use]
pub mod macros;

#[macro_use]
pub mod core {
  use std::error::Error;

  mod framework;
  pub use framework::*;
  #[macro_use]
  mod module;
  pub use module::*;
  mod state;

  pub type Err = Box<dyn Error + Send + Sync>;
  pub type Res<T> = Result<T, Err>;
  pub type R = Res<()>;
}

pub mod modules {
  automod::dir!(pub "src/modules");
}

pub mod plugins {
  #[path = "atakku/plugin.rs"]
  mod atakku;
  pub use atakku::Atakku;
  #[path = "discord/plugin.rs"]
  mod discord;
  pub use discord::Discord;
  #[path = "drg/plugin.rs"]
  mod drg;
  pub use drg::DeepRockGalactic;
  #[path = "ftv/plugin.rs"]
  mod ftv;
  pub use ftv::FemboyTV;
  #[path = "neko/plugin.rs"]
  mod neko;
  pub use neko::Gwaaa;
  #[path = "steam/plugin.rs"]
  mod steam;
  pub use steam::Steam;
}
