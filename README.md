![Lilac](/docs/images/lilac.jpg)
<div align="center">
  <a href="https://www.ycombinator.com/companies/lilac">
    <img alt="Y Combinator S25" src="https://img.shields.io/badge/Combinator-S25-orange?logo=ycombinator&labelColor=white" />
  </a>
</div>

---
<div align="center">

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Discord](https://img.shields.io/badge/Discord-7289DA?logo=discord&logoColor=white)](https://discord.com/invite/getlilac)
[![Version](https://img.shields.io/github/v/release/getlilac/lilac)](https://github.com/getlilac/lilac/releases)
[![Lilac Docs](https://img.shields.io/badge/Documentation-blue)](https://docs.getlilac.com)
</div>


Lilac ensures your data scientists always have enough GPUs for their work. We seamlessly connect compute from any source, on-prem or cloud. We provide a web-based UI for management, a powerful control plane for scheduling, and a lightweight agent for job execution.


![Lilac Demo](/docs/images/demo.gif)

## Quick Start

Get up and running with Lilac in just a few steps.

**1. Install the CLI**

We provide a simple installer script for macOS and Linux. Run the following command to automatically download and install the latest version:

```bash
curl -sSL https://raw.githubusercontent.com/getlilac/lilac/main/scripts/install.sh | sh
```

**2. Run the Lilac Platform**

First, create the necessary configuration files from the provided examples:
```bash
cp ./backend/lilac.example.toml ./backend/lilac.toml
cp ./docker/.env.example ./docker/.env
```

**Important:** If you are running Lilac on a remote server, you must edit the `docker/.env` file and replace `http://localhost:8081` with the public IP address or domain name of your server.

Then, to run Lilac using Docker Compose, simply run the following command. This will start a local PostgreSQL container, the Lilac backend (http://localhost:8081), and the Lilac web UI (http://localhost:8080).

```bash
docker compose --env-file ./docker/.env -f ./docker/docker-compose.dev.yml --profile postgres up
```

If you would like to bring your own PostgreSQL database, modify the `LILAC__DATABASE_URL` env var in the `docker-compose.dev.yml` file to point to your database endpoint. The start Lilac without the local Postgres instance:

```bash
docker compose -f ./docker/docker-compose.dev.yml up
```

**3. Run an Agent**

Once the platform is running, you can start an agent on any machine to add it to your compute cluster. The recommended method is to use our universal Docker image.

In order to get a cluster API key, you must first create a cluster in the UI (http://localhost:8080/clusters). Then you can navigate to the "API Keys" tab in the Cluster's details page and generate an API key. This API key is how Lilac identifies which cluster to associate your node to.

```bash
docker run -d \
  --name lilac-agent \
  --restart always \
  --privileged \
  -e LILAC_API_ENDPOINT="<YOUR_API_ENDPOINT>" \
  -e LILAC_CLUSTER_API_KEY="<YOUR_CLUSTER_API_KEY>" \
  # Optional: Add the following three lines to connect to a private Docker registry
  # -e LILAC_PRIVATE_REGISTRY_URL="<YOUR_REGISTRY_URL>" \
  # -e LILAC_PRIVATE_REGISTRY_USERNAME="<YOUR_REGISTRY_USERNAME>" \
  # -e LILAC_PRIVATE_REGISTRY_PASSWORD="<YOUR_REGISTRY_PASSWORD_OR_TOKEN>" \
  --gpus all \
  getlilac/lilac-agent:latest
```

**4. Submit a Job**

Configure the CLI with your user credentials:
```bash
lilac configure
```

Then, you can submit your first job to the cluster:
```bash
lilac submit
```

---

## Features

*   **Distributed Job Execution**: Run containerized jobs across a cluster of nodes.
*   **Web-Based UI**: A user-friendly interface for managing the Lilac cluster.
*   **RESTful API**: A comprehensive API for programmatic access to the platform.
*   **Job Queues**: Organize jobs into queues with different priorities.
*   **Resource-Based Scheduling**: The scheduler assigns jobs to nodes based on their resource availability and the job's requirements.
*   **Real-Time Monitoring**: Monitor the status of nodes, queues, and jobs in real-time.
*   **User and API Key Management**: Securely manage user accounts and API keys.

### Coming soon

*   **Instance Pools**: Pool instances across many cloud providers into a single, autoscaled cluster. Simply select your cloud providers and the compute you need, and Lilac will provision the instances for you.
*   **Workspaces**: Run Jupyter notebooks, VSCode, or bring your own Docker image on your clusters.
*   **Data Sources**: Connect your data source into Lilac to use them seamlessly in workspaces or training jobs.
*   **Runtime Logs**: Get improved visibility into your workloads. Export your logs to your observability platform of choice.
*   **SSO & RBAC**: Bring your own SSO and manage your users with fine-grained access control.

---

## Agent & CLI

The `lilac` CLI is the primary tool for interacting with the Lilac platform. For detailed instructions on all CLI commands, agent configuration, connecting to private registries, and more, please see **[Agent and CLI README](./agent/README.md)**.

## Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

## License

Lilac is licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).
