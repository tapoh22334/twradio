use std::collections::LinkedList;
use serde::{Serialize, Deserialize};
use crate::twitter_data;
use crate::display_bridge;
use crate::voicegen_agent;
use crate::audio_player;

const QUEUE_LENGTH : usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub tweet_id: String,
    pub text: String,
    pub name: String,
    pub username: String,
}

pub fn into(tweet: &twitter_data::Tweet, users: &Vec<twitter_data::User>) -> Record {
    let user = users.iter().find(|user| user.id == tweet.author_id).unwrap();

    Record {
        tweet_id: tweet.id.clone(),
        text: tweet.text.clone(),
        name: user.name.clone(),
        username: user.username.clone(),
    }
}

async fn wait_both(speech_rdy_rx: &mut tokio::sync::mpsc::Receiver<()>,
                   audio_rdy_rx: &mut tokio::sync::mpsc::Receiver<audio_player::AudioControlRdy>) {
    let _ = speech_rdy_rx.recv().await.unwrap();
    let _ = audio_rdy_rx.recv().await.unwrap();
}

pub fn remove<T>(list: &mut LinkedList::<T>, pos: usize) -> T {
        if pos == 0 {
            let v = list.pop_front().unwrap();

            return v;
        } else {
            // split_off function should compute in O(n) time.
            let mut split = list.split_off(pos);
            let v = split.pop_front().unwrap();
            list.append(&mut split);

            return v;
        }
}

pub fn start(mut tweet_rx: tokio::sync::mpsc::Receiver<Record>) -> (
        tokio::sync::mpsc::Receiver<display_bridge::ViewElements>,
        tokio::sync::mpsc::Receiver<voicegen_agent::Playbook>,
        tokio::sync::mpsc::Receiver<audio_player::AudioControl>,
        tokio::sync::mpsc::Sender<audio_player::AudioControlRdy>,
        tokio::sync::mpsc::Sender<voicegen_agent::Speech>
        )
{
    let mut wait_list = LinkedList::<Record>::new();
    let mut ready_list = LinkedList::<Record>::new();
    let mut played_list = LinkedList::<Record>::new();
    let mut speech_cache = LinkedList::<voicegen_agent::Speech>::new();

    let (display_tx, display_rx)
        = tokio::sync::mpsc::channel::<display_bridge::ViewElements>(QUEUE_LENGTH);

    let (playbook_tx, playbook_rx) 
        = tokio::sync::mpsc::channel::<voicegen_agent::Playbook>(QUEUE_LENGTH);

    let (speech_tx, mut speech_rx) 
        = tokio::sync::mpsc::channel::<voicegen_agent::Speech>(QUEUE_LENGTH);

    let (speech_rdy_tx, mut speech_rdy_rx) 
        = tokio::sync::mpsc::channel::<()>(QUEUE_LENGTH);

    let (audioctl_tx, audioctl_rx) 
        = tokio::sync::mpsc::channel::<audio_player::AudioControl>(1);

    let (audioctl_rdy_tx, mut audioctl_rdy_rx) 
        = tokio::sync::mpsc::channel::<audio_player::AudioControlRdy>(1);

    tokio::spawn(async move {
        loop {
            println!("Select> ");
            tokio::select!{
                Some(msg) = tweet_rx.recv() => {
                    println!("New tweet incoming");

                    wait_list.push_back(msg.clone());

                    display_tx.send(msg.clone().into()).await.unwrap();
                    playbook_tx.send(msg.clone().into()).await.unwrap();
                }

                Some(speech) = speech_rx.recv() => {
                    println!("Text to speech is complete");

                    let pos = wait_list.iter().position(|x| x.tweet_id == speech.tweet_id).unwrap();
                    ready_list.push_back(remove(&mut wait_list, pos));

                    speech_cache.push_back(speech);
                    speech_rdy_tx.send(()).await.unwrap();
                }

                _ = wait_both(&mut speech_rdy_rx, &mut audioctl_rdy_rx) => {
                    let s = speech_cache.pop_front().unwrap();
                    audioctl_tx.send(audio_player::AudioControl::Play(s.text)).await.unwrap();

                    let pos = ready_list.iter().position(|x| x.tweet_id == s.tweet_id).unwrap();
                    played_list.push_back(remove(&mut ready_list, pos));
                }

                else => {
                    println!("Core thread exit");
                    return ();
                }
            }
        }
    });

    (display_rx, playbook_rx, audioctl_rx, audioctl_rdy_tx, speech_tx)
}
