use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Determine working directory from args or current directory.
    let working_dir = args
        .iter()
        .position(|a| a == "--working-dir")
        .and_then(|i| args.get(i + 1))
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let allow_absolute_paths = args.iter().any(|a| a == "--allow-absolute-paths");

    let server = paperjam_mcp::PaperjamServer::with_config(paperjam_mcp::ServerConfig {
        working_dir,
        allow_absolute_paths,
    });

    // Start stdio transport.
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
