use crate::connection::tcp_socket::create_socket;

pub mod connection;
pub mod http_server;
pub mod model;


#[tokio::main]
async fn main() {
    println!("Hello, world!");

    create_socket();
}
