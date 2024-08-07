// Copyright 2024 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use derivative::Derivative;
use fluent::{bundle::FluentBundle as GenericFluentBundle, FluentArgs, FluentResource};
use intl_memoizer::concurrent::IntlLangMemoizer;
use rust_embed::RustEmbed;
use std::{collections::HashMap, fmt::Debug};

pub type FluentResources = HashMap<String, Vec<FluentResource>>;
pub type FluentBundle = GenericFluentBundle<FluentResource, IntlLangMemoizer>;

pub struct FluentBundles {
  pub bundles: HashMap<String, FluentBundle>,
  pub default: String,
}

impl Debug for FluentBundles {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("FluentBundles")
      .field("bundles", &"<ommitted>")
      .field("default", &self.default)
      .finish()
  }
}

#[derive(RustEmbed)]
#[folder = "locale/"]
struct Locale;

#[derive(Derivative)]
#[derivative(Default)]
pub struct Fluent {
  resources: FluentResources,
  #[derivative(Default(value = "\"en-US\".to_string()"))]
  default: String,
}

once_cell!(loc, LOCALE: crate::modules::fluent::FluentBundles);

impl Module for Fluent {
  async fn init(&mut self, fw: &mut Framework) -> R {
    load_resources(&mut self.resources)?;
    fw.runtime.push(|m| {
      let this = m.take::<Self>()?;
      Ok(Box::pin(async move {
        let mut bundles = HashMap::new();
        for (locale, res) in this.resources {
          let mut bundle = FluentBundle::new_concurrent(vec![locale.parse()?]);
          for r in res {
            bundle
              .add_resource(r)
              .map_err(|e| format!("Failed to bundle resource for locale {locale}: {:?}", e))?;
          }
          bundles.insert(locale, bundle);
        }
        LOCALE.set(FluentBundles {
          bundles,
          default: this.default,
        })?;
        Ok(None)
      }))
    });
    Ok(())
  }
}

fn load_resources(res: &mut FluentResources) -> R {
  log::info!("Loading default locale resources");
  for path in Locale::iter().filter(|n| n.ends_with(".ftl")) {
    let locale = path
      .split("/")
      .next()
      .ok_or("Failed to parse locale name")?
      .trim_end_matches(".ftl")
      .to_string();
    if !res.contains_key(&locale) {
      log::trace!("Initializing empty locale {locale}");
      res.insert(locale.clone(), vec![]);
    }
    let file = Locale::get(&path).ok_or("Locale {locale} from {path} could not be found")?;
    res
      .get_mut(&locale)
      .ok_or("Could not get {locale} from FluentResources")?
      .push(
        FluentResource::try_new(String::from_utf8(file.data.to_vec())?)
          .map_err(|(_, e)| format!("Failed to parse locale {locale} from {path}: {:?}", e))?,
      );
  }
  Ok(())
}

pub fn localize<'a>(
  bun: &FluentBundle,
  id: &str,
  attr: Option<&str>,
  args: Option<&FluentArgs<'_>>,
) -> Option<String> {
  let message = bun.get_message(id)?;
  let pattern = match attr {
    Some(attribute) => message.get_attribute(attribute)?.value(),
    None => message.value()?,
  };
  Some(bun.format_pattern(pattern, args, &mut vec![]).into())
}
