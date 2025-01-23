use ffs::providers::hetzner;
use ffs::utils::timestamp;

const DEFAULT_ACTION: &str = "ls";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let default = DEFAULT_ACTION.to_string();
    let action = args.get(1).unwrap_or(&default);
    match action.as_str() {
        "ls" | "list" => {
            if let Ok(jobs) = hetzner::list_jobs().await {
                for job in jobs {
                    println!("{job:?}");
                }
            }
        }
        "start" => {
            let default_name = format!("ffs-job-{}", timestamp());
            let name = args.get(2).unwrap_or(&default_name).to_string();
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
