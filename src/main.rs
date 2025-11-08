mod audio;

use crate::audio::{AudioCommand, start_playback_listener};
use serde::{Deserialize, Serialize};
use stardust_xr_asteroids::{
    ClientState, CustomElement, Element, Migrate, Reify, Transformable, client,
    elements::{Button, Model, Spatial, Text},
};
use stardust_xr_fusion::{
    drawable::{XAlign, YAlign},
    project_local_resources,
};
use std::sync::OnceLock;
use tokio::sync::mpsc;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    client::run::<State>(&[&project_local_resources!("res")]).await
}

// Application state
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct State {
    current_track_name: Option<String>,
    #[serde(skip)]
    command_tx: OnceLock<mpsc::UnboundedSender<AudioCommand>>,
}
impl Migrate for State {
    type Old = Self;
}
impl ClientState for State {
    const APP_ID: &'static str = "com.thomascolliers.stardust_music";

    fn initial_state_update(&mut self) {
        self.current_track_name = Some("Aphex Twin - On".to_string());
    }

    fn on_frame(&mut self, _info: &stardust_xr_fusion::root::FrameInfo) {
        // create channel for communicating with audio thread and start the audio playback thread
        self.command_tx.get_or_init(|| {
            let (tx, rx) = mpsc::unbounded_channel();
            tokio::spawn(async {
                start_playback_listener(rx).await;
            });
            tx
        });
    }
}
impl Reify for State {
    fn reify(&self) -> impl Element<State> {
        Spatial::default()
            .zoneable(true)
            .build()
            .child(Model::namespaced("stardust_music", "player").build())
            .child(
                Button::new(|state: &mut Self| {
                    let Some(command_tx) = state.command_tx.get() else {
                        return;
                    };
                    println!("Play");
                    command_tx.send(AudioCommand::Play("/home/ego/Downloads/Autechre___2008_04_04_USA_California_Los_Angeles.mp3".into())).unwrap();
                })
                .pos([0.0, 0.0, 0.0])
                .size([1.0, 1.0])
                .build(),
            )
            .maybe_child(self.current_track_name.as_ref().map(|track_name| {
                    Text::new(track_name)
                        .align_x(XAlign::Center)
                        .align_y(YAlign::Center)
                        .character_height(0.1).pos([0.0, 0.2, -0.2])
                        .build()
            }))
    }
}
