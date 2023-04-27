use std::net::SocketAddr;

#[derive(Parser)]
struct Arguments {
    remote_addr: SocketAddr,
    #[arg(long)]
    local_addr: Option<SocketAddr>,
}

#[tokio::main]
async fn main() {
    let Arguments {
        remote_addr,
        local_addr,
    } = Arguments::parse();
}
