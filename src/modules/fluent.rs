// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::core::*;
use fluent::{bundle::FluentBundle as GenericFluentBundle, FluentArgs, FluentResource};
use intl_memoizer::concurrent::IntlLangMemoizer;
use rust_embed::RustEmbed;
use std::collections::HashMap;

pub type FluentResources = HashMap<String, Vec<FluentResource>>;
pub type FluentBundle = GenericFluentBundle<FluentResource, IntlLangMemoizer>;

pub struct FluentBundles {
  pub bundles: HashMap<String, FluentBundle>,
  pub default: String,
}

#[derive(RustEmbed)]
#[folder = "locale/"]
struct Locale;

pub struct Fluent {
  resources: FluentResources,
  default: String,
}

impl Default for Fluent {
  fn default() -> Self {
    Self {
      resources: FluentResources::new(),
      default: "en-US".into(),
    }
  }
}

pub fn init() -> Res<FluentBundles> {
  let mut fr = FluentResources::new();
  load_resources(&mut fr)?;
  let mut bundles = HashMap::new();
  for (locale, res) in fr {
    let mut bundle = FluentBundle::new_concurrent(vec![locale.parse()?]);
    for r in res {
      bundle
        .add_resource(r)
        .map_err(|e| format!("Failed to bundle resource for locale {locale}: {:?}", e))?;
    }
    bundles.insert(locale, bundle);
  }
  Ok(FluentBundles {
    bundles,
    default: "en-US".into(),
  })
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
