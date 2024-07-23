![zakuro Logo](imgs/zakuro-banner.png)

# Zakuro AI Command Line Tool (zc)

--------------------------------------------------------------------------------


<!-- <p align="center">
      <img alt="Build" src="https://github.com/zakuro-ai/zakuro/actions/workflows/trigger.yml/badge.svg?branch=master">
      <img alt="GitHub" src="https://img.shields.io/github/license/zakuro-ai/zakuro.svg?color=blue">
      <img alt="GitHub release" src="https://img.shields.io/github/release/zakuro-ai/zakuro.svg">
</p> -->


<p align="center">
  <a href="#overview">Overview</a> •
  <a href="#installation">Installation</a> •
  <a href="#usage">Usage</a> •
  <a href="#development">Development</a> •
  <a href="#troubleshooting">Troubleshooting</a> •
</p>

## Overview

`zc` is the Zakuro AI command line tool designed to streamline operations within the Zakuro ecosystem. It provides various commands for managing Docker containers, updating configurations, retrieving cluster information, and more.

## Installation

### Prerequisites

- Rust (for building the `zc` tool)
- Docker and Docker Compose

### Building and Running

To build and run the `zc` tool, follow these steps:

1. **Clone the repository:**
   ```bash
   git clone https://github.com/zakuroai/zc.git
   cd zc
   ```

2. **Build the tool:**
   ```bash
   make build
   ```

3. **Run the tool:**
   ```bash
   ./zc
   ```

## Usage

### General Usage

```
Usage: zc [OPTIONS] COMMAND

A self-sufficient runtime for Zakuro

Options:
  -d, --docker    Execute the commands from zk0.
  -v, --version   Get the version of the current command line.
  -h, --help      Print this help.

Commands:
  connect         Enter zk0 in interactive mode.
  update          Update the command line.
  pull            Pull updated images.
  images          List Zakuro images built on the machine.
  ps              List current running Zakuro containers.
  kill            Remove current running Zakuro containers.
  restart         Restart the containers with updated images.
  wg0ip           Get the IP in the cluster.
  rmi             Remove Zakuro images.
  workers         List all workers connected to the main clusters (10.13.13.2).
  context <path>  Set new Zakuro context | untested.
```

### Command Descriptions

- `connect`: Enter zk0 in interactive mode.
- `update`: Update the command line tool by running a script from the Zakuro server.
- `pull`: Pull updated images.
- `images`: List Zakuro images built on the machine.
- `ps`: List current running Zakuro containers.
- `kill`: Remove current running Zakuro containers.
- `restart`: Restart the containers with updated images.
- `wg0ip`: Get the IP in the cluster.
- `rmi`: Remove Zakuro images.
- `workers`: List all workers connected to the main clusters (10.13.13.2).
- `context <path>`: Set a new Zakuro context (untested).

### Example

To update the `zc` tool:
```bash
zc update
```

To list all current running Zakuro containers:
```bash
zc ps
```

## Development

### Docker Compose

The `docker-compose.yml` file is used to build and run the Docker container for `zc`. The Docker image is built using the specified Dockerfile and context.

### Makefile

The `Makefile` includes commands to automate building, running, and debugging the `zc` tool.

#### Common Commands

- `make build`: Build the `zc` tool and Docker image.
- `make run`: Build and run the `zc` tool.
- `make release`: Execute the release version of the tool.
- `make debug`: Build and run the tool in debug mode with full backtrace.

## Troubleshooting

If you encounter issues running `zc`, ensure you have the required environment variables set, particularly `ZAKURO_AUTH`.

## Additional Resources

For more detailed documentation, visit [Zakuro AI Documentation](https://docs.zakuro.ai/).