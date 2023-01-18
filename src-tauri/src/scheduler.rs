use std::collections::LinkedList;
use serde::{Serialize, Deserialize};
use tauri::Manager;

use crate::twitter_data;
use crate::display_bridge;
use crate::voicegen_agent;
use crate::audio_player;

const QUEUE_LENGTH : usize = 24;
const HISTORY_LENGTH: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub tweet_id: String,
    pub created_at: String,
    pub text: String,
    pub name: String,
    pub username: String,
}

pub fn into(tweet: &twitter_data::Tweet, users: &Vec<twitter_data::User>) -> Record {
    let user = users.iter().find(|user| user.id == tweet.author_id).unwrap();

    Record {
        tweet_id: tweet.id.clone(),
        created_at: tweet.created_at.clone(),
        text: tweet.text.clone(),
        name: user.name.clone(),
        username: user.username.clone(),
    }
}

fn wait_both(mut speech_rdy_rx: tokio::sync::mpsc::Receiver<()>,
                   mut audio_rdy_rx: tokio::sync::mpsc::Receiver<audio_player::AudioControlRdy>)
            -> tokio::sync::mpsc::Receiver<()> {
    let (tx, rx)
        = tokio::sync::mpsc::channel::<()>(1);

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

fn scroll_view(app_handle: &tauri::AppHandle, tweet_id: String) {
    app_handle
        .emit_all("tauri://frontend/scroll", tweet_id)
        .unwrap();
}

pub fn start(app_handle: tauri::AppHandle,
             mut tweet_rx: tokio::sync::mpsc::Receiver<Record>,
             display_tx: tokio::sync::mpsc::Sender<display_bridge::ViewElements>,
             playbook_tx: tokio::sync::mpsc::Sender<voicegen_agent::Playbook>,
             mut speech_rx: tokio::sync::mpsc::Receiver<voicegen_agent::Speech>,
             audioctl_tx: tokio::sync::mpsc::Sender<audio_player::AudioControl>,
             audioctl_rdy_rx: tokio::sync::mpsc::Receiver<audio_player::AudioControlRdy>
             )
{
    let (speech_rdy_tx, speech_rdy_rx) 
        = tokio::sync::mpsc::channel::<()>(QUEUE_LENGTH);

    let mut wait_list = LinkedList::<Record>::new();
    let mut ready_list = LinkedList::<Record>::new();
    let mut played_list = LinkedList::<Record>::new();
    let mut speech_cache = LinkedList::<voicegen_agent::Speech>::new();
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
                    println!("New tweet incoming");

                    wait_list.push_back(msg.clone());

                    display_tx.send(msg.clone().into()).await.unwrap();
                    playbook_tx.send(msg.clone().into()).await.unwrap();
                }

                Some(speech) = speech_rx.recv() => {
                    println!("Text to speech is complete");

                    let index = wait_list.iter().position(|x| x.tweet_id == speech.tweet_id).unwrap();
                    ready_list.push_back(remove(&mut wait_list, index));

                    speech_cache.push_back(speech);
                    speech_rdy_tx.send(()).await.unwrap();
                }

                Some(_) = audio_speech_rdy_rx.recv() => {
                    println!("Audio and speech is ready, start playing.");

                    let s = speech_cache.pop_front().unwrap();
                    let voice_pack = vec![s.name, s.text];
                    audioctl_tx.send(audio_player::AudioControl::PlayMulti(voice_pack)).await.unwrap();

                    let index = ready_list.iter().position(|x| x.tweet_id == s.tweet_id).unwrap();
                    let target_tw = remove(&mut ready_list, index);
                    let target_twid = target_tw.tweet_id.clone();

                    scroll_view(&app_handle, target_twid);
                    played_list.push_back(target_tw);
                    if played_list.len() > HISTORY_LENGTH {
                        let _ = played_list.pop_front();
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
