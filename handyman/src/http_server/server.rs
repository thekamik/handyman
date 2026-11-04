use std::io;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

/// Starts an HTTP server on the given host and port, serving static files from `root_dir`.
///
/// Runs forever, accepting connections and handling each one in a separate task.
/// Returns an error only if the socket cannot be bound.
pub async fn create_server(host: &str, port: u16, root_dir: PathBuf) -> io::Result<()> {
    let listener = TcpListener::bind((host, port)).await?;
    println!("Server listening on {}", listener.local_addr()?);

    // Shared, read-only path; each task gets a cheap clone of the pointer, not of the path
    let root_dir: Arc<Path> = Arc::from(root_dir);

    loop {
        let (stream, client_addr) = match listener.accept().await {
            Ok(connection) => connection,
            Err(e) => {
                // A failed accept affects only one client, so keep serving others
                eprintln!("Failed to accept connection: {e}");
                continue;
            }
        };

        let root_dir = Arc::clone(&root_dir);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, client_addr, &root_dir).await {
                eprintln!("Error while handling {client_addr}: {e}");
            }
        });
    }
}

/// Handles a single client session: reads the HTTP request and serves the requested file.
async fn handle_connection(
    stream: TcpStream,
    client_addr: SocketAddr,
    root_dir: &Path,
) -> io::Result<()> {
    println!("New connection from {client_addr}, serving from {}", root_dir.display());

    // TODO: read the request, resolve the file inside `root_dir`, send the response
    let _ = stream;
    Ok(())
}
