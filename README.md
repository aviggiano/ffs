# ffs

ffs is a Fast Fuzzing Service CLI that runs your campaigns on the cloud.

## Installation

Install the latest released binary directly from crates.io:

```bash
cargo install ffs-cli
```

## Usage

A CLI tool for managing cloud computing jobs

```bash
Usage: ffs-cli [COMMAND]

Commands:
  init     Initialize the configuration with a cloud provider
  list     List all jobs
  start    Start a new job
  stop     Stop a running job
  tail     Tail logs from a job
  scp      Copy files from a job
  ssh      SSH into a job
  version  Display version information
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## SSH into a job

After listing jobs with `ffs-cli list`, you can quickly connect to one by ID:

```bash
ffs ssh <job-id>
```

This runs `ssh root@<job-ip>` using the job's public IPv4 address.
