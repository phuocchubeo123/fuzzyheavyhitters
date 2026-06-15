# MOSAIC

This is a Rust implementation of the Mosaic framework in the paper _Mosaic: A Modular Framework for Private Fuzzy Heavy Hitters_.
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

## The setting of Private Fuzzy Heavy Hitters
The definition of _Fuzzy Heavy Hitters_ already half of the story, the other half is: What does _private_ mean in this case?
First, we have $n$ clients, each will provide a $d$-dimensional point e.g. his/her starting point of the ride.
Now, a data analyst only wants to learn the Fuzzy Heavy Hitters. 
He does not want to know each client's provided point, and vice versa, the client also does not want the point to be leaked to the analyst.

In Mosaic, we realize this functionality with the help from two servers, which will "collect" the points from clients in a private way, then do some aggregation to learn only the final Fuzzy Heavy Hitters.

## Threat Model
In Mosaic, the model we follow is:
- The two servers are _non-colluding_ e.g. two different non-profit organizations helping with this task.
- The clients cannot learn any other point, but can try to provide _malformed_ sharings of points to mess up the computation.

# Mosaic's Solution

# Installation

Dependencies:
- Rust: this code has been tested with Rust 1.96.0 (ac68faa20 2026-05-25)


# How to run

To compile, set the Rust flag:
```
$ export RUSTFLAGS+="-C target-cpu=native" 
$ cargo build --release
```

You should prepare four terminals and one config file. First, run server0: 
```
$ cargo run --release --bin fhh_cli server0 --config (path_to_config) --threads (num_threads) 
```

Then, run server1:
```
$ cargo run --release --bin fhh_cli server0 --config (path_to_config) --threads (num_threads) 
```

Now, the servers should be ready to process client requests. 

```
$ cargo run --release --bin fhh_cli client --config (path_to_config)
```

Wait until the client sent through everything, run the dealer:
```
$ cargo run --release --bin fhh_cli dealer --config (path_to_config) --threads (num_threads)
```

# How to set up data files:
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

# What about the config?
The config format is as follow:
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
