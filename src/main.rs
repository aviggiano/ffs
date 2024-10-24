use ffs::hetzner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let servers = hetzner::list_servers().await;

    if let Ok(server) = servers {
        println!("{:?}", server);
    }

    Ok(())
}
