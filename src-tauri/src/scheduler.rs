use std::collections::LinkedList;
use serde::{Serialize, Deserialize};
use tauri::Manager;

use crate::twitter_data;
use crate::display_bridge;
use crate::voicegen_agent;
use crate::audio_player;
use crate::user_input;
use crate::voicegen_observer;

const QUEUE_LENGTH : usize = 24;
const HISTORY_LENGTH: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub tweet_id: String,
    pub created_at: String,
    pub text: String,
    pub name: String,
    pub username: String,
    pub profile_image_url: String,
}

pub fn into(tweet: &twitter_data::Tweet, users: &Vec<twitter_data::User>) -> Record {
    let user = users.iter().find(|user| user.id == tweet.author_id).unwrap();

    Record {
        tweet_id: tweet.id.clone(),
        created_at: tweet.created_at.clone(),
        text: tweet.text.clone(),
        name: user.name.clone(),
        username: user.username.clone(),
        profile_image_url: user.profile_image_url.clone(),
    }
}

fn wait_both(mut speech_rdy_rx: tokio::sync::mpsc::Receiver<()>,
                   mut audio_rdy_rx: tokio::sync::mpsc::Receiver<audio_player::AudioControlRdy>)
            -> tokio::sync::mpsc::Receiver<()> {
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

    tokio::spawn(async move {
        loop {
            let _ = audio_rdy_rx.recv().await.unwrap();
            let _ = speech_rdy_rx.recv().await.unwrap();
            tx.send(()).await.unwrap();
        }
    });

    rx
}

pub fn remove<T>(list: &mut LinkedList::<T>, index: usize) -> T {
        if index == 0 {
            let v = list.pop_front().unwrap();

            return v;
        } else {
            // split_off function should compute in O(n) time.
            let mut split = list.split_off(index);
            let v = split.pop_front().unwrap();
            list.append(&mut split);

            return v;
        }
}

struct Context {
    pub addr: std::net::SocketAddr,
    pub speaker: u64,
    pub focus_set: bool,
    pub cancelling: bool,
    pub wait_list: LinkedList::<Record>,
    pub ready_list: LinkedList::<Record>,
    pub played_list: LinkedList::<Record>,
    pub processing: Option<Record>,
    pub speech_cache: LinkedList::<voicegen_agent::Speech>,
}

impl Context {
    pub fn new() -> Self {
        Context {
            addr: std::net::SocketAddr::from(([127, 0, 0, 1], 50031)),
            speaker: 7,
            focus_set: false,
            cancelling: false,
            wait_list: LinkedList::<Record>::new(),
            ready_list: LinkedList::<Record>::new(),
            played_list: LinkedList::<Record>::new(),
            processing: Option::<Record>::None,
            speech_cache: LinkedList::<voicegen_agent::Speech>::new(),
        }
    }
}

pub fn start(app_handle: tauri::AppHandle,
             mut tweet_rx: tokio::sync::mpsc::Receiver<Record>,
             display_tx: tokio::sync::mpsc::Sender<display_bridge::DisplayContrl>,
             playbook_tx: tokio::sync::mpsc::Sender<voicegen_agent::Playbook>,
             mut speech_rx: tokio::sync::mpsc::Receiver<voicegen_agent::Speech>,
             audioctl_tx: tokio::sync::mpsc::Sender<audio_player::AudioControl>,
             audioctl_rdy_rx: tokio::sync::mpsc::Receiver<audio_player::AudioControlRdy>,
             mut user_rx: tokio::sync::mpsc::Receiver<user_input::UserInput>,
             mut speaker_rx: tokio::sync::mpsc::Receiver<voicegen_observer::Speaker>,
             )
{

    let (speech_rdy_tx, speech_rdy_rx) = tokio::sync::mpsc::channel::<()>(QUEUE_LENGTH);

    // Context
    let mut ctx = Context::new();

    // Operating clock
    let (clk_tx, mut clk_rx) = tokio::sync::mpsc::channel::<()>(1);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let _ = clk_tx.send(()).await;
        }
    });

    let mut audio_speech_rdy_rx = wait_both(speech_rdy_rx, audioctl_rdy_rx);

    tokio::spawn(async move {
        loop {
            println!("{:?}, {:?}, {:?}, {:?}",
                ctx.wait_list.len(),
                ctx.ready_list.len(),
                ctx.played_list.len(),
                ctx.speech_cache.len());

            print!("scheduler: Select> ");
            tokio::select!{
                Some(msg) = tweet_rx.recv() => {
                    println!("New tweet incoming {:?}", msg.tweet_id);

                    ctx.wait_list.push_back(msg.clone());

                    display_tx.send(display_bridge::DisplayContrl::Add(msg.clone().into())).await.unwrap();

                    if !ctx.focus_set {
                        display_tx.send(display_bridge::DisplayContrl::Scroll(msg.tweet_id)).await.unwrap();
                        ctx.focus_set = true;
                    }
                }

                Some(_) = clk_rx.recv() => {
                    if ctx.wait_list.len() == 0 { continue; } 
                    if ctx.processing.is_some() { continue; }
                    if ctx.cancelling { continue ; }

                    ctx.processing = Some(ctx.wait_list.front().unwrap().clone());
                    println!("<clk>start processing {:?}", ctx.processing.as_ref().unwrap().tweet_id);
                    playbook_tx.send(voicegen_agent::into(ctx.processing.clone().unwrap().clone().into(), ctx.addr, ctx.speaker)).await.unwrap();
                }

                Some(speech) = speech_rx.recv() => {
                    println!("Text to speech is complete {:?}", speech.tweet_id);
                    if ctx.cancelling {
                        // Ignore processing result.
                        println!("tts result is ignored");
                        ctx.cancelling = false;
                        continue;
                    };

                    ctx.ready_list.push_back(ctx.wait_list.pop_front().unwrap());
                    ctx.processing = None;

                    ctx.speech_cache.push_back(speech);
                    speech_rdy_tx.send(()).await.unwrap();
                }

                Some(_) = audio_speech_rdy_rx.recv() => {
                    println!("Audio and speech is ready, start playing.");

                    let target_tw = ctx.ready_list.pop_front().unwrap();
                    let target_twid = target_tw.tweet_id.clone();

                    let index = ctx.speech_cache.iter().position(|x| x.tweet_id == target_twid).unwrap();
                    let s = remove(&mut ctx.speech_cache, index);
                    let voice_pack = vec![s.name, s.text];
                    audioctl_tx.send(audio_player::AudioControl::PlayMulti(voice_pack)).await.unwrap();

                    ctx.played_list.push_back(target_tw);
                    if ctx.played_list.len() > HISTORY_LENGTH {
                        let ve = ctx.played_list.pop_front().unwrap();
                        display_tx.send(display_bridge::DisplayContrl::Delete(ve.tweet_id)).await.unwrap();
                    }

                    display_tx.send(display_bridge::DisplayContrl::Scroll(target_twid)).await.unwrap();
                }

                Some(user) = user_rx.recv() => {
                    print!("User input - ");
                    match user {
                        user_input::UserInput::Jump(twid) => {
                            print!("jump to {:?}", twid);
                            audioctl_tx.send(audio_player::AudioControl::Stop).await.unwrap();

                            // Cancel current playing speech only;
                            if twid == "" {
                                continue;
                            }

                            if ctx.processing.is_some() {
                                ctx.cancelling = true;
                            }

                            let p = ctx.wait_list.iter().position(|x| x.tweet_id == twid);
                            if p.is_some() {
                                ctx.played_list.append(&mut ctx.ready_list);

                                let tail = ctx.wait_list.split_off(p.unwrap());

                                ctx.played_list.append(&mut ctx.wait_list);
                                ctx.wait_list = tail;

                                ctx.processing = None;
                                ctx.speech_cache.clear();
                            }

                            let p = ctx.ready_list.iter().position(|x| x.tweet_id == twid);
                            if p.is_some() {
                                let tail = ctx.ready_list.split_off(p.unwrap());

                                ctx.played_list.append(&mut ctx.ready_list);

                                ctx.ready_list = tail;

                                ctx.processing = None;
                                let tail = ctx.speech_cache.split_off(p.unwrap());
                                ctx.speech_cache = tail;
                            }

                            while ctx.played_list.len() >= HISTORY_LENGTH {
                                let ve = ctx.played_list.pop_front().unwrap();
                                display_tx.send(display_bridge::DisplayContrl::Delete(ve.tweet_id)).await.unwrap();
                            }
                            display_tx.send(display_bridge::DisplayContrl::Scroll(twid)).await.unwrap();
                        },
                    }
                }

                Some(speaker) = speaker_rx.recv() => {
                    println!("{:?}", speaker);
                    ctx.speaker = speaker.speaker;

                    if ctx.processing.is_some() {
                        ctx.wait_list.push_front(ctx.processing.unwrap());
                        ctx.processing = None;
                        ctx.cancelling = true;
                    }
                    if ctx.ready_list.len() > 2 {
                        ctx.ready_list.append(&mut ctx.wait_list);
                        ctx.wait_list = ctx.ready_list.split_off(1);
                        ctx.speech_cache.split_off(1);
                    }

                }

                else => {
                    println!("Core thread exit");
                    return ();
                }
            }
        }
    });

}
