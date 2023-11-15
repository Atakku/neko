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

  #[macro_use]
  #[path="../framework.rs"]
  mod framework;
  pub use framework::*;

  pub type Err = Box<dyn Error + Send + Sync>;
  pub type Res<T> = Result<T, Err>;
  pub type R = Res<()>;
}

macro_rules! modules {
  ($root:ident, $plugin:ident) => {
    #[macro_use]
    pub mod $root;
    pub use $root::$plugin;
  };
}

#[macro_use]
pub mod modules {
  modules!(axum, Axum);
  modules!(cron, Cron);
  modules!(fluent, Fluent);
  modules!(poise, Poise);
  modules!(reqwest, Reqwest);
  modules!(sqlx, Postgres);
  modules!(svgui, SvgUi);
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
  plugins!(discord_roles, [schema], DiscordRoles);
  plugins!(discord_welcomer, [schema], DiscordWelcomer);
  plugins!(drg, [poise, wrapper], DeepRockGalactic);
  plugins!(ftv, FemboyTV);
  plugins!(mnts, Maintenance);
  plugins!(neko, [query, schema], Gwaaa);
  plugins!(steam, [poise, query, schema, wrapper], Steam);
}
