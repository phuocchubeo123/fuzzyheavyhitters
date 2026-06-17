# MOSAIC

This is a Rust implementation of the Mosaic framework in the paper _Mosaic: A Modular Framework for Private Fuzzy Heavy Hitters_ (CCS 2026).
The description of the problem setting for Mosaic can be found in the [Problem Settings](#problem-settings) section, and we describe our solution in the [Mosaic's Solution](#mosaics-solution) section.
The details about the artifact, including [Installation](#installation) and [How to run](#how-to-run) is shown afterward.

# Problem Settings

## What problem does Mosaic solve?
We consider the _Private Fuzzy Heavy Hitters_ problem. 
Given a dataset, a _Heavy Hitter_ could be simply thought of as a _popular_ item in this set.
For example, if this is a dataset of emojis, we want to know which emoji is the most liked by users.
In some other datasets, such as for ride-sharing applications, the locations stored are _not exact_, hence we consider the _Fuzzy Heavy Hitters_ problem for this use case.

More formally, given a dataset of $d$-dimensional points, a _Fuzzy Heavy Hitter_ is defined based on the following three parameters:
- The distance radius $\delta$, which we confine to be only positive integer,
- The distance metric $d$, which is either $L_\infty$ or $L_p$, and
- The count threshold $t$.

The question that whether a point $x$ is a Fuzzy Heavy Hitter is equivalent to the following question:
**Does there exists at least $t$ other points $y$ in the dataset, such that $d(x, y) \le \delta$?**

## Private Fuzzy Heavy Hitters
The definition of _Fuzzy Heavy Hitters_ already half of the story, the other half is: What does _private_ mean in this case?
First, we have $n$ clients, each will provide a $d$-dimensional point e.g. his/her starting point of the ride.
Now, a data analyst only wants to learn the Fuzzy Heavy Hitters. 
He does not want to know each client's provided point, and vice versa, the client also does not want the point to be leaked to the analyst.

In more details, there are two problem settings for Private Fuzzy Heavy Hitters that we consider in Mosaic:
- **Known Dictionary**: The data analyst has a list of potential Fuzzy Heavy Hitters, and wants to find out which ones of them are actually popular.
- **Unknown Dictionary**: The data analyst does not have any information beforehand, and discovers all the heavy hitters from the clients' dataset.

## Mosaic's design
In Mosaic, we realize this functionality with the help from two servers, which will "collect" the points from clients in a private way, then do some aggregation to learn only the final Fuzzy Heavy Hitters.
We illustrate the protocol by implementing the `server`, the `client`, and the `dealer`. 
All CLI codes can be found in **src/bin/mosaic_end_to_end**.
- `server0` and `server1`: The source code for the servers' CLI is in **mosaic_server.rs**. At the start of the protocol, the servers receive _secret sharings_ (or rather _function secret sharings_) of all clients' points. 
Then two servers run a distributed protocol that either check the heavy hitters (_known dictionary_ setting) or discover the heavy hitters (_unknown dictionary_ setting).
- `client`: We use a single file to simulate and generate _function secret sharing_ for many client points, instead of spawning out one instance for each client. 
The CLI's source code can be found in **mosaic_client.rs**.
The clients go offline after sending the secret shared points to the servers.
- `dealer`: There are in fact two options to run the distributed protocol between the servers, with one including the help from a dealer. 
The dealer's task is to generate _correlated randomness_ that does not require any information about the actual protocol data.
This means the random correlations can be generated _continuously_, or _processing for life_, and can be sent to the servers upon request.
The CLI code for the dealer is in **mosaic_dealer.rs**.

For more details about the math, please refer to the paper.

## Threat Model
In Mosaic, the model we follow is:
- The two servers are _non-colluding_ e.g. two different non-profit organizations helping with this task.
- The clients cannot learn any other point, but can try to provide _malformed_ sharings of points to mess up the computation.

# Experimental Results

# Installation

This code has been tested with `Ubuntu 24.04.4 LTS`.

Dependencies:
- Rust: this code has been tested with `Rust 1.96.0` (ac68faa20 2026-05-25)
- All dependencies in Cargo.toml has been up-to-date at the moment of publishing this artifact.

Please refer to the following guide to install Rust in Ubuntu: [Digital Ocean Guide](https://www.digitalocean.com/community/tutorials/install-rust-on-ubuntu-linux).

After getting all the dependencies, to compile, simply run `cargo build`:
```
cargo build --release
```

# How to run
There are four parties involved in this protocol: `server0`, `server1`, `client` and `dealer`. 
Before running the protocol, you first need to prepare a list of clients' points, and also a config file.

## Prepare data
Simply create a json file, for example, the following json file contains two client points:
```javascript
[
  [
    XXX,
    XXX
  ],
  [
    XXX,
    XXX
  ]
]
```    
TODO: More details on the data we prepared in the experiments.

## Prepare the config file
The config format is as follow.
We include the config for all of our experiment runs in the **configs** folder.
```javascript
{
  "data_file": "path_to_client_points",
  "query_file": "path_to_server_points_if_apply",
  "protocol": {
    "delta": XXX, // distance threshold 
    "threshold": XXX, // count threshold
    "h1": XX,
    "h2": XX,
    "h3": XX,
    "d": X,
    "share_method": "FSS", // "FSS" or "OKVS"
    "dictionary_type": "Unknown", // "Known" or "Unknown"
    "check_method": "FSS", // "FSS" or "GC"
    "check_property": "Equality", // "Equality" or "MuBounded"
    "threshold_method": "FSS", // "FSS" or "GC"
    "distance_metric": "Linf", // "Linf", "L1", "L2", or "L3"
    "num_clients": XXXX
  },
  "network": {
    "server0_addr": "XXX.XX.XX.XX",
    "server1_addr": "XXX.XX.XX.XX",
    "server0_to_server1_port": XXXX,
    "dealer_to_server0_port": XXXX,
    "dealer_to_server1_port": XXXX,
    "client_to_server0_port": XXXX,
    "client_to_server1_port": XXXX
  },
  "output": {
    "verbose": true,
    "show_intermediate": false,
    "output_file": "results.json"
  }
}
```


## Running the protocol

Open four terminals (or four different machines, that can talk to each other through Tcp).
Please run the following four command lines for the four simulated parties:
- `server0`
```
cargo run --release --bin mosaic_server -- --side 0 --config (path_to_config) --threads (num_threads) 
```
- `server1`
```
cargo run --release --bin mosaic_server -- --side 1 --config (path_to_config) --threads (num_threads) 
```
- `client`
```
cargo run --release --bin mosaic_client -- --config (path_to_config)
```
- `dealer`
```
cargo run --release --bin mosaic_dealer --config (path_to_config) --threads (num_threads)
```

## Command line parameters 
- `config`: All four commands need a config parameter, please provide the path to the config file that you prepared in the [Prepare Config](#prepare-the-config-file) section.
- `threads`: Specify the number of threads that the two servers and the dealer use.
Currently we only tested the code for the case when the number of threads used by all these three parties are the same.
So please set `threads` to be the same in all three commands.

# Authors
Gayathri Garimella, _Brown University_

Peihan Miao, _Brown University_

Eileen Nolan, _Brown University_

Phuoc Van Long Pham, _Brown University_

Siddarth Sitaraman, _Brown University_

For more information about the implementation, please contact Phuoc: phuoc_van_long_pham@brown.edu.