//! HEDTRONIX Server Entry Point

use hedtronix_api::config::ServerConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load configuration
    let config = ServerConfig::from_env();
    
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║                    HEDTRONIX v0.1.0                      ║");
    println!("║         Healthcare Operating System                      ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  Starting server on {}                       ║", config.bind_address);
    println!("║  Database: {}                                    ║", config.database_path);
    println!("╚══════════════════════════════════════════════════════════╝");
    
    // Start server
    hedtronix_api::start_server(config).await?;
    
    Ok(())
}
