// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::error::Error;

mod framework;
pub use framework::*;
mod module;
pub use module::*;
mod state;

pub type Err = Box<dyn Error + Send + Sync>;
pub type Res<T> = Result<T, Err>;
pub type R = Res<()>;
