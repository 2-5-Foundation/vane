## Vane Node

# Alternative Running
 Install Rust && Clone the repo && Then do cargo build --release and Run 

### Rust Setup

First, complete the [basic Rust setup instructions](./docs/rust-setup.md).


## 1) Launching the network locally - Using Docker (Automated)

The default compose file automatically launches a local network containing multiple relaychain validators (polkadot nodes) and collators (Vane collators) which you can interact with using the [browser-based polkadotjs wallet](https://polkadot.js.org/apps/#/explorer). Additionally, docker can be used to launch a collator to supported testnets.
Follow these steps to prepare a local development environment :hammer_and_wrench:

### Setup

1. You will need to [install docker](https://www.docker.com/products/docker-desktop) to launch our pre-built containers
2. Checkout the repository

```bash
git clone --recursive https://github.com/2-5-Foundation/vane.git

cd vane
```

### Run

Launch a local network using docker compose:

```bash
docker-compose -f scripts/docker-compose.yml up -d
```

The `scripts` directory contains the docker-compose file which is used to launch the network using a single command. Launching with docker ensures parachains are registered to the relaychain automatically and will begin authoring blocks shortly after launch.

To ensure your network is functioning properly:

Confirm docker containers are running
```bash
docker ps
```

Now you can head over to polkadotjs and check whether you see the parachain running here
``` https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9944#/accounts```