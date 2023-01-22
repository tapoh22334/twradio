use std::collections::LinkedList;
use serde::{Serialize, Deserialize};
use tauri::Manager;

use crate::twitter_data;
use crate::display_bridge;
use crate::voicegen_agent;
use crate::audio_player;
use crate::user_input;

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

pub fn start(app_handle: tauri::AppHandle,
             mut tweet_rx: tokio::sync::mpsc::Receiver<Record>,
             display_tx: tokio::sync::mpsc::Sender<display_bridge::DisplayContrl>,
             playbook_tx: tokio::sync::mpsc::Sender<voicegen_agent::Playbook>,
             mut speech_rx: tokio::sync::mpsc::Receiver<voicegen_agent::Speech>,
             audioctl_tx: tokio::sync::mpsc::Sender<audio_player::AudioControl>,
             audioctl_rdy_rx: tokio::sync::mpsc::Receiver<audio_player::AudioControlRdy>,
             mut user_rx: tokio::sync::mpsc::Receiver<user_input::UserInput>
             )
{
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], 50031));
    let speaker: u64 = 7;

    let (speech_rdy_tx, speech_rdy_rx) = tokio::sync::mpsc::channel::<()>(QUEUE_LENGTH);

    // Context
    let mut focus_set = false;
    let mut cancelling = false;
    let mut wait_list = LinkedList::<Record>::new();
    let mut ready_list = LinkedList::<Record>::new();
    let mut played_list = LinkedList::<Record>::new();
    let mut processing: Option<Record> = None;
    let mut speech_cache = LinkedList::<voicegen_agent::Speech>::new();

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
                wait_list.len(),
                ready_list.len(),
                played_list.len(),
                speech_cache.len());

            print!("scheduler: Select> ");
            tokio::select!{
                Some(msg) = tweet_rx.recv() => {
                    println!("New tweet incoming {:?}", msg.tweet_id);

                    wait_list.push_back(msg.clone());

                    display_tx.send(display_bridge::DisplayContrl::Add(msg.clone().into())).await.unwrap();

                    if !focus_set {
                        display_tx.send(display_bridge::DisplayContrl::Scroll(msg.tweet_id)).await.unwrap();
                        focus_set = true;
                    }
                }

                Some(_) = clk_rx.recv() => {
                    if wait_list.len() == 0 { continue; } 
                    if processing.is_some() { continue; }
                    if cancelling { continue ; }

                    processing = Some(wait_list.front().unwrap().clone());
                    println!("<clk>start processing {:?}", processing.as_ref().unwrap().tweet_id);
                    playbook_tx.send(voicegen_agent::into(processing.clone().unwrap().clone().into(), addr, speaker)).await.unwrap();
                }

                Some(speech) = speech_rx.recv() => {
                    println!("Text to speech is complete {:?}", speech.tweet_id);
                    if cancelling {
                        // Ignore processing result.
                        println!("tts result is ignored");
                        cancelling = false;
                        continue;
                    };

                    ready_list.push_back(wait_list.pop_front().unwrap());
                    processing = None;

                    speech_cache.push_back(speech);
                    speech_rdy_tx.send(()).await.unwrap();
                }

                Some(_) = audio_speech_rdy_rx.recv() => {
                    println!("Audio and speech is ready, start playing.");

                    let target_tw = ready_list.pop_front().unwrap();
                    let target_twid = target_tw.tweet_id.clone();

                    let index = speech_cache.iter().position(|x| x.tweet_id == target_twid).unwrap();
                    let s = remove(&mut speech_cache, index);
                    let voice_pack = vec![s.name, s.text];
                    audioctl_tx.send(audio_player::AudioControl::PlayMulti(voice_pack)).await.unwrap();

                    played_list.push_back(target_tw);
                    if played_list.len() > HISTORY_LENGTH {
                        let ve = played_list.pop_front().unwrap();
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

                            if processing.is_some() {
                                cancelling = true;
                            }

                            let p = wait_list.iter().position(|x| x.tweet_id == twid);
                            if p.is_some() {
                                played_list.append(&mut ready_list);

                                let tail = wait_list.split_off(p.unwrap());

                                played_list.append(&mut wait_list);
                                wait_list = tail;

                                processing = None;
                                speech_cache.clear();
                            }

                            let p = ready_list.iter().position(|x| x.tweet_id == twid);
                            if p.is_some() {
                                let tail = ready_list.split_off(p.unwrap());

                                played_list.append(&mut ready_list);

                                ready_list = tail;

                                processing = None;
                                let tail = speech_cache.split_off(p.unwrap());
                                speech_cache = tail;
                            }

                            while played_list.len() >= HISTORY_LENGTH {
                                let ve = played_list.pop_front().unwrap();
                                display_tx.send(display_bridge::DisplayContrl::Delete(ve.tweet_id)).await.unwrap();
                            }
                            display_tx.send(display_bridge::DisplayContrl::Scroll(twid)).await.unwrap();
                        },
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
