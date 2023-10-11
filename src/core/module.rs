// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::{Framework, Res, R};
use std::any::Any;

pub trait Module: Any {
  fn init(&mut self, fw: &mut Framework) -> R;
}

impl Framework {
  pub fn has_module<T: Module>(&mut self) -> bool {
    self.modules.has::<T>()
  }

  /// Load a supplied module
  pub fn init_module<T: Module>(&mut self, mut module: T) -> Res<&mut Self> {
    log::info!("Initializing {}", std::any::type_name::<T>());
    module.init(self)?;
    self.modules.put(module);
    Ok(self)
  }

  /// Check if module is already loaded, and if not, load a default impl
  pub fn req_module<T: Module + Default>(&mut self) -> Res<&mut T> {
    if !self.has_module::<T>() {
      self.init_module(T::default())?;
    }
    self.modules.borrow_mut::<T>()
  }
}

macro_rules! module {
  (@internal $name:ident, $fw:ident, $block:block) => {
    impl $crate::core::Module for $name {
      fn init(&mut self, $fw: &mut $crate::core::Framework) -> $crate::core::R {
        $block
        Ok(())
      }
    }
  };
  ($(#[$m:meta])* $name:ident {$($pn:ident: $pt:ty = $pd:expr),*$(,)?} fn init($fw:ident) $block:block) => {
    $(#[$m])*
    pub struct $name {
      $(pub $pn: $pt),*
    }

    impl Default for $name {
      fn default() -> Self { 
        Self {
          $($pn: $pd),*
        }
      }
    }

    module!(@internal $name, $fw, $block);
  };
  ($(#[$m:meta])* $name:ident; fn init($fw:ident) $block:block) => {
    $(#[$m])*
    #[derive(Default)]
    pub struct $name;
    module!(@internal $name, $fw, $block);
  };
}
