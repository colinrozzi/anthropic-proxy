use crate::bindings::ntwk::theater::http_framework::{
    add_route, create_server, register_handler, start_server, ServerConfig,
};
use crate::bindings::ntwk::theater::runtime::log;

/// Set up an HTTP server for the anthropic-proxy actor
pub fn setup_http_server() -> Result<u64, String> {
    log("Setting up HTTP server for anthropic-proxy");

    // Create server configuration
    let config = ServerConfig {
        port: Some(8085), // Choose a different port than the chat actor
        host: Some("0.0.0.0".to_string()),
        tls_config: None,
    };

    // Create the HTTP server
    let server_id = create_server(&config)?;
    log(&format!("Created server with ID: {}", server_id));

    // Register API handlers
    let api_handler_id = register_handler("handle_request")?;
    log(&format!("Registered API handler: {}", api_handler_id));

    // Add routes
    add_route(server_id, "/", "GET", api_handler_id)?;
    add_route(server_id, "/index.html", "GET", api_handler_id)?;
    
    // API endpoints
    add_route(server_id, "/models", "GET", api_handler_id)?;
    add_route(server_id, "/chat/completions", "POST", api_handler_id)?;

    // Start the server
    let port = start_server(server_id)?;
    log(&format!("Server started on port {}", port));

    Ok(server_id)
}
