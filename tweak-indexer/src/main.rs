use std::process::exit;
use clap::Parser;


mod chain;
mod database;

#[derive(Parser)]
#[command(long_about)]
struct Cli {
    #[arg(long)]
    start_height: Option<u32>,
    #[arg(long)]
    end_height: Option<u32>,
}



fn handle_inputs() -> (u32, u32) {

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
        //Current height - 1
        429147 //883312
    };

    let end_height = if let Some(height) = cli.end_height {
        height
    } else {
        //Current height - 1
        start_height + 1000
    };

    (start_height, end_height)
}

fn main() {
    let (start_height, end_height) = handle_inputs();
    let mut current_block = start_height;

    let db = match database::Database::new("blocks.db") {
        Ok(db) => db,
        Err(err) => {
            eprintln!("Not able to open database: {}", err);
            exit(1);
        }
    };

    let mut chain = chain::Chain::new(&db);
    while current_block <= end_height {
        let block_hash = match chain::get_block_hash(current_block) {
            Ok(block_hash_str) => block_hash_str,
            Err(err) => {
                eprintln!("Error fetching block hash: {}", err);
                exit(1);
            }
        };

        // check if the block has been handled
        if db.get_block(&block_hash).is_ok_and(|x| x.len() > 0) {
            println!("******** Already processed block hash {}, height: {} ********", block_hash, current_block);
            current_block += 1;
            continue;
        }
        
        println!("Block Hash {}, height: {}", block_hash, current_block);

        let block_hex = match chain::get_block(&block_hash) {
            Ok(block_str) => block_str,
            Err(err) => {
                eprintln!("Error fetching block: {}", err);
                exit(1);
            }
        };

        match chain.process_transactions(&block_hex) {
            Ok(has_tweaks) => {
                println!("Store block handled: {}",has_tweaks);
                db.insert_block(&block_hash, has_tweaks);
            },
            Err(err) => eprintln!("Not storing block: {}", err)
        }
        current_block += 1;
    }
    db.close();
    

}

//https://github.com/tokio-rs/axum
//https://github.com/rusqlite/rusqlite