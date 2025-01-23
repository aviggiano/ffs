use ffs::providers::hetzner;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let binding = "ls".to_string();
    let action = args.get(1).unwrap_or(&binding);
    match action.as_str() {
        "ls" => {
            if let Ok(jobs) = hetzner::list_jobs().await {
                for job in jobs {
                    println!("{job:?}");
                }
            }
        }
        "start" => {
            let name = args.get(2).unwrap().to_string();
            println!("Starting job {name}");
            hetzner::start_job(name).await?;
        }
        "stop" => {
            let id = args.get(2).unwrap().parse::<i64>().unwrap();
            println!("Stopping job {id}");
            hetzner::stop_job(id).await?;
        }
        "tail" => {
            let id = args.get(2).unwrap().parse::<i64>().unwrap();
            let filename = args.get(3).unwrap();
            println!("Fetching logs for job {id} at {filename}");
            hetzner::tail(id, filename.to_string()).await?;
        }
        _ => {
            println!("Invalid action");
        }
    }

    Ok(())
}
