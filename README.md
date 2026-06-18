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
The clients goes offline after sending the secret shared points to the servers.
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

## Parameters
We choose computational security parameter $\kappa = 128$ and statistical security parameters $\lambda = 40$. 

1. **OKVS**: We use RB-OKVS, and choose a conservative choice of parameters in order to achieve $\lambda = 40$ statistical parameter for failure rate.
For more details, we set the OKVS rate to be $\epsilon = 0.1$, which means the number of columns is $1.1\times$ the number of rows. 
Furthermore, the number of columns is always at least $60$.
About the band-width, we set the band-width to be a fixed $100$, and to be exactly the number of columns in case this quantity is smaller than $100$.
This achieves the OKVS encoding failure probability $2^{-40}$ according to the [RB-OKVS](https://www.usenix.org/conference/usenixsecurity23/presentation/bienstock) paper.

2. **Input length** $h_1$: The input length just needs to be large enough to represent each coordinate. 
In the cleaned Ride Austin dataset, each coordinate can be represented by $11$ bits, hence we set $h_1 = 11$ in all of our experiments.

3. **FSS output** $h_2$: The FSS output for the _FSS-based_ approach is set differently from the _OKVS-based_ approach.
- _FSS-based_ approach: As correctness is guarantee for all points in the domain, we do not need to account for failure probability, hence the output simple needs to not cause overflow when comparing with the distance threshold.
For example, for distance metric $L_2$ with threshold $\delta = 5$, and there are $2$ dimensions, we can bound the FSS (distance) output dimension with $5^2 + 1$, and the maximum total distance calculated is $2 \times (5^2 + 1) = 52$, which can be represented in $6$ bits.
In our experiments for the unknown dictionary setting, we choose $\delta = 5$, and we only run the experiments for the case when the number of dimensions is $d = 2$. 
In the following table, we provide suggested $h_2$ parameter in the FSS-based approach, for number of dimensions $d=2$ and $d=4$.

<div align="center">

| distance metric | $L_{\infty}$ | $L_1$ | $L_2$ |
| --- | ---: | ---: | ---:|
| $h_2$ when $d=2$ | $1$ | $4$ | $6$ |
| $h_2$ when $d=4$ | $1$ | $5$ | $7$ |

</div>

- _OKVS-based_ approach: Choosing $h_2$ for the OKVS-based approach is more tricky due to the correctness probability. 
For keys that are not encoded in the OKVS, the decoding output is a random value, and with a small probability, the summed-up distance would still indicate that $d(x, y) \le \delta$ (which is wrong).
For example, when the output range is $2^{h_2}$, and we consider the distance metric $L_2$, with radius $\delta = 5$, the random summed-up distance can be a false-positive with probability $5^2 / 2^{h_2}$.
We further union-bound this probability with the total number of times that an OKVS is evaluated, which we estimate to be the number of clients.
Again, in the $L_2$ distance metric example, this probability would be estimated as $5^2 n / 2^{h_2}$, and we choose $h_2 \ge \log(25 n) + \kappa$ to satisfy the statistical security parameter.
The full table for the parameter $h_2$ that we chose for the experiments is shown below.

<div align="center">

| distance metric | $L_{\infty}$ | $L_1$ | $L_2$ |
| --- | ---: | ---: | ---:|
| $n = 21,581$ | $h_2 = 55$ | $h_2 = 58$ | $h_2 = 60$ |
| $n = 59,040$ | $h_2 = 56$ | $h_2 = 59$ | $h_2 = 61$ |
| $n = 115,174$ | $h_2 = 58$ | $h_2 = 61$ | $h_2 = 63$ |

</div>

4. **Fuzzy match output** $h_3$: The fuzzy match output is a mod $2^{h_3}$ value, but with value being only either $0$ (if $d(x, y)$ exceeds $\delta$, which means the two points are not close, hence not a fuzzy match) or $1$.
For each point $x$ in consideration, it is fuzzy-matched with exactly $n$ other points $y$, where $n$ is the number of clients, hence aggregating the results return a value that does not exceed $n$.
We just need to choose $h_3$ such that $2^{h_3} \ge n$. 
The reference for the parameter $h_3$ chosen in our experiments for the RideAustin dataset is included in the following table.

<div align="center">

| Dataset | busiest_day | busiest_week | busiest_month |
| --- | ---: | ---: | ---:|
| Number of client points | $21,581$ | $59,040$ | $115,174$ |
| Fuzzy match output $h_3$ | $15$ | $16$ | $18$ |

</div>



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

# How to run Mosaic
There are four parties involved in this protocol: `server0`, `server1`, `client` and `dealer`. 
Before running the protocol, you first need to prepare a list of clients' points, a main config file, and a separate network config file.

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

In our experiments, we clean the RideAustin dataset and obtain three sub-datasets with different sizes, in order to test the scalability of Mosaic. 
Each sub-dataset is stored in a `.csv`, that is available in the `data` folder in the Zenodo repository of Mosaic. 
For example, the csv file for the rides in the busiest day (the other two datasets are for the busiest week and the busiest month) can be found at `data/sample_busiest_day.csv` on Zenodo.
The summary of the size of each sub-dataset is shown in the following table.
<div align="center">

| Dataset | busiest_day | busiest_week | busiest_month |
| --- | ---: | ---: | ---:|
| Number of client points | $21,581$ | $59,040$ | $115,174$ |
| $2\%$ match threshold | $431$ | $1,180$ | $2,303$ |

</div>

The .csv files are then extracted into json files that contain a list of $2$-dimensional points (for the starting points of rides) or $4$-dimensional points (for the start and end points of rides), using the code in `src/bin/data/ride_austin_json_generator.rs`.
We can run the json file generator from the csv file with the following command:
```
cargo run --release --bin ride_austin_json_generator -- 
--input (input_csv) --output (output_json) 
--output-type (output_type)
```
Here, the `output-type` is $0$ if you want $2$-dimensional points (corresponding to the start of a ride), and $1$ if you want $4$-dimensional points (corresponding to the start and the end of a ride).

To sample a dictionary for the servers to rely on in the _known dictionary_ setting, use `ride_austin_json_sample.rs`.
```
cargo run --release --bin ride_austin_json_sampler --
--input (input_csv) --output (output_json) 
--query-num (dictionary_size)
--output-type (output_type)
```
All prepared .json files for our experiments are available on Zenodo, in the same `data` folder. 

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
    "check_property": "Equality", // "Equality" or "MuBounded"
    "distance_metric": "Linf", // "Linf", "L1", "L2", or "L3"
    "num_clients": XXXX
  },
  "output": {
    "verbose": true,
    "show_intermediate": false,
    "output_file": "results.json"
  }
}
```
The method settings now live in a separate file at `configs/method_config.json`:
```javascript
{
  "check_method": "GC", // "GC" or "FSS"
  "threshold_method": "GC" // "GC" or "FSS"
}
```
The network settings now live in a separate file at `configs/network_config.json`:
```javascript
{
  "network": {
    "server0_addr": "XXX.XX.XX.XX",
    "server1_addr": "XXX.XX.XX.XX",
    "server0_to_server1_port": XXXX,
    "dealer_to_server0_port": XXXX,
    "dealer_to_server1_port": XXXX,
    "client_to_server0_port": XXXX,
    "client_to_server1_port": XXXX
  }
}
```


## Running the Mosaic protocol

Open four terminals (or four different machines, that can talk to each other through Tcp).
Please run the following four command lines for the four simulated parties:
- `server0`
```
cargo run --release --bin mosaic_server -- --side 0 
--config (path_to_config) 
--network-config (path_to_network_config) --method-config (path_to_method_config) 
--threads (num_threads)
```
- `server1`
```
cargo run --release --bin mosaic_server -- --side 1 
--config (path_to_config) 
--network-config (path_to_network_config) --method-config (path_to_method_config) 
--threads (num_threads)
```
- `client`
```
cargo run --release --bin mosaic_client -- 
--config (path_to_config) --network-config (path_to_network_config)
```
- `dealer`
```
cargo run --release --bin mosaic_dealer -- 
--config (path_to_config) --network-config (path_to_network_config)
--threads (num_threads)
```

Command line parameters:
- `config`: All four commands need a config parameter, please provide the path to the main config file that you prepared in the [Prepare Config](#prepare-the-config-file) section.
- `network-config`: The Mosaic and naive servers need the separate network config file. Use `configs/network_config.json` or point this flag at your own copy.
- `method-config`: The server method config file. Use `configs/method_config.json` or point this flag at your own copy.
- `threads`: Specify the number of threads that the two servers and the dealer use.
Currently we only tested the code for the case when the number of threads used by all these three parties are the same.
So please set `threads` to be the same in all three commands.

## Running the naive solution 
The naive solution only has two servers and a client, since we only implement the naive solution using Garbled Circuit for fuzzy matching.
You can reuse the main config file that you prepared for the Mosaic's solution runs, and the same `configs/network_config.json` file.
- `server0`
```
cargo run --release --bin naive_server -- --side 0 
--config (path_to_config) --network-config configs/network_config.json --num-threads (num_threads)
```
- `server1`
```
cargo run --release --bin naive_server -- --side 1 --config (path_to_config) --network-config configs/network_config.json --num-threads (num_threads)
```
- `client`
```
cargo run --release --bin naive_client -- --config (path_to_config) --network-config configs/network_config.json
```

# Optimizations
Since the time of publication, there has been two optimizations that makes the code run much faster now and also consumes less RAM. 

1. In the original code, we in fact forgot to parallelize the _FSS evaluation_ step in each loop iteration. 
Putting parallelization in this part reduces $8\times$ the running time compared to what was shown in Table 4 in the submission.

2. The original code stores every FSS evaluations in each iteration of the loop, with the goal in mind to be faster FSS evaluation for the next prefix. 
However, later in the run, the number of candidate prefixes grows into a large number, which causes the amount of FSS evaluations stored in RAM becoming to large ($>140$ GB).
In this new version, we implement the binary tree expansion in a more "streaming" fashion, which reduces the peak memory consumption into less $30$ GB. 
This implementation also introduces a trade-off, with more RAM consumption might be traded for even faster runs.


# Authors
Gayathri Garimella, _Brown University_

Peihan Miao, _Brown University_

Eileen Nolan, _Brown University_

Phuoc Van Long Pham, _Brown University_

Siddarth Sitaraman, _Brown University_

For more information about the implementation, please contact Phuoc: phuoc_van_long_pham@brown.edu.
