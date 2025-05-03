mod api;
mod bindings;
mod handlers;
mod types;

use crate::bindings::exports::ntwk::theater::actor::Guest;
use crate::bindings::exports::ntwk::theater::message_server_client::Guest as MessageServerClient;
use crate::bindings::ntwk::theater::runtime::log;
use crate::types::state::{Config, State};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct InitData {
    anthropic_api_key: String,
    store_id: Option<String>,
    config: Option<Config>,
}

struct Component;

impl Guest for Component {
    fn init(data: Option<Vec<u8>>, params: (String,)) -> Result<(Option<Vec<u8>>,), String> {
        log("Initializing anthropic-proxy actor");
        let (id,) = params;
        log(&format!("Actor ID: {}", id));

        // Parse initialization data
        let init_data: InitData = match data {
            Some(bytes) => match serde_json::from_slice(&bytes) {
                Ok(data) => data,
                Err(e) => {
                    return Err(format!("Failed to parse init data: {}", e));
                }
            },
            None => {
                return Err("No initialization data provided".to_string());
            }
        };

        log("Init data parsed successfully");

        // Initialize state
        let state = State::new(
            id,
            init_data.anthropic_api_key,
            init_data.store_id,
            init_data.config,
        );

        log("State initialized");

        // Serialize and return the state
        match serde_json::to_vec(&state) {
            Ok(state_bytes) => {
                log("Actor initialized successfully");
                Ok((Some(state_bytes),))
            }
            Err(e) => Err(format!("Failed to serialize state: {}", e)),
        }
    }
}

impl MessageServerClient for Component {
    fn handle_send(
        state: Option<Vec<u8>>,
        _params: (Vec<u8>,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        log("Handling send message in anthropic-proxy");

        // Nothing to return for a send
        Ok((state,))
    }

    fn handle_request(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        log("Handling request message in anthropic-proxy");
        let (request_id, data) = params;
        log(&format!("Request ID: {}", request_id));

        // Use our message handler
        handlers::message::handle_request(data, state.unwrap())
    }

    fn handle_channel_open(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        _params: (bindings::exports::ntwk::theater::message_server_client::Json,),
    ) -> Result<
        (
            Option<bindings::exports::ntwk::theater::message_server_client::Json>,
            (bindings::exports::ntwk::theater::message_server_client::ChannelAccept,),
        ),
        String,
    > {
        log("Channel open request received");

        Ok((
            state,
            (
                bindings::exports::ntwk::theater::message_server_client::ChannelAccept {
                    accepted: true,
                    message: None,
                },
            ),
        ))
    }

    fn handle_channel_close(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (String,),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        let (channel_id,) = params;
        log(&format!("Channel {} closed", channel_id));

        Ok((state,))
    }

    fn handle_channel_message(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (
            String,
            bindings::exports::ntwk::theater::message_server_client::Json,
        ),
    ) -> Result<(Option<bindings::exports::ntwk::theater::message_server_client::Json>,), String>
    {
        let (channel_id, _message) = params;
        log(&format!("Received message on channel {}", channel_id));

        Ok((state,))
    }
}

bindings::export!(Component with_types_in bindings);
