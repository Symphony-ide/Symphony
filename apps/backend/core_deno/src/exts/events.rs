use deno_core::error::AnyError;
use deno_core::{op, Extension, OpState};
use sveditor_core_api::extensions::client::ExtensionClient;
use sveditor_core_api::messaging::ClientMessages;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::events_manager::EventsManager;
use crate::WorkerHandle;

/// Listen messages from Core in Deno
#[op]
async fn op_listen_messages_core_rx(
    state: Rc<RefCell<OpState>>,
    event_name: String,
    _: (),
) -> Result<ClientMessages, AnyError> {
    let (listener, mut receiver) = mpsc::channel(1);
    let listener_id = Uuid::new_v4();

    let events_manager: EventsManager = {
        let state = state.try_borrow()?;

        state.try_borrow::<EventsManager>().unwrap().clone()
    };

    events_manager
        .listen_on(event_name.clone(), listener_id, listener)
        .await;

    let event_response = receiver.recv().await;

    events_manager.unlisten_from(event_name, listener_id).await;

    Ok(event_response.unwrap())
}

/// Send messages from Deno to Core
#[op]
async fn op_send_message_core_tx(
    state: Rc<RefCell<OpState>>,
    msg: ClientMessages,
    _: (),
) -> Result<(), AnyError> {
    let client: ExtensionClient = {
        let state = state.borrow();
        state.try_borrow::<ExtensionClient>().unwrap().to_owned()
    };

    client.send(msg).await?;

    Ok(())
}

/// Terminate the Deno worker
#[op]
async fn op_terminate_main_worker(
    state: Rc<RefCell<OpState>>,
    _: (),
    _: (),
) -> Result<(), AnyError> {
    let (worker_handle, client) = {
        let state = state.borrow();
        let worker_handle = state.borrow::<WorkerHandle>().to_owned();
        let client = state.borrow::<ExtensionClient>().to_owned();
        (worker_handle, client)
    };

    if let Some(handle) = &*worker_handle.lock().await {
        handle.terminate_execution();

        tracing::info!("Unloaded Deno Extension <{}>", client.name);
    }

    Ok(())
}

/// Duplex Events channels for Symphony and Deno
pub fn new(
    client: ExtensionClient,
    events_manager: EventsManager,
    worker_handle: WorkerHandle,
) -> Extension {
    Extension::builder()
        .ops(vec![
            op_send_message_core_tx::decl(),
            op_listen_messages_core_rx::decl(),
            op_terminate_main_worker::decl(),
        ])
        .state(move |s| {
            s.put(client.clone());
            s.put(events_manager.clone());
            s.put(worker_handle.clone());
            Ok(())
        })
        .build()
}
