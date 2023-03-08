use serde::{Deserialize, Serialize};
use std::collections::LinkedList;

use crate::audio_player;
use crate::display_bridge;
use crate::twitter_data;
use crate::twitter_agent;
use crate::user_input;
use crate::voicegen_agent;

const HISTORY_LENGTH: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub tweet_id: String,
    pub author_id: String,
    pub created_at: String,
    pub text: String,
    pub name: String,
    pub username: String,
    pub profile_image_url: String,
    pub attachments: Vec<(String, String)>,
}

pub fn into(tweet: &serde_json::Value, users: &Vec<serde_json::Value>, medias: &Vec<serde_json::Value>) -> Record {
    let user = users
        .iter()
        .find(|user| user["id"] == tweet["author_id"])
        .unwrap();

    let mut attachments = Vec::new();
    let media_keys = tweet["attachments"]["media_keys"].as_array();
    if media_keys != None {
        for media_key in media_keys.unwrap() {
            let media = medias
                .iter()
                .find(|media| media["media_key"].as_str().unwrap() == media_key.as_str().unwrap())
                .unwrap();

            let mtype = media["type"].as_str().unwrap().to_string();
            if mtype == "photo" {
                attachments.push((mtype, media["url"].as_str().unwrap().to_string()));
            } else if mtype == "gif" {
                attachments.push((mtype, media["preview_image_url"].as_str().unwrap().to_string()));
            } else if mtype == "video"{
                attachments.push((mtype, media["preview_image_url"].as_str().unwrap().to_string()));
            }
        }
    }

    Record {
        tweet_id: tweet["id"].as_str().unwrap().to_string(),
        author_id: tweet["author_id"].as_str().unwrap().to_string(),
        created_at: tweet["created_at"].as_str().unwrap().to_string(),
        text: tweet["text"].as_str().unwrap().to_string(),
        name: user["name"].as_str().unwrap().to_string(),
        username: user["username"].as_str().unwrap().to_string(),
        profile_image_url: user["profile_image_url"].as_str().unwrap().to_string(),
        attachments,
    }
}

fn remove<T>(list: &mut LinkedList<T>, index: usize) -> T {
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

struct Settings {
    pub addr: std::net::SocketAddr,
    pub speaker: u64,
    pub speech_rate: f64,
    pub paused: bool,
}

impl Settings {
    pub fn new() -> Self {
        Settings {
            addr: std::net::SocketAddr::from(([127, 0, 0, 1], 50031)),
            speaker: 0,
            speech_rate: 1.0f64,
            paused: false,
        }
    }
}

struct Context {
    pub forcus_position_initialized: bool,
    pub cancelling: bool,
    pub tts_processing: bool,
    pub wait_list: LinkedList<Record>,
    pub ready_list: LinkedList<Record>,
    pub played_list: LinkedList<Record>,
    pub speech_cache: LinkedList<voicegen_agent::Speech>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            forcus_position_initialized: false,
            cancelling: false,
            tts_processing: false,
            wait_list: LinkedList::<Record>::new(),
            ready_list: LinkedList::<Record>::new(),
            played_list: LinkedList::<Record>::new(),
            speech_cache: LinkedList::<voicegen_agent::Speech>::new(),
        }
    }

    pub fn add_new_tweet(&mut self, msg: &Record) {
        self.wait_list.push_back(msg.clone());
    }

    pub fn is_tts_ready(&self) -> bool {
        self.wait_list.len() > 0
            && !self.tts_processing
            && !self.cancelling
    }

    pub fn fetch_for_tts(&mut self) -> Record {
        self.tts_processing = true;
        self.wait_list.front().unwrap().clone()
    }

    pub fn cancelling(&self) -> bool {
        self.cancelling
    }

    pub fn cancel(&mut self) {
        self.cancelling = false;
    }

    pub fn tts_complete(&mut self, tts_result: Option<voicegen_agent::Speech>) {
        match tts_result {
            Some(tts_result) => {
                println!("Text to speech is complete {:?}", tts_result.tweet_id);

                self.tts_processing = false;
                self.ready_list.push_back(self.wait_list.pop_front().unwrap());
                self.speech_cache.push_back(tts_result);
            }
            None => {
                println!("Text to speech is failed ");
                self.tts_processing = false;
            }
        }
    }

    pub fn is_speech_ready(&self) -> bool {
        self.ready_list.len() > 0
    }

    pub fn fetch_for_playback(&mut self) -> (voicegen_agent::Speech, Option<String>){
        let mut overflow_id: Option<String> = None;
        let target_tw = self.ready_list.pop_front().unwrap();
        let target_twid = target_tw.tweet_id.clone();

        let index = self.speech_cache.iter().position(|x| x.tweet_id == target_twid).unwrap();
        let speech = remove(&mut self.speech_cache, index);

        self.played_list.push_back(target_tw);
        if self.played_list.len() > HISTORY_LENGTH {
            let ve = self.played_list.pop_front().unwrap();
            overflow_id = Some(ve.tweet_id);
        }

        return (speech, overflow_id)
    }

    pub fn jump_to_twid(&mut self, twid: &String) -> Vec<String> {
        if self.tts_processing {
            self.cancelling = true;
        }

        let p = self.wait_list.iter().position(|x| x.tweet_id == *twid);
        if p.is_some() {
            self.played_list.append(&mut self.ready_list);

            let tail = self.wait_list.split_off(p.unwrap());

            self.played_list.append(&mut self.wait_list);
            self.wait_list = tail;

            self.tts_processing = false;
            self.speech_cache.clear();
        }

        let p = self.ready_list.iter().position(|x| x.tweet_id == *twid);
        if p.is_some() {
            let tail = self.ready_list.split_off(p.unwrap());

            self.played_list.append(&mut self.ready_list);

            self.ready_list = tail;

            self.tts_processing = false;
            let tail = self.speech_cache.split_off(p.unwrap());
            self.speech_cache = tail;
        }

        let mut drop_list : Vec<String> = vec![];
        while self.played_list.len() >= HISTORY_LENGTH {
            let ve = self.played_list.pop_front().unwrap();
            drop_list.push(ve.tweet_id);
        }

        drop_list
    }

    pub fn remove_cache(&mut self) {
        if self.tts_processing {
            self.tts_processing = false;
            self.cancelling = true;
        }
        if self.ready_list.len() > 0 {
            self.ready_list.append(&mut self.wait_list);
            self.wait_list = self.ready_list.split_off(0);
            self.speech_cache.split_off(0);
        }
    }

}


pub fn start(
    _app_handle: tauri::AppHandle,
    display_tx: tokio::sync::mpsc::Sender<display_bridge::DisplayContrl>,
    playbook_tx: tokio::sync::mpsc::Sender<voicegen_agent::Playbook>,
    audioctl_tx: tokio::sync::mpsc::Sender<audio_player::AudioControl>,
    mut tweet_rx: tokio::sync::mpsc::Receiver<(twitter_agent::Timeline, Record)>,
    mut speech_rx: tokio::sync::mpsc::Receiver<Option<voicegen_agent::Speech>>,
    mut audioctl_rdy_rx: tokio::sync::mpsc::Receiver<audio_player::AudioControlRdy>,
    mut user_rx: tokio::sync::mpsc::Receiver<user_input::UserInput>,
) {
    // Context
    let mut ctx = Context::new();
    let mut settings = Settings::new();

    // Operating clock
    let (clk_tx, mut clk_rx) = tokio::sync::mpsc::channel::<()>(1);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            let _ = clk_tx.send(()).await;
        }
    });

    tokio::spawn(async move {
        loop {
            println!("");
            println!(
                "setting: {:?}, {:?}, {:?}, {:?}",
                settings.addr,
                settings.speaker,
                settings.speech_rate,
                settings.paused,
            );
            println!(
                "ctx: {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                ctx.wait_list.len(),
                ctx.ready_list.len(),
                ctx.played_list.len(),
                ctx.speech_cache.len(),
                ctx.cancelling,
                ctx.tts_processing
            );

            print!("scheduler: Select> ");
            tokio::select! {
                Some(_) = clk_rx.recv() => {
                    // Obtain Tweet

                    if !settings.paused {
                        match tweet_rx.try_recv() {
                            Ok((tl, msg)) => {
                                println!("New tweet incoming {:?}", tl);
                                println!("New tweet incoming {:?}", msg.tweet_id);

                                ctx.add_new_tweet(&msg);
                                display_tx.send(display_bridge::DisplayContrl::Add(msg.clone().into())).await.unwrap();

                                if !ctx.forcus_position_initialized {
                                    display_tx.send(display_bridge::DisplayContrl::Scroll(msg.tweet_id)).await.unwrap();
                                    ctx.forcus_position_initialized = true;
                                }
                            }

                            Err(e) => {
                                match e {
                                    tokio::sync::mpsc::error::TryRecvError::Empty => {},

                                    e => {
                                        println!("scheduler: twitter agent closes pci {:?}", e);
                                        return ();
                                    }
                                }
                            },
                        }
                    }

                    // TTS Start
                    if ctx.is_tts_ready() {
                        let r = ctx.fetch_for_tts();
                        println!("<clk>start tts_processing {:?}", r.tweet_id);

                        playbook_tx.send(
                            voicegen_agent::into(r, settings.addr, settings.speaker, settings.speech_rate)
                            ).await.unwrap();

                    }

                    // Process TTS Result
                    match speech_rx.try_recv() {
                        Ok(speech) => {
                            if ctx.cancelling() {
                                // Ignore processing result. if the cancelling state
                                println!("tts result is ignored");
                                ctx.cancel();
                                continue;
                            }
                            ctx.tts_complete(speech);
                        },

                        Err(e) => {
                            match e {
                                tokio::sync::mpsc::error::TryRecvError::Empty => {},

                                e => {
                                    println!("scheduler: voicegen_agent closes pci {:?}", e);
                                    return ();
                                }
                            }
                        },
                    }

                    // Play speech
                    if ctx.is_speech_ready() {
                        match audioctl_rdy_rx.try_recv() {
                            Ok(_) => {
                                println!("Audio and speech is ready, start playing.");
                                let (speech, overflow) = ctx.fetch_for_playback();

                                let voice_pack = vec![speech.name, speech.text];
                                audioctl_tx.send(audio_player::AudioControl::PlayMulti(voice_pack)).await.unwrap();

                                if let Some(twid) = overflow {
                                    display_tx.send(display_bridge::DisplayContrl::Delete(twid)).await.unwrap();
                                }
                                display_tx.send(display_bridge::DisplayContrl::Scroll(speech.tweet_id)).await.unwrap();
                            },

                            Err(e) => {
                                match e {
                                    tokio::sync::mpsc::error::TryRecvError::Empty => {},

                                    e => {
                                        println!("scheduler: audio player closes pci {:?}", e);
                                        return ();
                                    }
                                }
                            },
                        }
                    }

                }

                Some(user) = user_rx.recv() => {
                    print!("User input - ");
                    match user {
                        user_input::UserInput::Jump(twid) => {
                            print!("jump to {:?}", twid);
                            audioctl_tx.send(audio_player::AudioControl::Stop).await.unwrap();

                            // Cancel current playing speech only;
                            if twid == "" { continue; }

                            let drop_list = ctx.jump_to_twid(&twid);

                            for id in drop_list {
                                display_tx.send(display_bridge::DisplayContrl::Delete(id)).await.unwrap();
                            }
                            display_tx.send(display_bridge::DisplayContrl::Scroll(twid)).await.unwrap();
                        },

                        user_input::UserInput::Paused(msg) => {
                            settings.paused = msg;
                        }

                        user_input::UserInput::Speaker(speaker) => {
                            println!("{:?}", speaker);
                            settings.addr = speaker.addr;
                            settings.speaker = speaker.speaker;

                            ctx.remove_cache();
                        }

                        user_input::UserInput::SpeechRate(speech_rate) => {
                            settings.speech_rate = speech_rate;

                            ctx.remove_cache();
                        }
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
