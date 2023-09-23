use super::{axum::Axum, sqlx::db};
use axum::{
  response::{IntoResponse, Redirect, Response},
  routing::get,
  Form, http::HeaderValue,
};
use axum_session::{SessionConfig, SessionLayer, SessionPgSession, SessionPgSessionStore};
use regex::Regex;
use reqwest::{StatusCode, header};
use sea_query::{InsertStatement, Query, QueryStatement, SelectStatement, OnConflict};
use url::Url;

async fn settings(session: SessionPgSession) -> Response {
  let Some(id) = session.get::<i32>("neko_id") else {
    return Redirect::to("/login").into_response();
  };
  let mut res =format!("your id is '{id}'<br><a href=\"/link/discord\">link a discord acc</a><br><a href=\"/link/steam\">link a steam acc</a>").into_response();
  res.headers_mut().append(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
  res
}

async fn root(session: SessionPgSession) -> Response {
  let mut res = match session.get::<i32>("neko_id") {
    Some(id) => {
      format!("you are logged in as id {id}<br><a href=\"/settings\">go to settings</a>").into_response()
    }
    None => format!("you are not logged in<br><a href=\"/login\">i wanna log in</a>").into_response(),
  };
  res.headers_mut().append(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
  res
}
async fn logout(session: SessionPgSession) -> Response {
  match session.get::<i32>("neko_id") {
    Some(_) => {
      session.destroy();
      Redirect::to("/").into_response()
    }
    None => {let mut res = format!("you are not logged in lmao").into_response();
    res.headers_mut().append(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
    res}
  }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RedirForm {
  redirect: Option<String>,
}

async fn login_now(session: SessionPgSession) -> Response {
  match session.get::<i32>("neko_id") {
    Some(id) => {let mut res = format!(
      "you are already logged in, and your id is '{id}'<br><a href=\"/logout\">log me out!!!!!</a>"
    )
    .into_response();
    res.headers_mut().append(header::CONTENT_TYPE, HeaderValue::from_static("text/html"));
    res},
    None => Redirect::to("/link/discord").into_response(),
  }
}

once_cell!(sid_regex, REGEX: Regex);

module!(
  Gwaaa {}

  fn init(fw) {
    fw.req_module::<crate::modules::sqlx::Postgres>()?;
    REGEX.set(Regex::new("^https://steamcommunity.com/openid/id/([0-9]{17})$")?)?;


    let axum = fw.req_module::<Axum>()?;

    axum.routes.push(|r| {
      Box::pin(async move {
        let session_config = SessionConfig::default().with_table_name("neko_users_sessions");
        let session_store = SessionPgSessionStore::new(Some(db().clone().into()), session_config)
            .await
            .unwrap();

          Ok(r.route("/", get(root)).route("/login", get(login_now))
          .route("/logout", get(logout))
          .route("/settings", get(settings))
          .route("/callback/steam", get(callback_steam))
          .route("/callback/discord", get(callback_discord))
          .route("/link/steam", get(link_steam))
          .route("/link/discord", get(link_discord))
        .layer(SessionLayer::new(session_store)))
      })
    });
  }
);

// who needs actual error handling tbh
async fn callback_steam(
  session: SessionPgSession,
  Form(cb): Form<VerifyForm>,
) -> axum::response::Result<Response> {
  let Some(id) = session.get::<i32>("neko_id") else {
    return Ok(Redirect::to("/login?redirect=%2Fsettings").into_response());
  };
  let mut validate = cb;
  validate.mode = "check_authentication".to_owned();
  let form_str =
    serde_urlencoded::to_string(&validate).unwrap();

  let client = reqwest::Client::new();
  let response = client
    .post("https://steamcommunity.com/openid/login")
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(form_str)
    .send()
    .await
    .unwrap()
    .text()
    .await
    .unwrap();

  let is_valid = response.split('\n').any(|line| line == "is_valid:true");
  if !is_valid {
    return Err("NOT VALID GWAAAA".into());
  }

  let captures = sid_regex()
    .captures(&validate.claimed_id)
    .unwrap();
  let steam_id = captures
    .get(1)
    .unwrap()
    .as_str()
    .parse::<i64>()
    .unwrap();
  use crate::schema::neko::UsersSteam::*;
  let mut qb = InsertStatement::new();
  qb.into_table(Table);
  qb.columns([NekoId, SteamId]);
  qb.values([id.into(), steam_id.into()])
    .unwrap();
  qb.on_conflict(OnConflict::column(SteamId).update_column(NekoId).to_owned());
  execute!(&qb).unwrap();
  Ok(Redirect::to("/settings").into_response())
}

const DOMAIN: &str = "https://link.neko.rs";

async fn link_steam() -> axum::response::Result<Response> {
  let form = RedirectForm {
    ns: "http://specs.openid.net/auth/2.0",
    identity: "http://specs.openid.net/auth/2.0/identifier_select",
    claimed_id: "http://specs.openid.net/auth/2.0/identifier_select",
    mode: "checkid_setup",
    realm: DOMAIN,
    return_to: &format!("{DOMAIN}/callback/steam"),
  };
  let form_str = serde_urlencoded::to_string(&form).unwrap();

  let mut redirect_url = Url::parse("https://steamcommunity.com/openid/login").unwrap();
  redirect_url.set_query(Some(&form_str));
  Ok(Redirect::to(redirect_url.as_str()).into_response())
}

async fn link_discord(session: SessionPgSession) -> axum::response::Result<Response> {
  let form = RedirectDiscord {
    response_type: "code",
    client_id: "1064379551318278204",
    scope: "identify",
    state: "todo",
    redirect_uri: format!("{DOMAIN}/callback/discord"),
    prompt: "consent",
  };
  let form_str = serde_urlencoded::to_string(&form).unwrap();

  let mut redirect_url = Url::parse("https://discord.com/oauth2/authorize").unwrap();
  redirect_url.set_query(Some(&form_str));
  Ok(Redirect::to(redirect_url.as_str()).into_response())
}

async fn callback_discord(
  session: SessionPgSession,
  Form(cb): Form<DiscordCallback>,
) -> axum::response::Result<Response> {
  if cb.state != "todo" {
    return Ok(StatusCode::IM_A_TEAPOT.into_response());
  }
  let form_str = serde_urlencoded::to_string(&DiscordTokenReq {
    client_id: &"1064379551318278204",
    client_secret: &expect_env!("DISCORD_SECRET"),
    grant_type: &"authorization_code",
    code: &cb.code,
    redirect_uri: &format!("{DOMAIN}/callback/discord"),
  }).unwrap();

  let client = reqwest::Client::new();
  let response = client
    .post("https://discord.com/api/v10/oauth2/token")
    .header("Content-Type", "application/x-www-form-urlencoded")
    .body(form_str)
    .send()
    .await
    .unwrap();
  let response = response
    .json::<DiscordTokenRes>()
    .await
    .unwrap();

  let response = client
    .get("https://discord.com/api/v10/oauth2/@me")
    .header("Content-Type", "application/json")
    .bearer_auth(response.access_token)
    .send()
    .await
    .unwrap()
    .json::<DiscordAuthRes>()
    .await
    .unwrap();

  if let None = response.user {
    return Err("NOT VALID GWAAAA".into());
  }

  let did = response.user.unwrap().id.parse::<i64>().unwrap();

  let id = match session.get::<i32>("neko_id") {
    None => {
      use crate::schema::neko::UsersDiscord;
      let mut qb = SelectStatement::new();
      qb.from(UsersDiscord::Table);
      qb.column(UsersDiscord::NekoId);
      qb.and_where(ex_col!(UsersDiscord, DiscordId).eq(did));
      let newid = match fetch_optional!(&qb, (i32,)).unwrap() {
        Some(id) => id.0,
        None => {
          use crate::schema::neko::Users::*;
          let mut qb = InsertStatement::new();
          qb.into_table(Table);
          qb.columns([Slug]);
          qb.values([Option::<String>::None.into()]).unwrap();
          qb.returning(Query::returning().columns([Id]));
          fetch_one!(&qb, (i32,))
            .unwrap()
            .0
        }
      };
      session.set("neko_id", newid);
      newid
    }
    Some(id) => id,
  };
  use crate::schema::neko::UsersDiscord::*;
  let mut qb = InsertStatement::new();
  qb.into_table(Table);
  qb.columns([NekoId, DiscordId]);
  qb.values([id.into(), did.into()])
    .unwrap();
    qb.on_conflict(OnConflict::column(DiscordId).update_column(NekoId).to_owned());
  execute!(&qb).unwrap();
  Ok(Redirect::to("/settings").into_response())
}

#[derive(serde::Serialize)]
struct RedirectDiscord<'a> {
  response_type: &'static str,
  client_id: &'static str,
  scope: &'static str,
  state: &'a str,
  redirect_uri: String,
  prompt: &'a str,
}

#[derive(serde::Deserialize)]
struct DiscordCallback {
  code: String,
  state: String,
}

#[derive(serde::Serialize)]
struct DiscordTokenReq<'a> {
  client_id: &'a str,
  client_secret: &'a str,
  grant_type: &'a str,
  code: &'a str,
  redirect_uri: &'a str,
}
#[derive(serde::Deserialize)]
struct DiscordTokenRes {
  access_token: String
}

#[derive(serde::Deserialize)]
struct DiscordAuthRes {
  user: Option<DiscordUser>,
}
#[derive(serde::Deserialize)]
struct DiscordUser {
  id: String,
}

#[derive(serde::Serialize)]
struct RedirectForm<'a> {
  #[serde(rename = "openid.ns")]
  ns: &'static str,
  #[serde(rename = "openid.identity")]
  identity: &'static str,
  #[serde(rename = "openid.claimed_id")]
  claimed_id: &'static str,
  #[serde(rename = "openid.mode")]
  mode: &'static str,
  #[serde(rename = "openid.return_to")]
  return_to: &'a str,
  #[serde(rename = "openid.realm")]
  realm: &'a str,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct VerifyForm {
  #[serde(rename = "openid.ns")]
  ns: String,
  #[serde(rename = "openid.mode")]
  mode: String,
  #[serde(rename = "openid.op_endpoint")]
  op_endpoint: String,
  #[serde(rename = "openid.claimed_id")]
  claimed_id: String,
  #[serde(rename = "openid.identity")]
  identity: Option<String>,
  #[serde(rename = "openid.return_to")]
  return_to: String,
  #[serde(rename = "openid.response_nonce")]
  response_nonce: String,
  #[serde(rename = "openid.invalidate_handle")]
  invalidate_handle: Option<String>,
  #[serde(rename = "openid.assoc_handle")]
  assoc_handle: String,
  #[serde(rename = "openid.signed")]
  signed: String,
  #[serde(rename = "openid.sig")]
  sig: String,
}
