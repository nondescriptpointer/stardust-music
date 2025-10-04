use phonic::{DefaultOutputDevice, FilePlaybackOptions, Player};
use tokio::sync::mpsc::UnboundedReceiver;

#[derive(Debug)]
pub enum AudioCommand {
    Play(String),
    PlayPause,
    Stop,
    UpdateRate(f64),
}

pub async fn start_playback_listener(mut message_rx: UnboundedReceiver<AudioCommand>) {
    tracing::info!("Audio playback listener started");
    // setup the player
    let output_device = DefaultOutputDevice::open().expect("Could not open default audio device");
    let mut player = Player::new(output_device, None);
    let mut current_playback_id: Option<usize> = None;
    let mut playing = false;

    while let Some(message) = message_rx.recv().await {
        tracing::debug!("Received audio command: {:?}", message);
        match message {
            AudioCommand::Play(path) => {
                // stop existing
                if let Some(playback_id) = current_playback_id {
                    let _ = player.stop_source(playback_id, None);
                }
                // play new one
                let result = player.play_file(path, FilePlaybackOptions::default().streamed());
                current_playback_id = result.ok();
                playing = true;
            }
            AudioCommand::PlayPause => {
                // play/pause current by setting rate to 0
                if let Some(playback_id) = current_playback_id {
                    let _ = player.set_source_speed(
                        playback_id,
                        if playing { 0.0 } else { 1.0 },
                        None,
                        None,
                    );
                    playing = !playing;
                }
            }
            AudioCommand::Stop => {
                if let Some(playback_id) = current_playback_id {
                    let _ = player.stop_source(playback_id, None);
                }
                playing = false;
            }
            AudioCommand::UpdateRate(rate) => {
                // update playback rate, TODO: might want to glide this
                if let Some(playback_id) = current_playback_id {
                    let _ = player.set_source_speed(
                        playback_id,
                        rate,
                        None,
                        None,
                    );
                    playing = !playing;
                }
            }
        }
    }
    tracing::info!("Audio playback listener exiting - no more messages");
}