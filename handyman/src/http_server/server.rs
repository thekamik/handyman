//! Minimal static file server used as a known upstream for testing the proxy; not a production server.

use std::io;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tracing::{debug, info, warn};

const INDEX_FILE: &str = "main.html";
const MAX_REQUEST_HEAD: usize = 16 * 1024;

/// Starts an HTTP server on the given host and port, serving static files from `root_dir`.
///
/// Runs forever, accepting connections and handling each one in a separate task.
/// Returns an error only if the socket cannot be bound or `root_dir` does not exist.
pub async fn create_server(host: &str, port: u16, root_dir: PathBuf) -> io::Result<()> {
    // Canonical root, so the path traversal check compares real paths
    let root_dir = tokio::fs::canonicalize(&root_dir).await?;
    let listener = TcpListener::bind((host, port)).await?;
    info!(addr = %listener.local_addr()?, root = %root_dir.display(), "test HTTP server listening");

    // Shared, read-only path; each task gets a cheap clone of the pointer, not of the path
    let root_dir: Arc<Path> = Arc::from(root_dir);

    loop {
        let (stream, client_addr) = match listener.accept().await {
            Ok(connection) => connection,
            Err(e) => {
                // A failed accept affects only one client, so keep serving others
                warn!("failed to accept connection: {e}");
                continue;
            }
        };

        let root_dir = Arc::clone(&root_dir);
        tokio::spawn(async move {
            if let Err(e) = handle_connection(stream, client_addr, &root_dir).await {
                warn!(%client_addr, "error while handling connection: {e}");
            }
        });
    }
}

/// Handles a single request and closes the connection (no keep-alive, to keep the server simple).
async fn handle_connection(mut stream: TcpStream, client_addr: SocketAddr, root_dir: &Path) -> io::Result<()> {
    let Some(head) = read_request_head(&mut stream).await? else {
        return Ok(()); // client connected and left without sending anything
    };
    let mut parts = head.lines().next().unwrap_or_default().split(' ');
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or("/");
    debug!(%client_addr, method, target, "request");

    let response = match method {
        "GET" | "HEAD" => serve_file(root_dir, target).await,
        _ => Response::text(405, "Method Not Allowed"),
    };
    response.write_to(&mut stream, method == "HEAD").await?;
    stream.shutdown().await
}

/// Reads up to the end of the request headers; the request body, if any, is ignored.
async fn read_request_head(stream: &mut TcpStream) -> io::Result<Option<String>> {
    let mut buf = Vec::new();
    let mut chunk = [0u8; 4096];
    loop {
        let n = stream.read(&mut chunk).await?;
        if n == 0 {
            return if buf.is_empty() { Ok(None) } else { Err(io::ErrorKind::UnexpectedEof.into()) };
        }
        buf.extend_from_slice(&chunk[..n]);
        if let Some(end) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            return Ok(Some(String::from_utf8_lossy(&buf[..end]).into_owned()));
        }
        if buf.len() > MAX_REQUEST_HEAD {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "request head too large"));
        }
    }
}

async fn serve_file(root_dir: &Path, target: &str) -> Response {
    // Query string and fragment are not part of the file path
    let path = target.split(['?', '#']).next().unwrap_or("/");
    let Some(relative) = sanitize(path) else {
        return Response::text(403, "Forbidden");
    };

    let mut full = root_dir.join(relative);
    if tokio::fs::metadata(&full).await.is_ok_and(|m| m.is_dir()) {
        full.push(INDEX_FILE);
    }

    match tokio::fs::canonicalize(&full).await {
        // Second line of defence: a symlink inside the root must not lead outside of it
        Ok(real) if !real.starts_with(root_dir) => Response::text(403, "Forbidden"),
        Ok(real) => match tokio::fs::read(&real).await {
            Ok(body) => Response { status: 200, content_type: content_type(&real), body },
            Err(_) => Response::text(404, "Not Found"),
        },
        Err(_) => Response::text(404, "Not Found"),
    }
}

/// Turns a URL path into a relative file path, rejecting `..` and anything else that could escape the root.
fn sanitize(path: &str) -> Option<PathBuf> {
    let relative = Path::new(path.trim_start_matches('/'));
    relative
        .components()
        .all(|c| matches!(c, Component::Normal(_)))
        .then(|| relative.to_path_buf())
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json",
        Some("txt") => "text/plain; charset=utf-8",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        _ => "application/octet-stream",
    }
}

struct Response {
    status: u16,
    content_type: &'static str,
    body: Vec<u8>,
}

impl Response {
    fn text(status: u16, body: &str) -> Self {
        Self { status, content_type: "text/plain; charset=utf-8", body: body.as_bytes().to_vec() }
    }

    async fn write_to(&self, stream: &mut TcpStream, head_only: bool) -> io::Result<()> {
        let reason = match self.status {
            200 => "OK",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            _ => "",
        };
        // Content-Length counts bytes, not characters; it stays the same for HEAD
        let head = format!(
            "HTTP/1.1 {} {reason}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            self.status,
            self.content_type,
            self.body.len()
        );
        stream.write_all(head.as_bytes()).await?;
        if !head_only {
            stream.write_all(&self.body).await?;
        }
        Ok(())
    }
}
