use std::{process::exit, thread::sleep, time::Duration};
use clap::Parser;
use database::Database;
use tracing::{error, info, warn, Level};
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, EnvFilter, Layer, Registry};
use tracing_appender::rolling;

mod chain;
mod database;

#[derive(Parser)]
#[command(long_about)]
struct Cli {
    #[arg(long)]
    start_height: Option<u32>,
    #[arg(long)]
    end_height: Option<u32>,
    #[arg(long)]
    blocks: Option<u32>,
}

struct StartupParams {
    start_height: u32,
    end_height: u32,
    continuous_index: bool,
    db_path: String,
}

fn setup_logging() {
    // Create a rolling file appender (daily logs)
    let file_appender = rolling::daily("logs", "debug.log");

    // let file_appender = File::create("debug.log");
    // let file_appender = match file_appender  {Ok(file) => file,Err(error) => panic!("Error: {:?}",error),};
    // let debug_log = fmt::layer()
        // .with_writer(Arc::new(file_appender));

    // Console log layer
    let stdout_layer = fmt::layer()
        .pretty() // Makes console logs readable
        .with_filter(EnvFilter::from_default_env()); // Uses RUST_LOG

    // File layer for warnings & errors only
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_filter(filter::LevelFilter::from_level(Level::INFO)); // Only log warn & error

    // Combine both layers into a subscriber
    let subscriber = Registry::default()
        .with(stdout_layer)
        .with(file_layer);

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global subscriber");
}

fn auto_index(db: &Database) -> (u32, u32) {

    let starting_block= db.get_highest_block().map_or_else(
        |err| {
            error!("Failed to fetch highest block: {}", err);
            exit(1);
        },
        |highest_block| if highest_block > 0 { highest_block } else { 709632 }, //Default to first Taproot block
    );

    let mut last_block = match chain::get_block_count() {
        Ok(block_count) => block_count.parse().expect("Failed to parse current block count"),
        Err(err) => {
            error!("Error fetching block count: {}", err);
            exit(1);
        }
    };

    if last_block < starting_block {
        last_block = starting_block
    }

    (starting_block, last_block)
}

fn handle_inputs() -> StartupParams {

    let cli = Cli::parse();

    // let silent_address = if let Some(silent_str) = cli.silent.as_deref() {
    //     SilentPaymentAddress::try_from(silent_str).expect("invalid silent address input provided")
    // } else {
    //     println!("Generating Silent Payment Address, not provided with --silent argument");
    //     //Generate receiver spend key pair
    //     let spend_privkey = SecretKey::new(&mut rand::thread_rng());
    //     let spend_pubkey = PublicKey::from_secret_key(&secp, &spend_privkey);
    //     println!("Spend Pub Key (Hex): {}", hex::encode(spend_pubkey.serialize()));
    //     //Generate receiver scan key pair
    //     let scan_privkey = SecretKey::new(&mut rand::thread_rng());
    //     let scan_pubkey = PublicKey::from_secret_key(&secp, &scan_privkey);
    //     println!("Scan Pub Key (Hex): {}", hex::encode(scan_pubkey.serialize()));
    //     SilentPaymentAddress::new(scan_pubkey, spend_pubkey, Network::Mainnet, 0).unwrap()
    // };
    let start_height = if let Some(height) = cli.start_height {
        height
    } else {
        0
    };

    let end_height = if let Some(height) = cli.end_height {
        height
    } else {
        let block_count = if let Some(count) = cli.blocks {
            count
        } else {
            10
        };
        start_height + block_count
    };

    StartupParams{ 
        start_height: start_height, 
        end_height: end_height, 
        continuous_index: start_height == 0, 
        db_path: String::from("blocks.db"),
    }
}

fn index_blocks(startup: StartupParams) {

    let db = match database::Database::new(&startup.db_path) {
        Ok(db) => db,
        Err(err) => {
            error!("Not able to open database: {}", err);
            exit(1);
        }
    };

    let mut current_block = startup.start_height;
    let mut last_block = startup.end_height;
    
    loop {
        // determine next block based on last block processed in db
        if startup.continuous_index {
            (current_block, last_block) = auto_index(&db);
        }

        let mut chain = chain::Chain::new(&db);
        while current_block <= last_block {
            let block_hash = match chain::get_block_hash(current_block) {
                Ok(block_hash_str) => block_hash_str,
                Err(err) => {
                    if err.contains("height out of range") {
                        info!("At current block height");
                        break;
                    } else {
                        error!("Error fetching block hash: {}", err);
                        exit(1);
                    }
                }
            };

            // check if the block has been handled
            if db.get_block(&block_hash).is_ok_and(|x| x.len() > 0) {
                info!("******** Already processed block hash {}, height: {} ********", block_hash, current_block);
                current_block += 1;
                continue;
            }
            
            info!("Block Hash {}, height: {}", block_hash, current_block);

            let block_hex = match chain::get_block(&block_hash) {
                Ok(block_str) => block_str,
                Err(err) => {
                    error!("Error fetching block: {}", err);
                    exit(1);
                }
            };

            match chain::get_block_input_transactions(&block_hash) {
                Ok(prev_scripts) => chain.set_previous_scripts(prev_scripts),
                Err(err) => {
                    error!("Error fetching prev out scripts: {}", err);
                    exit(1);
                }
            }
            match chain.process_transactions(&block_hex) {
                Ok(has_tweaks) => {
                    let _ = db.insert_block(&database::Block { 
                        height: current_block, 
                        hash: block_hash, 
                        has_tweaks: has_tweaks 
                    });
                },
                Err(err) => warn!("Not storing block: {}", err)
            }
            current_block += 1;
        }

        if startup.continuous_index {
            info!("Sleeping for 5 minutes, then try again");
            sleep(Duration::from_secs(300));
        } else {
            db.close();
            return;
        }
    }

}

fn main() {
    setup_logging();
    index_blocks( handle_inputs());
}

#[cfg(test)]
mod tests {
    use crate::database;
    use crate::chain::{Chain,get_block_with_input};

    #[test]
    fn test_process_transactions() {
        let db = database::Database::new(":memory:").unwrap();
        let mut chain = Chain::new(&db);

        let block_hash = "0000000000000000000149ba526848af34e4dbed814a85859753fadf5594e226";

        let block_hex = match get_block_with_input(&block_hash) {
            Ok(block_str) => block_str,
            Err(err) => {
                err
            }
        };

        println!("json: {:?}",block_hex);
        let has_tweaks = chain.process_transactions(&block_hex).unwrap();
    }
}