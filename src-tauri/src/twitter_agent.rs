use twitter_v2::authorization::Oauth2Token;
use serde::{Deserialize, Serialize};

use crate::scheduler;
use crate::twitter_authorizator;
use crate::twitter_client;

use tauri::Manager;

//const QUEUE_LENGTH : usize = 24;
//const QUEUE_LENGTH: usize = 512;
const QUEUE_LENGTH: usize = 64;
const REQUEST_PERIOD: u64 = 10000; // milliseconds

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Timeline {
    User,
    Search {query: String},
}

fn emit_clear_error(
        app_handle: &tauri::AppHandle,
    ) {

    app_handle
        .emit_all("tauri://frontend/authorization-failed", "")
        .unwrap();

    app_handle
        .emit_all("tauri://frontend/other-error", "")
        .unwrap();
}

fn emit_error_other(
        app_handle: &tauri::AppHandle,
    ) {

    app_handle
        .emit_all(
            "tauri://frontend/other-error",
            "ネットワークに異常があります",
            )
        .unwrap();
}

async fn request_search_timeline(
                        app_handle: &tauri::AppHandle,
                        ctx: &mut SearchTimelineContext,
                        token_opt: &mut Option<Oauth2Token>,
                        ) -> Vec<scheduler::Record>
{

    let mut ret: Vec::<scheduler::Record> = vec![];
    let mut tweets =
        match twitter_client::request_search(&token_opt.clone().unwrap(),
                                                ctx.query_opt.as_ref().unwrap().as_str(),
                                                ctx.since_id_opt.as_ref().map(|s| s.as_str())).await {
            Ok(t) => {
                emit_clear_error(&app_handle);
                t
            }

            Err(e) => match e {
                twitter_client::RequestError::Unauthorized => {
                    println!("twitter_agent: unauthorized {:?}", e);

                    *token_opt = None;
                    return vec![];
                }

                twitter_client::RequestError::Unknown(msg) => {
                    println!("twitter_agent: unknown {:?}", msg);

                    emit_error_other(&app_handle);

                    return vec![];
                }
            },
        };

    if tweets["meta"]["result_count"].as_u64().unwrap() > 0 {
        println!("{:?}", tweets);

        let users = tweets["includes"]["users"].clone();
        let media = tweets["includes"]["media"].clone();

        let mut rev_data = tweets["data"].as_array_mut().unwrap();
        rev_data.reverse();
        for tweet in rev_data {
            ctx.since_id_opt = Some(tweet["id"].as_str().unwrap().to_string());

            let empty_vec = Vec::new();
            let record: scheduler::Record = scheduler::Record::from_tweet(&tweet, &users.as_array().unwrap(), &media.as_array().unwrap_or(&empty_vec)).unwrap();
            ret.push(record)
        }
    }

    ret
}

async fn request_user_timeline(
                        app_handle: &tauri::AppHandle,
                        usrctx: &mut UserTimelineContext,
                        token_opt: &mut Option<Oauth2Token>,
                        ) -> Vec<scheduler::Record>
{

    println!("user_id_opt {:?}", usrctx.user_id_opt);
    if usrctx.user_id_opt.is_none() {
        usrctx.user_id_opt = match twitter_client::request_user_id(&token_opt.clone().unwrap()).await {
            Ok(t) => {
                Some(t)
            }
            Err(e) => match e { 
                twitter_client::RequestError::Unauthorized => {
                    println!("twitter_agent: user id error unauthorized  {:?}", e);

                    *token_opt = None;
                    return vec![];
                }

                twitter_client::RequestError::Unknown(msg) => {
                    println!("twitter_agent: user id error unknown  {:?}", msg);

                    emit_error_other(&app_handle);
                    return vec![];
                }
            }
        };
    }

    println!("user_id_opt {:?}", usrctx.user_id_opt);

    let mut ret: Vec::<scheduler::Record> = vec![];
    if usrctx.user_id_opt.is_some() {
        let mut tweets =
            match twitter_client::request_tweet_new(&token_opt.clone().unwrap(), usrctx.user_id_opt.clone().unwrap().as_str(), usrctx.since_id_opt.as_ref().map(|s| s.as_str())).await {
                //let tweets = match twitter_client::request_user_timeline(&token, user_id.as_str(), start_time).await {
                Ok(t) => {
                    emit_clear_error(&app_handle);
                    t
                }

                Err(e) => match e {
                    twitter_client::RequestError::Unauthorized => {
                        println!("twitter_agent: unauthorized {:?}", e);

                        *token_opt = None;
                        return vec![];
                    }

                    twitter_client::RequestError::Unknown(msg) => {
                        println!("twitter_agent: unknown {:?}", msg);

                        emit_error_other(&app_handle);

                        return vec![];
                    }
                },
            };

        if tweets["meta"]["result_count"].as_u64().unwrap() > 0 {
            println!("{:?}", tweets);

            let users = tweets["includes"]["users"].clone();
            let media = tweets["includes"]["media"].clone();

            let mut rev_data = tweets["data"].as_array_mut().unwrap();
            rev_data.reverse();
            for tweet in rev_data {
                usrctx.since_id_opt = Some(tweet["id"].as_str().unwrap().to_string());

                let empty_vec = Vec::new();
                let record: scheduler::Record = scheduler::Record::from_tweet(&tweet, &users.as_array().unwrap(), &media.as_array().unwrap_or(&empty_vec)).unwrap();
                ret.push(record)
            }
        }
    }

    ret
}

struct SearchTimelineContext {
    query_opt: Option<String>,
    since_id_opt: Option<String>
}

impl SearchTimelineContext {
    pub fn new() -> Self {
        Self {
            query_opt: Some("#Twitter".to_string()),
            since_id_opt: None,
        }
    }
}

struct UserTimelineContext {
    user_id_opt: Option<String>,
    since_id_opt: Option<String>
}

impl UserTimelineContext {
    pub fn new() -> Self {
        Self {
            user_id_opt: None,
            since_id_opt: None,
        }
    }
}


pub fn start(
    app_handle: tauri::AppHandle,
    authctl_tx: tokio::sync::mpsc::Sender<twitter_authorizator::AuthControl>,
    mut token_rx: tokio::sync::mpsc::Receiver<Oauth2Token>,
    mut timeline_rx: tokio::sync::mpsc::Receiver<Timeline>,
) -> (tokio::sync::mpsc::Receiver<scheduler::Record>, tokio::sync::mpsc::Receiver<(Timeline, scheduler::Record)>) {

    let (user_tl_tx, user_tl_rx) = tokio::sync::mpsc::channel(QUEUE_LENGTH);
    let (search_tl_tx, search_tl_rx) = tokio::sync::mpsc::channel(QUEUE_LENGTH);

    // Operating clock
    let (clk_tx, mut clk_rx) = tokio::sync::mpsc::channel::<()>(1);
    let clk_tx_c = clk_tx.clone();
    tokio::spawn(async move {
        loop {
            let _ = clk_tx.send(()).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(REQUEST_PERIOD)).await;
        }
    });

    tokio::spawn(async move {
        let mut token_opt: Option<Oauth2Token> = None;
        let mut usrctx: UserTimelineContext = UserTimelineContext::new();
        let mut search_ctx: SearchTimelineContext = SearchTimelineContext::new();
        let mut timeline: Timeline = Timeline::User;

        token_opt = Some(token_rx.recv().await.unwrap());
        loop {
            tokio::select! {
                Some(t) = token_rx.recv() => {
                    token_opt = Some(t);
                }

                Some(_) = clk_rx.recv() => {
                    println!("clk_rx");
                    if token_opt.is_none() {
                        continue;
                    }

                    let records = match timeline {
                        Timeline::User => {
                            request_user_timeline(&app_handle, &mut usrctx, &mut token_opt).await
                        }

                        Timeline::Search{ref query} => {
                            println!("twitter_agent: {:?}", query);
                            if search_ctx.query_opt.clone().is_none()
                                || search_ctx.query_opt.clone().unwrap() != query.to_string() {
                                search_ctx = SearchTimelineContext::new();
                                search_ctx.query_opt = Some(query.clone());
                            }

                            request_search_timeline(&app_handle, &mut search_ctx, &mut token_opt).await
                        }
                    };

                    if token_opt.is_none() {
                        authctl_tx
                            .send(twitter_authorizator::AuthControl::Authorize)
                            .await
                            .unwrap();

                        continue;
                    }

                    for r in records {
                        match timeline {
                            Timeline::User => { user_tl_tx.try_send(r); }
                            Timeline::Search{ref query} => { search_tl_tx.try_send((timeline.clone(), r)); }
                        }
                    }
                }

                Some(tl) = timeline_rx.recv() => {
                    println!("timeline: {:?}", tl);
                    timeline = tl;
                    clk_tx_c.send(()).await;
                }
            }
        }
    });

    (user_tl_rx, search_tl_rx)
}
