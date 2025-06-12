# ffs

ffs is a Fast Fuzzing Service CLI that runs your campaigns on the cloud.

## SSH into a job

After listing jobs with `ffs ls`, you can quickly connect to one by ID:

```bash
ffs ssh <job-id>
```

This runs `ssh root@<job-ip>` using the job's public IPv4 address.
