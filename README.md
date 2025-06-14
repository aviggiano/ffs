# ffs

ffs is a Fast Fuzzing Service CLI that runs your campaigns on the cloud.

## Installation

Install the latest released binary directly from crates.io:

```bash
cargo install ffs
```


## SSH into a job

After listing jobs with `ffs ls`, you can quickly connect to one by ID:

```bash
ffs ssh <job-id>
```

This runs `ssh root@<job-ip>` using the job's public IPv4 address.

## View job logs

Use `ffs tail` to print logs for a job. Add `-f` to follow the logs in real time:

```bash
ffs tail -f <job-id>
```
