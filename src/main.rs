use ffs::database::Database;
use ffs::providers::ProviderFactory;
use ffs::utils::timestamp;

const DEFAULT_ACTION: &str = "ls";
const DEFAULT_PROVIDER: &str = "hetzner";
const DEFAULT_JOB_NAME_PREFIX: &str = "ffs-job-";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut database = Database::new();
    let provider = ProviderFactory::new(
        &database
            .get("provider")
            .unwrap_or_else(|| DEFAULT_PROVIDER.to_string())
            .as_str(),
    );

    let args: Vec<String> = std::env::args().collect();
    let default_action = DEFAULT_ACTION.to_string();
    let action = args.get(1).unwrap_or(&default_action);
    match action.as_str() {
        "init" => {
            let default_provider = DEFAULT_PROVIDER.to_string();
            let provider_name = args.get(2).unwrap_or(&default_provider);
            database.set("provider", &provider_name)?;
        }
        "ls" | "list" => {
            if let Ok(jobs) = provider.list_jobs().await {
                for job in jobs {
                    println!("{job:?}");
                }
            }
        }
        "start" => {
            let default_name = format!("{DEFAULT_JOB_NAME_PREFIX}{}", timestamp());
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
        "scp" => {
            let id = args.get(2).unwrap().to_string();
            let filename = args.get(3).unwrap().to_string();
            let destination = args.get(4).unwrap().to_string();
            println!("Copying {filename} to {destination} for job {id}");
            provider.scp(&id, &filename, &destination).await?;
        }
        _ => {
            println!("Invalid action");
        }
    }

    Ok(())
}
