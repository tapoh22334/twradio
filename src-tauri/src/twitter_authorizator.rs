use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::trace::TraceLayer;
use tracing_subscriber::prelude::*;

use oauth2::basic::BasicClient;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, CsrfToken, PkceCodeChallenge, PkceCodeVerifier,
    RedirectUrl, RevocationUrl, Scope, TokenUrl,
};

use twitter_v2::authorization::Oauth2Token;
use twitter_v2::error::Result;

use tauri::Manager;

fn callback_server() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 41157))
}

pub fn entrypoint_url() -> String {
    format!("http://{}/login", callback_server())
}

pub fn new_oauth2_client() -> BasicClient {
    let client_id = ClientId::new("YkxNZ3ZDNzU4Q1ZNdEJfd0U2cFg6MTpjaQ".to_string());
    let addr = callback_server();
    let redirect_url = RedirectUrl::from_url(format!("http://{addr}/callback").parse().unwrap());
    let auth_url = AuthUrl::from_url("https://twitter.com/i/oauth2/authorize".parse().unwrap());
    let token_url = TokenUrl::from_url("https://api.twitter.com/2/oauth2/token".parse().unwrap());
    let revocation_url =
        RevocationUrl::from_url("https://api.twitter.com/2/oauth2/revoke".parse().unwrap());

    BasicClient::new(client_id, None, auth_url, Some(token_url))
        .set_revocation_uri(revocation_url)
        .set_redirect_uri(redirect_url)
}

struct Oauth2Ctx {
    oauth2client: BasicClient,
    verifier: Option<PkceCodeVerifier>,
    state: Option<CsrfToken>,
    token: Option<Oauth2Token>,
    token_tx: Option<tokio::sync::oneshot::Sender<Oauth2Token>>,
}

impl Oauth2Ctx {
    pub fn new(tx: tokio::sync::oneshot::Sender<Oauth2Token>) -> Self {
        Self {
            oauth2client: new_oauth2_client(),
            verifier: None,
            state: None,
            token: None,
            token_tx: Some(tx),
        }
    }
}

async fn login(Extension(ctx): Extension<Arc<Mutex<Oauth2Ctx>>>) -> impl IntoResponse {
    let mut ctx = ctx.lock().unwrap();
    // create challenge
    let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
    // create authorization url
    let (url, state) = ctx
        .oauth2client
        .authorize_url(CsrfToken::new_random)
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
    let token = client
        .exchange_code(code)
        .set_pkce_verifier(verifier)
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
        .unwrap();

    let token: Oauth2Token = token.try_into().unwrap();

    // set context for use with twitter API
    {
        let mut ctx = ctx.lock().unwrap();
        ctx.token = Some(token.clone());
        ctx.token_tx.take().unwrap().send(token.clone()).unwrap();
    }

    Ok(Redirect::to("/result"))
}

async fn auth_result(Extension(_ctx): Extension<Arc<Mutex<Oauth2Ctx>>>) -> impl IntoResponse {
    Ok::<_, (StatusCode, String)>("Complete! close this window")
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

pub async fn refresh_token_if_expired(token: &mut Oauth2Token) -> Result<bool, &'static str> {
    let client = new_oauth2_client();
    if token.is_expired() {
        if let Some(refresh_token) = token.refresh_token() {

            if let Ok(token_res) = client
                .exchange_refresh_token(refresh_token)
                .request_async(oauth2::reqwest::async_http_client)
                .await {

                *token = token_res.try_into().unwrap();

                Ok(true)
            } else {
                Err("Failed to requesting refresh token exchange")
            }
        } else {
            Err("No Refresh token found")
        }
    } else {
        Ok(false)
    }
}

pub fn start_server() -> (
    tokio::sync::oneshot::Sender<()>,
    tokio::sync::oneshot::Receiver<Oauth2Token>,
) {
    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "oauth2_callback=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // initialize Oauth2Client with ID and Secret and the callback to this server
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let (token_tx, token_rx) = tokio::sync::oneshot::channel::<Oauth2Token>();

    let oauth_ctx = Oauth2Ctx::new(token_tx);

    // initialize server
    let app = Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/result", get(auth_result))
        .route("/debug_token", get(debug_token))
        .layer(TraceLayer::new_for_http())
        .layer(Extension(Arc::new(Mutex::new(oauth_ctx))));

    // run server
    let addr = callback_server();
    tracing::debug!("Serving at {}", addr);

    tokio::spawn(async move {
        axum::Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(async {
                shutdown_rx.await.ok();
            })
            .await
            .unwrap();
    });

    (shutdown_tx, token_rx)
}

async fn perform_oauth2_flow() -> Oauth2Token {
    let (shutdown_tx, token_rx) = start_server();
    webbrowser::open(entrypoint_url().as_str()).unwrap();

    let t = token_rx.await.ok().unwrap();
    shutdown_tx.send(()).ok().unwrap();
    t
}

async fn get_token_from_storage(app_handle: &tauri::AppHandle) -> Option<Oauth2Token> {
    println!("get token from storage");
    app_handle
        .emit_all("tauri://frontend/token-request", ())
        .unwrap();

    let (token_complete_tx, token_complete_rx) =
        tokio::sync::oneshot::channel::<Option<Oauth2Token>>();
    let token_complete_tx: std::sync::Mutex<
        Option<tokio::sync::oneshot::Sender<Option<Oauth2Token>>>,
    > = std::sync::Mutex::new(Some(token_complete_tx));

    let id = app_handle.listen_global("tauri://backend/token-response", move |event| {
        let t: Option<Oauth2Token> = {
            if let Some(payload) = event.payload() {
                serde_json::from_str(payload).unwrap()
            } else {
                None
            }
        };

        let mut token_complete_tx = token_complete_tx.lock().unwrap();
        token_complete_tx.take().unwrap().send(t).unwrap();
    });

    let t: Option<Oauth2Token> = token_complete_rx.await.unwrap();
    // unlisten to the event using the `id` returned on the `listen_global` function
    // an `once_global` API is also exposed on the `App` struct
    app_handle.unlisten(id);

    t
}

fn save_token_into_storage(app_handle: &tauri::AppHandle, token: Oauth2Token) {
    app_handle
        .emit_all("tauri://frontend/token-register", token)
        .unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthControl {
    Authorize,
}

pub fn start(
    app_handle: tauri::AppHandle,
    mut authctl_rx: tokio::sync::mpsc::Receiver<AuthControl>,
) -> tokio::sync::mpsc::Receiver<Oauth2Token> {
    let (token_tx, token_rx) = tokio::sync::mpsc::channel::<Oauth2Token>(1);

    tokio::spawn(async move {
        loop {
            match authctl_rx.recv().await {
                Some(msg) => match msg {
                    AuthControl::Authorize => {
                        let mut token: Oauth2Token = {
                            if let Some(t) = get_token_from_storage(&app_handle).await {
                                println!("token is already in storage");

                                t
                            } else {
                                println!("Token not found. Request authorization");

                                let t = perform_oauth2_flow().await;
                                save_token_into_storage(&app_handle, t.clone());
                                t
                            }
                        };

                        if let Ok(expired) = refresh_token_if_expired(&mut token).await{
                            if expired {
                                println!("******* Token refreshed *********");
                                save_token_into_storage(&app_handle, token.clone());

                            } else {
                                println!("token is not expired");

                            }
                        } else {
                            // Failed to exchanging refresh token
                            app_handle
                                .emit_all("tauri://frontend/token-unregister", ())
                                .unwrap();

                            ()
                        }

                        token_tx.send(token).await.unwrap();
                    }
                },

                None => {
                    return ();
                }
            }
        }
    });

    token_rx
}
