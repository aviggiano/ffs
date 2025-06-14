use clap::{Parser, Subcommand};
use ffs::database::Database;
use ffs::providers::ProviderFactory;
use ffs::utils::timestamp;

const DEFAULT_PROVIDER: &str = "hetzner";
const DEFAULT_JOB_NAME_PREFIX: &str = "ffs-job-";

#[derive(Parser)]
#[command(name = "ffs")]
#[command(about = "A CLI tool for managing cloud computing jobs")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the configuration with a cloud provider
    Init {
        /// Cloud provider to use (hetzner, aws)
        #[arg(default_value = DEFAULT_PROVIDER)]
        provider: String,
    },
    /// List all jobs
    #[command(alias = "ls")]
    List,
    /// Start a new job
    Start {
        /// Name of the job to start
        name: Option<String>,
    },
    /// Stop a running job
    Stop {
        /// ID of the job to stop
        id: String,
    },
    /// Tail logs from a job
    Tail {
        /// Follow the log output
        #[arg(short = 'f', long)]
        follow: bool,
        /// ID of the job
        id: String,
    },
    /// Copy files from a job
    Scp {
        /// ID of the job
        id: String,
        /// Source file path
        filename: String,
        /// Destination path
        destination: String,
    },
    /// SSH into a job
    Ssh {
        /// ID of the job
        id: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    let mut database = Database::new();

    let provider = ProviderFactory::create_provider(
        database
            .get("provider")
            .unwrap_or_else(|| DEFAULT_PROVIDER.to_string())
            .as_str(),
    );

    match cli.command.unwrap_or(Commands::List) {
        Commands::Init {
            provider: provider_name,
        } => {
            database.set("provider", &provider_name)?;
            println!("Provider set to: {provider_name}");
        }
        Commands::List => {
            if let Ok(jobs) = provider.list_jobs().await {
                for job in jobs {
                    println!("{job:?}");
                }
            }
        }
        Commands::Start { name } => {
            let job_name =
                name.unwrap_or_else(|| format!("{DEFAULT_JOB_NAME_PREFIX}{}", timestamp()));
            println!("Starting job {job_name}");
            let job = provider.start_job(&job_name).await?;
            println!("Job {job:?} started");
        }
        Commands::Stop { id } => {
            println!("Stopping job {id}");
            let job = provider.stop_job(&id).await?;
            println!("Job {job:?} stopped");
        }
        Commands::Tail { follow, id } => {
            println!("Fetching logs for job {id}");
            provider.tail(&id, follow).await?;
        }
        Commands::Scp {
            id,
            filename,
            destination,
        } => {
            println!("Copying {filename} to {destination} for job {id}");
            provider.scp(&id, &filename, &destination).await?;
        }
        Commands::Ssh { id } => match provider.get_job(&id).await? {
            Some(job) => {
                println!("Connecting to {}", job.ipv4);
                std::process::Command::new("ssh")
                    .arg(format!("root@{}", job.ipv4))
                    .status()?;
            }
            None => println!("Job {id} not found"),
        },
    }

    Ok(())
}
