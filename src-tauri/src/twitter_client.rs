use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RevocationUrl, Scope, TokenUrl,
};

use twitter_v2::TwitterApi;
use twitter_v2::authorization::Oauth2Token;
use twitter_v2::error::Result;
use twitter_v2::data::{User, Tweet};

use reqwest::Url;

struct Oauth2Ctx {
    oauth2client: BasicClient,
    verifier: Option<PkceCodeVerifier>,
    state: Option<CsrfToken>,
    token: Option<Oauth2Token>,
}

fn callback_server() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 41157))
}

fn base_url() -> Url {
    Url::parse("https://api.twitter.com/2/").unwrap()
}

pub fn entrypoint_url() -> String {
    format!("http://{}/login", callback_server())
}

impl Oauth2Ctx {
    pub fn new() -> Self {

        // serve on port 3000
        let client_id = ClientId::new("YkxNZ3ZDNzU4Q1ZNdEJfd0U2cFg6MTpjaQ".to_string());
        let addr           = callback_server();
        let redirect_url   = RedirectUrl::from_url(format!("http://{addr}/callback").parse().unwrap());
        let auth_url       = AuthUrl::from_url("https://twitter.com/i/oauth2/authorize".parse().unwrap());
        let token_url      = TokenUrl::from_url("https://api.twitter.com/2/oauth2/token".parse().unwrap());
        let revocation_url = RevocationUrl::from_url("https://api.twitter.com/2/oauth2/revoke".parse().unwrap());

        Self {
            oauth2client: BasicClient::new(client_id, None, auth_url, Some(token_url))
                .set_revocation_uri(revocation_url)
                .set_redirect_uri(redirect_url),
            verifier: None,
            state: None,
            token: None,
        }
    }
}

async fn login(Extension(ctx): Extension<Arc<Mutex<Oauth2Ctx>>>) -> impl IntoResponse {
    let mut ctx = ctx.lock().unwrap();
    // create challenge
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    // create authorization url
    let (url, state) = ctx.oauth2client.authorize_url(CsrfToken::new_random)
            .set_pkce_challenge(challenge)
                // Set the desired scopes.
            .add_scope(Scope::new("tweet.read".to_string()))
            .add_scope(Scope::new("users.read".to_string()))
            .add_scope(Scope::new("offline.access".to_string()))
            .url();

    // set context for reference in callback
    ctx.verifier = Some(verifier);
    ctx.state = Some(state);
    // redirect user
    Redirect::to(&url.to_string())
}

#[derive(Deserialize)]
pub struct CallbackParams {
    code: AuthorizationCode,
    state: CsrfToken,
}

async fn callback(
    Extension(ctx): Extension<Arc<Mutex<Oauth2Ctx>>>,
    Query(CallbackParams { code, state }): Query<CallbackParams>,
) -> impl IntoResponse {
    let (client, verifier) = {
        let mut ctx = ctx.lock().unwrap();
        // get previous state from ctx (see login)
        let saved_state = ctx.state.take().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No previous state found".to_string(),
            )
        })?;
        // // check state returned to see if it matches, otherwise throw an error
        if state.secret() != saved_state.secret() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid state returned".to_string(),
            ));
        }
        // // get verifier from ctx
        let verifier = ctx.verifier.take().ok_or_else(|| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "No PKCE verifier found".to_string(),
            )
        })?;
        let client = ctx.oauth2client.clone();
        (client, verifier)
    };

    // request oauth2 token
    let token = client.exchange_code(code)
            .set_pkce_verifier(verifier)
            .request_async(oauth2::reqwest::async_http_client)
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
            .unwrap();

    let t: Oauth2Token = token.try_into().unwrap();

    // set context for use with twitter API
    let mut ctx = ctx.lock().unwrap();
    ctx.token = Some(t);

    Ok(Redirect::to("/tweets"))
}

async fn refresh_token_if_expired(client: & BasicClient, token: &mut Oauth2Token) -> Result<bool, &'static str> {
    if token.is_expired() {
        if let Some(refresh_token) = token.refresh_token() {
            let token_res = client.exchange_refresh_token(refresh_token)
                .request_async(oauth2::reqwest::async_http_client)
                .await
                .unwrap();

            *token = token_res.try_into().unwrap();
            Ok(true)
        } else {
            Err("No Refresh token found")
        }
    } else {
        Ok(false)
    }
}

async fn tweets(Extension(ctx): Extension<Arc<Mutex<Oauth2Ctx>>>) -> impl IntoResponse {
    // get oauth token
    let (mut oauth_token, oauth_client) = {
        let ctx = ctx.lock().unwrap();
        let token = ctx
            .token
            .as_ref()
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "User not logged in!".to_string()))?
            .clone();
        let client = ctx.oauth2client.clone();
        (token, client)
    };

    // refresh oauth token if expired
    if refresh_token_if_expired(&oauth_client, &mut oauth_token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        // save oauth token if refreshed
        ctx.lock().unwrap().token = Some(oauth_token.clone());
    }

    let api = TwitterApi::new(oauth_token);
    // get tweet by id
    let tweet = api
        .get_tweet(20)
        .send()
        .await
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    Ok::<_, (StatusCode, String)>(Json(tweet.into_data()))
}

#[derive(Debug, Deserialize)]
struct TwitterResponse<T> {
    data: T
}

async fn tweets2(Extension(ctx): Extension<Arc<Mutex<Oauth2Ctx>>>) -> impl IntoResponse {
    // get oauth token
    let (mut oauth_token, oauth_client) = {
        let ctx = ctx.lock().unwrap();
        let token = ctx
            .token
            .as_ref()
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, "User not logged in!".to_string()))?
            .clone();
        let client = ctx.oauth2client.clone();
        (token, client)
    };

    // refresh oauth token if expired
    if refresh_token_if_expired(&oauth_client, &mut oauth_token)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
    {
        // save oauth token if refreshed
        ctx.lock().unwrap().token = Some(oauth_token.clone());
    }

    let client = reqwest::Client::new();
    let auth_val = format!("Bearer {}", oauth_token.access_token().secret());

    let url = base_url().join("users/me").unwrap();
    let user_id: TwitterResponse<User> = client.get(url)
                    .header(reqwest::header::AUTHORIZATION, auth_val.clone())
                    .send()
                    .await.unwrap()
                    .json()
                    .await.unwrap();

    let user_id = user_id.data.id;
    let url = base_url().join(format!("users/{user_id}/timelines/reverse_chronological").as_str()).unwrap();
    let timeline: TwitterResponse<Vec<Tweet>> = client.get(url)
                    .header(reqwest::header::AUTHORIZATION, auth_val.clone())
                    .send()
                    .await.unwrap()
                    .json()
                    .await.unwrap();

    // get tweet by id
    //let url = base_url().join("users/{user_id}/timelines/reverse_chronological").unwrap();
    let mut texts = Vec::<String>::new();
    for tweet in timeline.data {
        texts.push(tweet.text);
    }
    let json = serde_json::to_string(&texts).unwrap();

    Ok::<_, (StatusCode, String)>(json)
}

async fn revoke(Extension(ctx): Extension<Arc<Mutex<Oauth2Ctx>>>) -> impl IntoResponse {
    //// get oauth token
    //let (oauth_token, oauth_client) = {
    //    let ctx = ctx.lock().unwrap();
    //    let token = ctx
    //        .token
    //        .as_ref()
    //        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "User not logged in!".to_string()))?
    //        .clone();
    //    let client = ctx.oauth2client.clone();
    //    (token, client)
    //};
    //// revoke token
    //oauth_client
    //    .revoke_token(oauth_token.revokable_token())
    //    .await
    //    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;
    //Ok::<_, (StatusCode, String)>("Token revoked!")
}

async fn debug_token(Extension(ctx): Extension<Arc<Mutex<Oauth2Ctx>>>) -> impl IntoResponse {
    // get oauth token
    let oauth_token = ctx
        .lock()
        .unwrap()
        .token
        .as_ref()
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "User not logged in!".to_string()))?
        .clone();
    // get underlying token
    Ok::<_, (StatusCode, String)>(Json(oauth_token))
}

pub async fn start_server() {
    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "oauth2_callback=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // initialize Oauth2Client with ID and Secret and the callback to this server
    let oauth_ctx = Oauth2Ctx::new();

    // initialize server
    let app = Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/tweets", get(tweets))
        .route("/tweets2", get(tweets2))
        .route("/revoke", get(revoke))
        .route("/debug_token", get(debug_token))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(Arc::new(Mutex::new(oauth_ctx))));

    // run server
    let addr = callback_server();
    println!("\nOpen http://{}/login in your browser\n", addr);
    tracing::debug!("Serving at {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
