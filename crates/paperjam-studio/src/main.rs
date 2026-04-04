use std::net::SocketAddr;
use std::path::PathBuf;

use axum::Router;
use clap::Parser;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[derive(Parser)]
#[command(name = "paperjam-studio", about = "Serve the paperjam studio web UI")]
struct Args {
    /// Port to serve on
    #[arg(long, default_value = "3000")]
    port: u16,

    /// Built web app directory (defaults to web/dist relative to crate)
    #[arg(long)]
    dist_dir: Option<PathBuf>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let dist_dir = args.dist_dir.unwrap_or_else(|| {
        let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web/dist");
        if crate_dir.exists() {
            crate_dir
        } else {
            PathBuf::from("dist")
        }
    });

    if !dist_dir.exists() {
        eprintln!("Error: dist directory not found: {}", dist_dir.display());
        eprintln!("Hint: Run 'npm run build' in crates/paperjam-studio/web/ first");
        std::process::exit(1);
    }

    let app = Router::new()
        .fallback_service(ServeDir::new(&dist_dir).append_index_html_on_directories(true))
        .layer(CorsLayer::permissive());

    let addr = SocketAddr::from(([127, 0, 0, 1], args.port));
    eprintln!("paperjam studio at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
