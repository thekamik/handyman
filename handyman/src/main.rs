use crate::connection::tcp_socket::create_socket;

pub mod connection;
pub mod http_server;


async fn main() {
    println!("Hello, world!");

    create_socket();
}
