use serde::{Serialize, Deserialize};
use rodio::{Sink};
use rodio::source::{SineWave, Source};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AudioControl {
    Tick,
    Play(Vec<u8>),
    PlayMulti(Vec<Vec<u8>>),
    Volume(u32),
    Pause,
    Resume,
    Quit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioControlRdy {}

pub fn start(app_handle: tauri::AppHandle,
             mut audioctl_rx: tokio::sync::mpsc::Receiver<AudioControl>,
             audioctl_rdy_tx: tokio::sync::mpsc::Sender<AudioControlRdy>
) {
    std::thread::spawn(move || {
        let (_os, mut osh)
            = rodio::OutputStream::try_default()
                .expect("failed to open audio device");

        let sink = Sink::try_new(&osh).expect("failed to create new sink");

        audioctl_rdy_tx.blocking_send(AudioControlRdy{}).unwrap();
        let mut playing = false;

        loop {
            match audioctl_rx.blocking_recv() {
                Some(msg) => {
                    match msg {
                        AudioControl::Tick => {
                            if sink.empty() && playing {
                                println!("audio_coordinator: sink empty");
                                let _ = audioctl_rdy_tx.try_send(AudioControlRdy{});
                                playing = false;
                            }
                        },

                        AudioControl::Play(data) => {
                            println!("audio_coordinator: recv Play");

                            let source = rodio::Decoder::new(
                                std::io::Cursor::new(data))
                                    .expect("failed to decord wav");
                            sink.append(source);
                            playing = true;
                        },

                        AudioControl::PlayMulti(audio_vec) => {
                            println!("audio_coordinator: recv PlayVec");
                            for data in audio_vec {
                                let source = rodio::Decoder::new(
                                    std::io::Cursor::new(data))
                                        .expect("failed to decord wav");
                                sink.append(source);
                            }
                            playing = true;
                        },

                        AudioControl::Volume(n) => {
                            println!("audio_coordinator: recv Volume {:?}", n);
                            sink.set_volume(n as f32 / 100f32);
                        },

                        AudioControl::Pause => {
                            println!("audio_coordinator: recv Pause");
                            sink.pause();
                        },

                        AudioControl::Resume => {
                            println!("audio_coordinator: recv Resume");
                            sink.play();
                        },

                        AudioControl::Quit => {
                            println!("audio_coordinator: recv Quit");
                            break;
                        }
                    }
                },
                None => { return (); }
            }
        }

        println!("sound_coordinator: thread exit");

    });

}
