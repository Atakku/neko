// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

#![feature(trait_alias)]
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

#[macro_use]
pub mod modules {
  pub mod axum;
  #[macro_use]
  pub mod cron;
  pub mod fluent;
  #[macro_use]
  pub mod poise;
  pub mod reqwest;
  #[macro_use]
  pub mod sqlx;
  pub mod svgui;
}

macro_rules! plugins {
  ($root:ident, $plugin:ident) => {
    plugins!($root, [], $plugin);
  };
  ($root:ident, [$($mod:ident),*], $plugin:ident) => {
    mod $root {
      mod module;
      pub use module::*;
      $(pub mod $mod;)*
    }
    pub use $root::$plugin;
  };
}

pub mod plugins {
  plugins!(discord_cache, [schema], DiscordCache);
  plugins!(discord_welcomer, [schema], DiscordWelcomer);
  plugins!(drg, [wrapper], DeepRockGalactic);
  plugins!(ftv, FemboyTV);
  plugins!(mnts, Maintenance);
  plugins!(neko, [query, schema], Gwaaa);
  plugins!(steam, [poise, query, schema, wrapper], Steam);
}
