use ffs::providers::{aws, hetzner, Provider};
use ffs::utils::timestamp;

const DEFAULT_ACTION: &str = "ls";
const DEFAULT_PROVIDER: &str = "hetzner";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let default_action = DEFAULT_ACTION.to_string();
    let action = args.get(1).unwrap_or(&default_action);
    let default_provider = DEFAULT_PROVIDER.to_string();
    let provider = match args.get(2).unwrap_or(&default_provider).as_str() {
        "hetzner" => hetzner::HetznerProvider::new(),
        // "aws" => aws::AWSProvider::new(),
        _ => return Err("Invalid provider".into()),
    };
    match action.as_str() {
        "init" => {
            println!("Initializing provider...");
        }
        "ls" | "list" => {
            if let Ok(jobs) = provider.list_jobs().await {
                for job in jobs {
                    println!("{job:?}");
                }
            }
        }
        "start" => {
            let default_name = format!("ffs-job-{}", timestamp());
            let name = args.get(2).unwrap_or(&default_name).to_string();
            println!("Starting job {name}");
            let job = provider.start_job(&name).await?;
            println!("Job {job:?} started");
        }
        "stop" => {
            let id = args.get(2).unwrap().to_string();
            println!("Stopping job {id}");
            let job = provider.stop_job(&id).await?;
            println!("Job {job:?} stopped");
        }
        "tail" => {
            let id = args.get(2).unwrap().to_string();
            let filename = args.get(3).unwrap().to_string();
            println!("Fetching logs for job {id} at {filename}");
            provider.tail(&id, &filename).await?;
        }
        _ => {
            println!("Invalid action");
        }
    }

    Ok(())
}
