mod api;
mod bindings;
mod handlers;
mod types;
mod utils;

use crate::api::AnthropicClient;
use crate::bindings::exports::ntwk::theater::actor::Guest;
use crate::bindings::exports::ntwk::theater::http_handlers::Guest as HttpHandlersGuest;
use crate::bindings::exports::ntwk::theater::message_server_client::Guest as MessageServerClient;
use crate::bindings::ntwk::theater::http_types::{HttpRequest as FrameworkHttpRequest, HttpResponse as FrameworkHttpResponse, MiddlewareResult};
use crate::bindings::ntwk::theater::runtime::log;
use crate::types::state::{Config, State};
use crate::utils::setup_http_server;

use serde::{Deserialize, Serialize};
use serde_json::json;

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
        
        // Set up HTTP server
        let server_id = setup_http_server()?;
        
        // Initialize state
        let state = State::new(
            id,
            init_data.anthropic_api_key,
            init_data.store_id,
            init_data.config,
            Some(server_id),
        );
        
        log("State initialized");
        
        // Serialize and return the state
        match serde_json::to_vec(&state) {
            Ok(state_bytes) => {
                log("Actor initialized successfully");
                Ok((Some(state_bytes),))
            },
            Err(e) => {
                Err(format!("Failed to serialize state: {}", e))
            }
        }
    }
}

impl HttpHandlersGuest for Component {
    fn handle_request(
        state: Option<Vec<u8>>,
        params: (u64, FrameworkHttpRequest),
    ) -> Result<(Option<Vec<u8>>, (FrameworkHttpResponse,)), String> {
        let (handler_id, request) = params;
        log(&format!("Handling HTTP request with handler ID: {}", handler_id));

        // Map Framework HTTP request to the expected format
        let mapped_request = crate::bindings::ntwk::theater::http_client::HttpRequest {
            method: request.method.clone(),
            uri: request.uri.clone(),
            headers: request.headers.clone(),
            body: request.body.clone(),
        };

        // Use our HTTP handler
        let (new_state, (response,)) = handlers::http::handle_request(mapped_request, state.unwrap())?;

        // Map the response back
        let framework_response = FrameworkHttpResponse {
            status: response.status,
            headers: response.headers,
            body: response.body,
        };

        Ok((new_state, (framework_response,)))
    }

    fn handle_middleware(
        state: Option<Vec<u8>>,
        params: (u64, FrameworkHttpRequest),
    ) -> Result<(Option<Vec<u8>>, (MiddlewareResult,)), String> {
        let (handler_id, request) = params;
        log(&format!("Middleware called with handler ID: {}", handler_id));

        // For now, just pass all requests through
        Ok((
            state,
            (MiddlewareResult {
                proceed: true,
                request,
            },),
        ))
    }

    fn handle_websocket_connect(
        state: Option<Vec<u8>>,
        params: (u64, u64, String, Option<String>),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (handler_id, connection_id, path, _query) = params;
        log(&format!("WebSocket connect: Handler {}, Connection {}, Path {}", 
                    handler_id, connection_id, path));
        
        // Just return the state unchanged for now
        Ok((state,))
    }

    fn handle_websocket_message(
        state: Option<Vec<u8>>,
        params: (u64, u64, crate::bindings::ntwk::theater::websocket_types::WebsocketMessage),
    ) -> Result<(Option<Vec<u8>>, (Vec<crate::bindings::ntwk::theater::websocket_types::WebsocketMessage>,)), String> {
        let (handler_id, connection_id, message) = params;
        log(&format!("WebSocket message: Handler {}, Connection {}", 
                    handler_id, connection_id));
        
        // Just return empty messages for now
        Ok((state, (vec![],)))
    }

    fn handle_websocket_disconnect(
        state: Option<Vec<u8>>,
        params: (u64, u64),
    ) -> Result<(Option<Vec<u8>>,), String> {
        let (handler_id, connection_id) = params;
        log(&format!("WebSocket disconnect: Handler {}, Connection {}", 
                    handler_id, connection_id));
        
        // Just return the state unchanged for now
        Ok((state,))
    }
}

impl MessageServerClient for Component {
    fn handle_send(
        state: Option<Vec<u8>>,
        params: (Vec<u8>,),
    ) -> Result<(Option<Vec<u8>>,), String> {
        log("Handling send message");
        let (data,) = params;
        
        // Nothing to return for a send
        Ok((state,))
    }

    fn handle_request(
        state: Option<Vec<u8>>,
        params: (String, Vec<u8>),
    ) -> Result<(Option<Vec<u8>>, (Option<Vec<u8>>,)), String> {
        log("Handling request message");
        let (request_id, data) = params;
        log(&format!("Request ID: {}", request_id));
        
        // Use our message handler
        handlers::message::handle_message(data, state.unwrap())
    }

    fn handle_channel_open(
        state: Option<bindings::exports::ntwk::theater::message_server_client::Json>,
        params: (bindings::exports::ntwk::theater::message_server_client::Json,),
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
        let (channel_id, message) = params;
        log(&format!("Received message on channel {}", channel_id));
        
        Ok((state,))
    }
}

bindings::export!(Component with_types_in bindings);
