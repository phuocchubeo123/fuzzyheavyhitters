use clap::Parser;
use mosaic::{
    channel::{setup_parallel_channels, CommTrackingChannel},
    configs::{cli_config::CliConfig, network_config::NetworkConfig},
    fuzzy_match::dealer::{CheckKeyMode, FssDealer}, util::get_distance_threshold,
};


struct NetworkDealerSetup {
    signal_channels_server0: Vec<CommTrackingChannel>,
    signal_channels_server1: Vec<CommTrackingChannel>,
}

fn setup_network_dealer(
    network_config_path: &str,
    num_channels: usize,
) -> Result<NetworkDealerSetup, String> {
    let network_config = NetworkConfig::from_file(network_config_path)?;

    println!(
        "Setting up {} parallel dealer channels to each server...",
        num_channels
    );

    let signal_channels_server0 = setup_parallel_channels(
        true,
        num_channels,
        &network_config.server0_addr,
        network_config.dealer_to_server0_port,
    )?;

    let signal_channels_server1 = setup_parallel_channels(
        true,
        num_channels,
        &network_config.server1_addr,
        network_config.dealer_to_server1_port,
    )?;

    println!(
        "Connected to both servers with {} channels each",
        num_channels
    );

    Ok(NetworkDealerSetup {
        signal_channels_server0,
        signal_channels_server1,
    })
}


fn print_dealer_summary(
    num_channels: usize,
    dealer_time: std::time::Duration,
    total_time: std::time::Duration,
    signal_channels_server0: &[CommTrackingChannel],
    signal_channels_server1: &[CommTrackingChannel],
) {
    let mut total_bytes_sent_0 = 0;
    let mut total_bytes_received_0 = 0;
    let mut total_bytes_sent_1 = 0;
    let mut total_bytes_received_1 = 0;

    for channel in signal_channels_server0 {
        let (sent, received) = channel.get_communication_stats();
        total_bytes_sent_0 += sent;
        total_bytes_received_0 += received;
    }

    for channel in signal_channels_server1 {
        let (sent, received) = channel.get_communication_stats();
        total_bytes_sent_1 += sent;
        total_bytes_received_1 += received;
    }


    let total_bytes =
        total_bytes_sent_0 + total_bytes_received_0 + total_bytes_sent_1 + total_bytes_received_1;

    println!("\n=== Dealer Performance Summary ===");
    println!(
        "📊 FSS key generation and distribution time: {:.2?}",
        dealer_time
    );
    println!("📊 Total dealer time: {:.2?}", total_time);
    println!(
        "📡 Communication with server 0 ({} channels):",
        num_channels
    );
    println!(
        "   Bytes sent: {} bytes ({:.2} KB)",
        total_bytes_sent_0,
        total_bytes_sent_0 as f64 / 1024.0
    );
    println!(
        "   Bytes received: {} bytes ({:.2} KB)",
        total_bytes_received_0,
        total_bytes_received_0 as f64 / 1024.0
    );
    println!(
        "📡 Communication with server 1 ({} channels):",
        num_channels
    );
    println!(
        "   Bytes sent: {} bytes ({:.2} KB)",
        total_bytes_sent_1,
        total_bytes_sent_1 as f64 / 1024.0
    );
    println!(
        "   Bytes received: {} bytes ({:.2} KB)",
        total_bytes_received_1,
        total_bytes_received_1 as f64 / 1024.0
    );
    println!(
        "📡 Total communication: {} bytes ({:.2} KB)",
        total_bytes,
        total_bytes as f64 / 1024.0
    );
}
/// Run as dealer - generates and distributes FSS keys to servers
fn run_dealer(config_path: &str, network_config_path: &str, num_threads: usize) -> Result<(), String> {
    if num_threads == 0 {
        return Err("--threads must be greater than 0".to_string());
    }

    rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
        .map_err(|e| format!("Failed to configure Rayon thread pool: {}", e))?;

    let start_time = std::time::Instant::now();
    println!("Starting FSS Dealer...");
    let cli_config = CliConfig::from_file(config_path)?;

    let distance_threshold = get_distance_threshold(
        cli_config.protocol.delta,
        &cli_config.protocol.distance_metric,
    );

    let dealer = FssDealer::new(
        distance_threshold,
        cli_config.protocol.match_threshold,
        cli_config.protocol.h2,
        cli_config.protocol.h3,
        cli_config.protocol.num_clients,
        cli_config.protocol.d,
    );

    let check_key_mode = match cli_config.protocol.check_property.as_str() {
        "Equality" => CheckKeyMode::Equality,
        "MuBounded" => CheckKeyMode::MuBounded,
        other => return Err(format!("Unsupported check property: {}", other)),
    };

    // Determine number of parallel channels (use specified num_threads or system parallelism)
    let num_channels = num_threads;

    let mut network = setup_network_dealer(network_config_path, num_channels)?;

    let dealer_start = std::time::Instant::now();
    dealer
        .run_dealer_parallel(
            &mut network.signal_channels_server0,
            &mut network.signal_channels_server1,
            check_key_mode,
        )
        .map_err(|e| format!("Failed to run parallel dealer protocol: {}", e))?;
    let dealer_time = dealer_start.elapsed();

    let total_time = start_time.elapsed();

    print_dealer_summary(
        num_channels,
        dealer_time,
        total_time,
        &network.signal_channels_server0,
        &network.signal_channels_server1,
    );

    Ok(())
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    config: String,
    #[arg(short, long)]
    network_config: String,
    #[arg(short, long)]
    threads: usize,
}

fn main() {
    let args = Args::parse();
    let config_path = &args.config;
    let network_config_path = &args.network_config;
    let num_threads = args.threads;

    let result = run_dealer(config_path, network_config_path, num_threads);

    if let Err(e) = result {
        eprintln!("Error running dealer: {}", e);
    }
}
