use clap::Parser;
use mosaic::{
    channel::{listen_to, setup_parallel_channels, CommTrackingChannel},
    configs::{cli_config::CliConfig, method_config::MethodConfig, network_config::NetworkConfig},
    fuzzy_match::protocol::MosaicProtocol,
    randomness::prg::PRG,
};
use std::fs;
use scuttlebutt::channel::AbstractChannel;

/// Load query points from JSON file
fn load_query_points(file_path: &str) -> Result<Vec<Vec<u128>>, String> {
    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read query file {}: {}", file_path, e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse query file {}: {}", file_path, e))
}


struct NetworkServerSetup {
    client_channel: CommTrackingChannel,
    other_server_channels: Vec<CommTrackingChannel>,
    signal_dealer_channels: Vec<CommTrackingChannel>,
    check_dealer_channels: Vec<CommTrackingChannel>,
    threshold_dealer_channels: Vec<CommTrackingChannel>,
}

fn send_ready_signal(client_channel: &mut CommTrackingChannel, server_id: u8) -> Result<(), String> {
    client_channel
        .write_bytes(b"hi")
        .map_err(|e| format!("Server {}: failed to send ready signal: {}", server_id, e))?;
    client_channel
        .flush()
        .map_err(|e| format!("Server {}: failed to flush ready signal: {}", server_id, e))?;
    println!("Server {}: sent ready signal to client", server_id);
    Ok(())
}

fn setup_network_server(
    network_config_path: &str,
    is_server1: bool,
    num_threads: usize,
) -> Result<NetworkServerSetup, String> {
    let server_id = if is_server1 { 1 } else { 0 };
    let network_config = NetworkConfig::from_file(network_config_path)?;

    let server_addr = if is_server1 {
        network_config.server1_addr.clone()
    } else {
        network_config.server0_addr.clone()
    };

    let client_to_server_port = if is_server1 {
        network_config.client_to_server1_port
    } else {
        network_config.client_to_server0_port
    };

    let mut client_channel = listen_to(server_addr.clone(), client_to_server_port)
        .map_err(|e| format!("Failed to listen for client connection: {}", e))?;

    let server0_addr = &network_config.server0_addr;
    let server0_to_server1_port = network_config.server0_to_server1_port;

    println!(
        "Server {}: Setting up {} inter-server channels...",
        server_id, num_threads
    );

    let other_server_channels = if is_server1 {
        setup_parallel_channels(true, num_threads, server0_addr, server0_to_server1_port)
    } else {
        setup_parallel_channels(false, num_threads, server0_addr, server0_to_server1_port)
    }?;

    println!(
        "Server {}: Successfully established {} inter-server channels",
        server_id,
        other_server_channels.len()
    );

    let num_dealer_channels = num_threads;
    println!(
        "Server {}: Setting up {} dealer channels...",
        server_id, num_dealer_channels
    );

    let dealer_to_server_port = if is_server1 {
        network_config.dealer_to_server1_port
    } else {
        network_config.dealer_to_server0_port
    };

    let signal_dealer_channels = setup_parallel_channels(
        false,
        num_dealer_channels,
        &server_addr,
        dealer_to_server_port,
    )?;
    let check_dealer_channels = setup_parallel_channels(
        false,
        num_dealer_channels,
        &server_addr,
        dealer_to_server_port + num_dealer_channels as u16,
    )?;
    let threshold_dealer_channels = setup_parallel_channels(
        false,
        num_dealer_channels,
        &server_addr,
        dealer_to_server_port + 2 * num_dealer_channels as u16,
    )?;

    println!(
        "Server {}: Successfully established {} check dealer channels and {} threshold dealer channels",
        server_id,
        check_dealer_channels.len(),
        threshold_dealer_channels.len()
    );

    send_ready_signal(&mut client_channel, server_id)?;

    Ok(NetworkServerSetup {
        client_channel,
        other_server_channels,
        signal_dealer_channels,
        check_dealer_channels,
        threshold_dealer_channels,
    })
}
fn print_server_summary(
    server_id: u8,
    protocol_time: std::time::Duration,
    other_server_channels: &[CommTrackingChannel],
    signal_dealer_channels: &[CommTrackingChannel],
    check_dealer_channels: &[CommTrackingChannel],
    threshold_dealer_channels: &[CommTrackingChannel],
) {
    let mut total_other_server_bytes_sent = 0;
    let mut total_other_server_bytes_received = 0;
    let mut total_dealer_bytes_sent = 0;
    let mut total_dealer_bytes_received = 0;

    for channel in other_server_channels {
        let (sent, received) = channel.get_communication_stats();
        total_other_server_bytes_sent += sent;
        total_other_server_bytes_received += received;
    }

    for channel in signal_dealer_channels {
        let (sent, received) = channel.get_communication_stats();
        total_dealer_bytes_sent += sent;
        total_dealer_bytes_received += received;
    }
    for channel in check_dealer_channels {
        let (sent, received) = channel.get_communication_stats();
        total_dealer_bytes_sent += sent;
        total_dealer_bytes_received += received;
    }
    for channel in threshold_dealer_channels {
        let (sent, received) = channel.get_communication_stats();
        total_dealer_bytes_sent += sent;
        total_dealer_bytes_received += received;
    }

    println!("\n=== Server {} Performance Summary ===", server_id);
    println!("📊 Protocol execution time: {:.2?}", protocol_time);
    println!(
        "📡 Communication with other server ({} channels):",
        other_server_channels.len()
    );
    println!(
        "   Bytes sent: {} bytes ({:.2} KB)",
        total_other_server_bytes_sent,
        total_other_server_bytes_sent as f64 / 1024.0
    );
    println!(
        "   Bytes received: {} bytes ({:.2} KB)",
        total_other_server_bytes_received,
        total_other_server_bytes_received as f64 / 1024.0
    );
    println!(
        "📡 Communication with dealer ({} channels):",
        signal_dealer_channels.len()
    );
    println!(
        "   Bytes sent: {} bytes ({:.2} KB)",
        total_dealer_bytes_sent,
        total_dealer_bytes_sent as f64 / 1024.0
    );
    println!(
        "   Bytes received: {} bytes ({:.2} KB)",
        total_dealer_bytes_received,
        total_dealer_bytes_received as f64 / 1024.0
    );
    println!(
        "📡 Total communication: {} bytes ({:.2} KB)",
        total_other_server_bytes_sent
            + total_other_server_bytes_received
            + total_dealer_bytes_sent
            + total_dealer_bytes_received,
        (total_other_server_bytes_sent
            + total_other_server_bytes_received
            + total_dealer_bytes_sent
            + total_dealer_bytes_received) as f64
            / 1024.0
    );
}
/// Run server for both known and unknown dictionary cases
fn run_server(config_path: &str, network_config_path: &str, method_config_path: &str, is_server1: bool, num_threads: usize) -> Result<(), String> {
    if num_threads == 0 {
        return Err("--threads must be greater than 0".to_string());
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .map_err(|e| format!("Failed to configure Rayon thread pool: {}", e))?;

    let cli_config = CliConfig::from_file(config_path)?;
    let method_config = MethodConfig::from_file(method_config_path)?;
    let server_id = if is_server1 { 1 } else { 0 };
    let is_known_dictionary = cli_config.protocol.dictionary_type == "Known";

    println!(
        "Server {} starting {} dictionary protocol with {} parallel threads...",
        server_id,
        if is_known_dictionary {
            "known"
        } else {
            "unknown"
        },
        num_threads
    );

    let NetworkServerSetup {
        mut client_channel,
        mut other_server_channels,
        mut signal_dealer_channels,
        mut check_dealer_channels,
        mut threshold_dealer_channels,
    } = setup_network_server(network_config_path, is_server1, num_threads)?;

    // Create protocol configuration
    let protocol_parameters = cli_config.protocol.clone();
    let mut protocol = MosaicProtocol::new(
        protocol_parameters.clone(),
        method_config.clone(),
        is_server1,
        protocol_parameters.clone().enable_sketch,
        protocol_parameters.clone().num_clients,
        protocol_parameters.clone().match_threshold,
    );

    println!("Server {}: Receiving shares from client...", server_id);
    let mut shares = protocol
        .receive_client_shares(&mut client_channel)
        .map_err(|e| format!("Failed to receive client shares: {}", e))?;
    println!(
        "Server {}: Received {} shares from client",
        server_id,
        shares.len()
    );
    let sketch_data = if protocol_parameters.enable_sketch {
        println!("Server {}: Receiving sketch data from client...", server_id);
        let data = protocol
            .receive_client_sketch_data(&mut client_channel)
            .map_err(|e| format!("Failed to receive client sketch data: {}", e))?;
        println!(
            "Server {}: Received sketch data for {} shares",
            server_id,
            data.len()
        );
        Some(data)
    } else {
        None
    };

    if protocol_parameters.enable_sketch {
        println!("Server {}: Verifying client shares using sketches...", server_id);

        let start_sketch = std::time::Instant::now();

        let sketch_data = sketch_data
            .as_ref()
            .ok_or_else(|| "Sketch data missing while sketching is enabled".to_string())?;
        let (malicious_flags, bad_count) = {
            let prg_seed = [0u8; 16]; // Change later, need to exchange seed between servers
            let mut prg = PRG::new(Some(&prg_seed), 0);
            let flags = protocol
                .verify_client_shared_ranges(
                    &shares,
                    sketch_data,
                    &mut prg,
                    &mut other_server_channels,
                )
                .map_err(|e| format!("Failed to verify client shares: {}", e))?;
            let bad_count = flags.iter().filter(|&&flag| flag).count();
            (flags, bad_count)
        };
        let total = malicious_flags.len();
        if bad_count > 0 {
            println!(
                "Server {}: Removing {} of {} shares flagged by sketch verification",
                server_id, bad_count, total
            );
        }
        shares = shares
            .into_iter()
            .zip(malicious_flags.iter())
            .filter_map(|(share, flag)| if *flag { None } else { Some(share) })
            .collect();
        if bad_count > 0 {
            protocol = MosaicProtocol::new(
                protocol_parameters.clone(),
                method_config.clone(),
                is_server1,
                protocol_parameters.enable_sketch,
                shares.len(),
                protocol_parameters.match_threshold,
            );
        }

        println!(
            "Server {}: Sketch verification completed in {:.2?}",
            server_id,
            start_sketch.elapsed()
        );

    }

    println!("Server {}: Running protocol...", server_id);

    let start_time = std::time::Instant::now();

    let protocol_time = if is_known_dictionary {
        // Load query points for known dictionary
        println!(
            "Server {}: Loading query points from {}",
            server_id, cli_config.query_file
        );
        let query_points = load_query_points(&cli_config.query_file)?;

        // Run the protocol for known dictionary
        println!(
            "Server {}: Using {} dealer channels and {} server channels for parallel processing",
            server_id,
            signal_dealer_channels.len(),
            other_server_channels.len()
        );

        let results = protocol.run_server_known_dictionary_parallel(
            &shares,
            &query_points,
            &mut signal_dealer_channels,
            &mut check_dealer_channels,
            &mut threshold_dealer_channels,
            &mut other_server_channels,
        )?;
        println!("Server {}: Protocol execution completed", server_id);
        println!(
            "Found {} heavy hitters.",
            results.len(),
        );

        start_time.elapsed()
    } else {
        println!(
            "Server {}: Running unknown dictionary protocol...",
            server_id
        );

        // Run the protocol for unknown dictionary
        let heavy_hitters = protocol.run_server_unknown_dictionary_parallel2(
            &shares,
            &mut signal_dealer_channels,
            &mut check_dealer_channels,
            &mut threshold_dealer_channels,
            &mut other_server_channels,
        )?;
        println!("Server {}: Protocol execution completed", server_id);
        println!(
            "Found {} heavy hitters.",
            heavy_hitters.len(),
        );

        start_time.elapsed()
    };

    print_server_summary(
        server_id,
        protocol_time,
        &other_server_channels,
        &signal_dealer_channels,
        &check_dealer_channels,
        &threshold_dealer_channels,
    );

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    side: u8,
    #[arg(short, long)]
    config: String,
    #[arg(short, long)]
    network_config: String,
    #[arg(short, long)]
    method_config: String,
    #[arg(short, long, default_value_t = 1)]
    threads: usize,
}

fn main() {
    let args = Args::parse();
    let side = args.side;
    let config_path = args.config;
    let network_config_path = args.network_config;
    let method_config_path = args.method_config;
    let num_threads = args.threads;

    let result = if side == 0 {
        run_server(&config_path, &network_config_path, &method_config_path, false, num_threads)
    } else if side == 1 {
        run_server(&config_path, &network_config_path, &method_config_path, true, num_threads)
    } else {
        Err("Side must be 0 or 1".to_string())
    };

    if let Err(e) = result {
        eprintln!("Error running server: {}", e);
    }
}
