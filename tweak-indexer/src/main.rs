use secp256k1::{rand, PublicKey, SecretKey, Scalar, Secp256k1};
use std::str::FromStr;
use sha2::{Sha256, Digest};
use silentpayments::{SilentPaymentAddress, Network};
use clap::Parser;

#[derive(Parser)]
#[command(long_about)]
struct Cli {
    #[arg(long)]
    send_pub: Option<String>,
    #[arg(long)]
    silent: Option<String>,
}

fn compute_silent_payment_key_tweak(sender_pubkey: &PublicKey, receiver_scan_pubkey: &PublicKey) -> Scalar {
    let mut hasher = Sha256::new();
    hasher.update(receiver_scan_pubkey.serialize());
    hasher.update(sender_pubkey.serialize());
    let tweak_bytes = hasher.finalize();

    Scalar::from_be_bytes(tweak_bytes.try_into().expect("32 bytes required")).expect("Tweak convert failed")
}

fn compute_spend_payment_address(tweak: Scalar, receiver_scan_pubkey: &PublicKey) -> PublicKey {
    let secp = Secp256k1::new();
    let tweak_point = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&tweak.to_be_bytes()).expect("Could not generate key from point"));
    
    receiver_scan_pubkey.combine(&tweak_point).expect("Failed to combine keys")
}

fn handle_inputs() -> (PublicKey, SilentPaymentAddress) {
    let secp = Secp256k1::new();
    let cli = Cli::parse();

    //Generate sender public key if not supplied
    let sender_pubkey = if let Some(pubkey_str) = cli.send_pub.as_deref() {
        PublicKey::from_str(pubkey_str).expect("invalid pub key input provided")
    } else {
        //Generate sender key pair
        println!("Generating sender public key, not provided with --send_pub argument");
        let sender_privkey = SecretKey::new(&mut rand::thread_rng());
        PublicKey::from_secret_key(&secp, &sender_privkey)
    };

    //Generate silent payment address if not supplied
    let silent_address = if let Some(silent_str) = cli.silent.as_deref() {
        SilentPaymentAddress::try_from(silent_str).expect("invalid silent address input provided")
    } else {
        println!("Generating Silent Payment Address, not provided with --silent argument");
        //Generate receiver spend key pair
        let spend_privkey = SecretKey::new(&mut rand::thread_rng());
        let spend_pubkey = PublicKey::from_secret_key(&secp, &spend_privkey);
        //Generate receiver scan key pair
        let scan_privkey = SecretKey::new(&mut rand::thread_rng());
        let scan_pubkey = PublicKey::from_secret_key(&secp, &scan_privkey);
        SilentPaymentAddress::new(scan_pubkey, spend_pubkey, Network::Mainnet, 0).unwrap()
    };

    (sender_pubkey, silent_address)
}

fn main() {

    let (sender_pubkey, silent_address) = handle_inputs();
    
    let tweak = compute_silent_payment_key_tweak(&sender_pubkey, &silent_address.get_scan_key());
    let tweak_hex = hex::encode(tweak.to_be_bytes());
    println!("Tweak (Hex): {}", tweak_hex);

    let txn_out_pubkey = compute_spend_payment_address(tweak, &silent_address.get_scan_key());
    println!("Txn Out Pub Key (Hex): {}", hex::encode(txn_out_pubkey.serialize()));

}

//https://github.com/tokio-rs/axum
//https://github.com/rusqlite/rusqlite