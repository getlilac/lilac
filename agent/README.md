# Lilac CLI and Agent

Welcome to the Lilac CLI and Agent documentation. This guide provides instructions for both submitting training jobs and running the agent on your compute nodes.

## Overview

The `lilac` binary serves two primary purposes:

1.  **User CLI**: For data scientists and ML engineers to submit training jobs to the Lilac control plane.
2.  **Agent**: A daemon that runs on compute nodes, connects to the control plane, and executes assigned jobs.

These two modes have separate configurations to ensure clarity and security.

---

## For Users: Submitting Jobs

As a user, your primary interaction with `lilac` will be to configure your API credentials and submit jobs.

### 1. Installation

We provide an easy-to-use installer script for macOS and Linux. To install the `lilac`, run the following command in your terminal. It will automatically detect your operating system, download the correct binary from the latest GitHub release, and install it into `/usr/local/bin`.

```bash
curl -sSL https://raw.githubusercontent.com/getlilac/lilac/main/scripts/install.sh | sh
```

The script may ask for your password to move the binary into `/usr/local/bin` using `sudo`. Once installed, you can verify it's working by running:

```bash
lilac --version
```

### 2. Configuration

To configure the CLI, run the interactive `configure` command:

```bash
lilac configure
```

This will prompt you for the following information:

-   **Lilac API Endpoint**: The URL of the Lilac control plane (e.g., `http://lilac.example.com`).
-   **User API Key**: Your personal API key for authenticating with the Lilac API. You can get this by going to the UI and clicking the key icon in the sidebar.

This command creates a configuration file at `~/.lilac/config.toml`.

### 3. Submitting a Job

The `submit` command allows you to submit a pre-built Docker image for execution on the cluster.

```bash
lilac submit
```

The command will interactively guide you through the submission process, asking for:

-   **Job Name**: A descriptive name for your job.
-   **Docker Image URI**: The full URI of your job's Docker image (e.g., `your-registry.com/your-repo/your-image:latest`).
-   **Queue**: The compute queue to submit the job to.
-   **Resource Requirements**: CPU, memory, and GPU requirements for the job.

> **Note:** The CLI does not build Docker images (yet). You are responsible for building your image and pushing it to a registry that the Lilac agent can access.

### Non-Interactive Submission

For automated workflows, you can provide all job parameters as command-line arguments. If all required arguments are provided along with the `--non-interactive` flag, the job will be submitted directly without interactive prompts.

| Argument | Description | Example |
|---|---|---|
| `--name` | The name of the job. | `--name my-training-job` |
| `--docker-uri` | The Docker image URI for the job. | `--docker-uri my-registry/my-image:latest` |
| `--queue-id` | The ID of the queue to submit the job to. | `--queue-id 8943e3de-e745-42bc-9c85-a037c58ac03a` |
| `--cpu` | The amount of CPU to request in millicores. | `--cpu 1000` |
| `--memory` | The amount of memory to request in MB. | `--memory 2048` |
| `--gpu-count` | The number of GPUs to request. | `--gpu-count 1` |
| `--gpu-model` | The model of GPU to request (e.g., A100). | `--gpu-model A100` |
| `--gpu-memory`| The minimum VRAM per GPU in GB. | `--gpu-memory 16` |
| `--non-interactive` | A flag to skip all interactive prompts. | `--non-interactive` |

If you provide arguments without the `--non-interactive` flag, they will be used as default values in the interactive prompts.
---

## For Administrators: Running the Agent

As an administrator, you will configure and run the Lilac agent on the compute nodes in your cluster.

### 1. Configuration

To configure the agent, run the interactive `agent configure` command:

```bash
lilac agent configure
```

This will prompt you for:

-   **Lilac API Endpoint**: The URL of the Lilac control plane.
-   **Cluster API Key**: The shared secret for authenticating agents with the Lilac cluster.
-   **Private Docker Registry (Optional)**: If your jobs use images from a private registry, you can configure the credentials here.

This command creates a configuration file at `~/.lilac/agent.toml`.

### 2. Starting the Agent

Once configured, you can start the agent daemon with the following command:

```bash
lilac agent start
```

The agent will connect to the control plane, report its available resources, and wait for jobs to be assigned.

> **Note on GPUs**: For the agent to utilize and report on NVIDIA GPU resources, the host machine must have NVIDIA drivers installed.

### 4. Running the Universal Agent (Docker)

The recommended way to run the agent is using the official "universal" Docker image. This image is self-contained and includes all necessary dependencies, including its own Docker daemon (Docker-in-Docker) and the NVIDIA Container Toolkit.

Use the following command to run or update the agent. This command will gracefully stop and remove any old agent container before pulling the latest image and starting a new one.

**Important:** You must replace `<YOUR_API_ENDPOINT>` and `<YOUR_CLUSTER_API_KEY>` with your actual credentials.

```bash
docker stop lilac-agent && docker rm lilac-agent && \
docker pull getlilac/lilac-agent:latest && \
docker run -d \
  --name lilac-agent \
  --restart always \
  --privileged \
  -e LILAC_API_ENDPOINT="<YOUR_API_ENDPOINT>" \
  -e LILAC_CLUSTER_API_KEY="<YOUR_CLUSTER_API_KEY>" \
  --gpus all \
  getlilac/lilac-agent:latest
```

### 5. Connecting to Private Registries

If your jobs use images from a private registry, you can configure the credentials using environment variables.

Below is the full command template. Add the three `LILAC_PRIVATE_REGISTRY_*` variables to connect to your private registry.

```bash
docker stop lilac-agent && docker rm lilac-agent && \
docker pull getlilac/lilac-agent:latest && \
docker run -d \
  --name lilac-agent \
  --restart always \
  --privileged \
  -e LILAC_API_ENDPOINT="<YOUR_API_ENDPOINT>" \
  -e LILAC_CLUSTER_API_KEY="<YOUR_CLUSTER_API_KEY>" \
  -e LILAC_PRIVATE_REGISTRY_URL="<YOUR_REGISTRY_URL>" \
  -e LILAC_PRIVATE_REGISTRY_USERNAME="<YOUR_REGISTRY_USERNAME>" \
  -e LILAC_PRIVATE_REGISTRY_PASSWORD="<YOUR_REGISTRY_PASSWORD_OR_TOKEN>" \
  --gpus all \
  getlilac/lilac-agent:latest
```

#### Registry-Specific Configurations

Here are the correct values to use for popular container registries:

| Registry | `LILAC_PRIVATE_REGISTRY_URL` | `LILAC_PRIVATE_REGISTRY_USERNAME` | `LILAC_PRIVATE_REGISTRY_PASSWORD` |
|---|---|---|---|
| **Docker Hub** | `https://index.docker.io/v1/` | Your Docker Hub Username | Your Personal Access Token (PAT) |
| **Google (GCR)** | `https://gcr.io` | `_json_key` | The full content of your JSON service account key |
| **Amazon (ECR)** | `https://<aws_account_id>.dkr.ecr.<region>.amazonaws.com` | `AWS` | The temporary token from `aws ecr get-login-password` |

> #### **Known Limitation: AWS ECR**
>
> AWS ECR tokens are temporary (typically valid for 12 hours). The current agent version does **not** automatically refresh these tokens. This means that after the token expires, the agent will fail to pull new images from ECR. Proper support for ECR's token refresh mechanism is coming soon. For now, the agent works best with registries that use static, long-lived credentials like Personal Access Tokens.

### 6. Minimum Requirements

While the agent itself is lightweight, it needs to handle Docker operations, which can be resource-intensive, especially during the `docker pull` phase for large images. To ensure reliable operation, we recommend the following minimum resources for any environment running the agent:
>
>   **CPU**: 1 core

>   **Memory**: 2 GB RAM
>

>   **Disk Space**: 100GB
>
 Running the agent with fewer resources may lead to instability, especially OOM (Out of Memory) errors when pulling large Docker images. PyTorch jobs can rapidly fill up disk space, up to 100GB. If you plan on running simpler jobs, you can get away with significantly less disk space.

---

## Advanced Configuration

Both user and agent configurations can be managed via environment variables, which is ideal for automated deployments (e.g., Kubernetes, systemd).

### Configuration Files

-   **User Config**: `~/.lilac/config.toml`
-   **Agent Config**: `~/.lilac/agent.toml`

### Environment Variables

Environment variables override settings from the configuration files.

| Variable                          | Description                                | Relevant To |
| --------------------------------- | ------------------------------------------ | ----------- |
| `LILAC_API_ENDPOINT`              | The Lilac control plane URL.               | User & Agent|
| `LILAC_USER_API_KEY`              | Your personal user API key.                | User        |
| `LILAC_CLUSTER_API_KEY`           | The shared cluster API key.                | Agent       |
| `LILAC_NODE_ID`                   | A unique ID for the node (optional).       | Agent       |
| `LILAC_PRIVATE_REGISTRY_URL`      | URL of the private Docker registry.        | Agent       |
| `LILAC_PRIVATE_REGISTRY_USERNAME` | Username for the private registry.         | Agent       |
| `LILAC_PRIVATE_REGISTRY_PASSWORD` | Password or token for the private registry.| Agent       |

---

## Known Limitations

### Resource Reporting

The agent's resource discovery mechanism is designed to be accurate in the most common deployment scenarios. However, there are some nuances to be aware of:

*   **Linux Containers**: When running in a Linux container with resource limits, the agent will correctly report the container's allocated memory and CPU by reading from the `cgroup` filesystem.
*   **Bare-Metal (Linux, macOS, Windows)**: When running directly on a host machine, the agent will report the total resources of that machine.
*   **Docker Desktop (macOS)**: When running in a non-linux container on Docker Desktop for Mac, the agent will report the resources of the Docker VM (e.g., 8GB RAM, 8 Cores by default), not the entire Mac's resources.
*   **Windows Containers**: When running in a non-linux Windows container with resource limits, the agent will fall back to reporting the host's total resources. This is an unsupported configuration, and for accurate scheduling, the agent should be run in a Linux container.
