use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Determine working directory from args or current directory.
    let working_dir = std::env::args()
        .skip_while(|a| a != "--working-dir")
        .nth(1)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    let server = paperjam_mcp::PaperjamServer::new(working_dir);

    // Start stdio transport.
    let transport = (tokio::io::stdin(), tokio::io::stdout());
    let service = server.serve(transport).await?;
    service.waiting().await?;

    Ok(())
}
