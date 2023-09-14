// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use std::any::Any;

use crate::{framework::Framework, Res, R};

pub trait Module: Any {
  fn init(&self, fw: &mut Framework) -> R;
}

impl Framework {
  pub fn has_module<T: Module>(&mut self) -> bool {
    self.modules.has::<T>()
  }

  pub fn init_module<T: Module>(&mut self, module: T) -> Res<&mut Self> {
    log::info!("Initializing {}", std::any::type_name::<T>());
    module.init(self)?;
    self.modules.put(module);
    Ok(self)
  }

  pub fn req_module<T: Module + Default>(&mut self) -> Res<&mut T> {
    if !self.has_module::<T>() {
      self.init_module(T::default())?;
    }
    self.modules.borrow_mut::<T>()
  }
}
