// Copyright 2023 Atakku <https://atakku.dev>
//
// This project is dual licensed under MIT and Apache.

use crate::{
  core::*,
  modules::fluent::{loc, localize, Fluent, FluentBundle, FluentBundles},
};
use futures::future::join_all;
use poise::{
  serenity_prelude::{Context as SCtx, GatewayIntents},
  BoxFuture, Command, Context, Event, FrameworkContext, FrameworkOptions,
};

pub type Fw = poise::Framework<Vec<EventHandler>, Err>;
pub type FwCtx<'a> = FrameworkContext<'a, Vec<EventHandler>, Err>;
pub type Ctx<'a> = Context<'a, Vec<EventHandler>, Err>;
pub type Cmd = Command<Vec<EventHandler>, Err>;
pub type EventHandler = for<'a> fn(&'a SCtx, &'a Event<'a>) -> BoxFuture<'a, R>;

// TODO: add documentation,
// Poise wrapper module, to let other modules add commands and subscribe to events easily

module! {
  Poise {
    token: String = env!("DISCORD_TOKEN"),
    intents: GatewayIntents = GatewayIntents::GUILD_MESSAGES,
    commands: Vec<Cmd>,
    event_handlers: Vec<EventHandler>,
  }

  fn init(fw) {
    fw.req::<Fluent>()?;
    rt!(fw, |poise| {
      Ok(Some(tokio::spawn(async move {
        Fw::builder()
          .token(poise.token)
          .intents(poise.intents)
          .options(FrameworkOptions {
            commands: localized_commands(poise.commands, loc()),
            event_handler: |c, e, _f, ehs| {
              Box::pin(async move {
                join_all(ehs.iter().map(|eh| (eh)(c, e))).await;
                Ok(())
              })
            },
            ..Default::default()
          })
          .setup(move |_c, _r, _f| {
            Box::pin(async move { Ok(poise.event_handlers) })
          })
          .run()
          .await?;
        Ok(())
      })))
    });
  }

  pub fn add_command(&mut self, cmd: Cmd) {
    self.commands.push(cmd);
  }

  pub fn add_commands(&mut self, mut cmd: Vec<Cmd>) {
    self.commands.append(&mut cmd);
  }

  pub fn add_event_handler(&mut self, eh: EventHandler) {
    self.event_handlers.push(eh);
  }

  pub fn add_intent(&mut self, intent: GatewayIntents) {
    self.intents.insert(intent);
  }
}

const LOCALES: [&str; 31] = [
  "id", "da", "de", "en-GB", "en-US", "es-ES", "fr", "hr", "it", "lt", "hu", "nl", "no", "pl",
  "pt-BR", "ro", "fi", "sv-SE", "vi", "tr", "cs", "el", "bg", "ru", "uk", "hi", "th", "zh-CN",
  "ja", "zh-TW", "ko",
];

fn localized_commands(mut commands: Vec<Cmd>, fb: &FluentBundles) -> Vec<Cmd> {
  if let Some(bun) = fb.bundles.get(&fb.default) {
    for loc in LOCALES {
      log::trace!("Defaulting locale '{loc}' from '{}'", fb.default);
      for cmd in &mut commands {
        log::trace!(
          "Defaulting locale '{loc}' for '{}' from '{}'",
          cmd.name,
          fb.default
        );
        localize_cmd(cmd, loc, bun, None, loc == fb.default)
      }
    }
  } else {
    log::warn!("Default locale '{}' was not found", fb.default);
  }
  for (loc, bun) in fb
    .bundles
    .iter()
    .filter(|(l, _)| LOCALES.contains(&l.as_str()) && *l != &fb.default)
  {
    log::info!("Applying locale '{loc}'");
    for cmd in &mut commands {
      log::trace!("Applying locale '{loc}' to '{}'", cmd.name);
      localize_cmd(cmd, loc, bun, None, true)
    }
  }
  commands
}

fn localize_cmd(
  cmd: &mut Cmd,
  loc: &str,
  fb: &FluentBundle,
  parent_path: Option<&str>,
  log_missing: bool,
) {
  let path = format!("{}_{}", parent_path.unwrap_or("cmd"), cmd.name);
  // Skip trying to localize group commands
  if !cmd.subcommand_required {
    if let Some(name) = get_loc(loc, fb, &path, None, true, log_missing) {
      cmd.name_localizations.insert(loc.into(), name.into());
    }
    if let Some(desc) = get_loc(loc, fb, &path, Some("desc"), false, log_missing) {
      cmd
        .description_localizations
        .insert(loc.into(), desc.into());
    }
    for prm in &mut cmd.parameters {
      let prm_path = format!("prm_{}", &prm.name);
      if let Some(name) = get_loc(loc, fb, &path, Some(&prm_path), true, log_missing) {
        prm.name_localizations.insert(loc.into(), name.into());
      }
      if let Some(desc) = get_loc(
        loc,
        fb,
        &path,
        Some(&format!("{prm_path}_desc")),
        false,
        log_missing,
      ) {
        prm
          .description_localizations
          .insert(loc.into(), desc.into());
      }
      for cho in &mut prm.choices {
        let path = format!("cho_{}", &prm.name);
        if let Some(name) = get_loc(loc, fb, &path, Some(&cho.name), false, log_missing) {
          cho.localizations.insert(loc.into(), name.into());
        }
      }
    }
  }
  for sub in &mut cmd.subcommands {
    localize_cmd(sub, loc, fb, Some(&path), log_missing);
  }
}

fn get_loc<'a>(
  loc: &str,
  bun: &FluentBundle,
  path: &str,
  attr: Option<&str>,
  check_lowercase: bool,
  warn_missing: bool,
) -> Option<String> {
  let log_path = attr
    .and_then(|a| Some(format!("{path}.{a}")))
    .unwrap_or(path.into());
  if let Some(localized) = localize(bun, path, attr, None) {
    if !check_lowercase || localized.chars().all(char::is_lowercase) {
      return Some(localized);
    } else {
      log::error!("Locale '{loc}' contains uppercase characters in '{log_path}'")
    }
  } else if warn_missing {
    log::warn!("Locale '{loc}' is missing '{log_path}'")
  }
  return None;
}

macro_rules! commands {
  (@internal $(#[$m:meta])* fn $name:ident($ctx:ident$(,$($tt:tt)*)?) $block:block) => {
    $(#[$m])*
    pub async fn $name($ctx: crate::modules::poise::Ctx<'_>, $($($tt)*)?) -> $crate::core::R {wrap!($block)}
  };
  (@internal $(#[$m:meta])* fn $name:ident($ctx:ident$(,$($tt:tt)*)?) $(-> BasicCommand)? $block:block) => {
    #[poise::command(prefix_command, slash_command)]
    $(#[$m])*
    pub async fn $name($ctx: crate::modules::poise::Ctx<'_>, $($($tt)*)?) -> $crate::core::R {wrap!($block)}
  };
  (@internal $(#[$m:meta])* fn $name:ident($ctx:ident$(,$($tt:tt)*)?) $(-> OwnerCommand)? $block:block) => {
    #[poise::command(prefix_command, owners_only)]
    $(#[$m])*
    pub async fn $name($ctx: crate::modules::poise::Ctx<'_>, $($($tt)*)?) -> $crate::core::R {wrap!($block)}
  };
  ($($(#[$m:meta])* fn $name:ident($ctx:ident$(,$($tt:tt)*)?) $(-> $type:ident)? $block:block)*) => {
    $(commands!(@internal $(#[$m])* fn $name($ctx$(,$($tt)*)?) $(-> $type)? $block);)*
  };
}

macro_rules! lim_choice {
  ($enum:ident, $from:ident, [$($param:ident),*]) => {
    #[derive(poise::ChoiceParameter)]
    pub enum $enum {
      $($param),*
    }

    impl Into<$from> for $enum {
      fn into(self) -> $from {
        match self {
          $($enum::$param => $from::$param),*
        }
      }
    }
  };
}
