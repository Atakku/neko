// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::{Framework, Res, R};
use std::any::Any;
use std::future::Future;

pub trait Module: Any {
  fn init(&mut self, _: &mut Framework) -> impl Future<Output = R>;
  //{
  //  async {Ok(())}
  //}
}

impl Framework {
  pub fn has_module<T: Module>(&mut self) -> bool {
    self.modules.has::<T>()
  }

  /// Load a supplied module
  pub async fn init_module<T: Module>(&mut self, mut module: T) -> Res<&mut Self> {
    log::info!("Initializing {}", std::any::type_name::<T>());
    module.init(self).await?;
    self.modules.put(module);
    Ok(self)
  }

  /// Check if module is already loaded, and if not, load a default impl
  pub async fn req_module<T: Module + Default>(&mut self) -> Res<&mut T> {
    if !self.has_module::<T>() {
      self.init_module(T::default()).await?;
    }
    self.modules.borrow_mut::<T>()
  }
}
