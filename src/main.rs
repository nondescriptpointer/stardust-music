mod audio;

use serde::{Deserialize, Serialize};
use stardust_xr_asteroids::{
    ClientState, CustomElement, Element, Migrate, Reify, Transformable, client,
    elements::{Button, Model, Spatial, Text},
};
use stardust_xr_fusion::{
    drawable::{XAlign, YAlign},
    project_local_resources,
};
use tokio::sync::mpsc;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::audio::{AudioCommand, start_playback_listener};

// OnceLock to communicate between audio thread and the client
use std::sync::OnceLock;
static AUDIO_TX: OnceLock<mpsc::UnboundedSender<AudioCommand>> = OnceLock::new();
fn clone_audio_tx() -> mpsc::UnboundedSender<AudioCommand> {
    AUDIO_TX
        .get()
        .expect("AUDIO_TX not initialized; call init_audio_tx() before client::run")
        .clone()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let registry = tracing_subscriber::registry();
    let registry = registry.with(
        tracing_tracy::TracyLayer::new(tracing_tracy::DefaultConfig::default())
            .with_filter(LevelFilter::DEBUG),
    );
    let log_layer = tracing_subscriber::fmt::Layer::new()
        .with_thread_names(true)
        .with_ansi(true)
        .with_line_number(true)
        .with_filter(EnvFilter::from_default_env());
    registry.with(log_layer).init();

    // create channel for communicating with audio thread and start the audio playback thread
    let (tx, rx) = mpsc::unbounded_channel();
    AUDIO_TX.set(tx).unwrap();
    tokio::spawn(async {
        start_playback_listener(rx).await;
    });

    client::run::<State>(&[&project_local_resources!("res")]).await
}

// Application state
#[derive(Debug, Serialize, Deserialize)]
pub struct State {
    current_track_name: Option<String>,
}
impl Default for State {
    fn default() -> Self {
        Self {
            current_track_name: None,
        }
    }
}
impl Migrate for State {
    type Old = Self;
}
impl ClientState for State {
    const APP_ID: &'static str = "com.thomascolliers.stardust_music";

    fn initial_state_update(&mut self) {
        self.current_track_name = Some("Aphex Twin - On".to_string());
    }

    fn on_frame(&mut self, _info: &stardust_xr_fusion::root::FrameInfo) {}
}
impl Reify for State {
    fn reify(&self) -> impl Element<State> {
        Spatial::default()
            .zoneable(true)
            .build()
            .child(Model::namespaced("stardust_music", "player").build())
            .child(
                Button::new(|_| {
                    println!("Play");
                    clone_audio_tx().send(AudioCommand::Play("/home/ego/Downloads/Autechre___2008_04_04_USA_California_Los_Angeles.mp3".into())).unwrap();
                })
                    .pos([0.0, 0.0, 0.0])
                    .size([1.0, 1.0])
                    .build(),
            )
            .maybe_child(self.current_track_name.as_ref().map(|track_name| {
                Spatial::default().pos([0.0, 0.2, -0.2]).build().child(
                    Text::new(track_name)
                        .align_x(XAlign::Center)
                        .align_y(YAlign::Center)
                        .character_height(0.1)
                        .build(),
                )
            }))
    }
}

// 0.5