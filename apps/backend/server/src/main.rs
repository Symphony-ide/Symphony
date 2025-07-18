use std::sync::Arc;
use std::thread;

use sveditor_core::handlers::HTTPHandler;
use sveditor_core::{Configuration, Server};
use sveditor_core_api::extensions::manager::ExtensionsManager;
use sveditor_core_api::messaging::ClientMessages;
use sveditor_core_api::states::{MemoryPersistor, StatesList, TokenFlags};
use sveditor_core_api::tokio;
use sveditor_core_api::tokio::sync::mpsc::channel;
use sveditor_core_api::{Mutex, State};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};
use git_for_symphony;

fn setup_logger() {
    let filter = EnvFilter::default()
        .add_directive("server=info".parse().unwrap())
        .add_directive("symphony=info".parse().unwrap())
        .add_directive("sveditor_core_api=info".parse().unwrap())
        .add_directive("sveditor_core=info".parse().unwrap())
        .add_directive("typescript_lsp_symphony=info".parse().unwrap());

    let subscriber = Registry::default().with(filter).with(fmt::Layer::default());

    tracing::subscriber::set_global_default(subscriber).expect("Unable to set global subscriber");
}

#[tokio::main]
async fn main() {
    setup_logger();

    let (core_tx, core_rx) = channel::<ClientMessages>(1);

    let extensions_manager = ExtensionsManager::new(core_tx.clone(), None)
        .load_extension_from_entry(git_for_symphony::entry, git_for_symphony::get_info(), 1)
        .await
        .to_owned();

    let states = {
        let sample_state = State::new(1, extensions_manager, Box::new(MemoryPersistor::new()));

        let states = StatesList::new()
            .with_tokens(&[TokenFlags::All("test".to_string())])
            .with_state(sample_state);

        Arc::new(Mutex::new(states))
    };

    let http_handler = HTTPHandler::builder().build().wrap();

    let config = Configuration::new(http_handler, core_tx, core_rx);

    let mut server = Server::new(config, states);

    server.run().await;

    println!("Open http://localhost:8080/?state=0&token=test");

    thread::park();
}
