// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use super::{Poise, Postgres};
use crate::core::*;

pub struct Steam;

impl Module for Steam {
  fn init(&self, fw: &mut Framework) -> R {
    fw.req_module::<Postgres>()?;
    let poise = fw.req_module::<Poise>()?;
    Ok(())
  }
}
