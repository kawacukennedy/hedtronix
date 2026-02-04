
use hedtronix_api::config::ServerConfig;

fn main() {
    // Start the API server in a background thread for the desktop app
    // In a real desktop app, we might use a different transport or process model
    // but this simplest way to bundle the backend with the frontend.
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
             // Use a randomly assigned port or specific desktop configuration
             let config = ServerConfig {
                 bind_address: "127.0.0.1:8080".to_string(), // Keep consistent with frontend proxy
                 ..ServerConfig::from_env()
             };
             if let Err(e) = hedtronix_api::start_server(config).await {
                 eprintln!("Failed to start server: {}", e);
             }
        });
    });

    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
