use secp256k1::{rand, PublicKey, SecretKey, Scalar, Secp256k1};
use sha2::{Sha256, Digest};
use silentpayments::{SilentPaymentAddress, Network};

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

fn main() {
    let secp = Secp256k1::new();

    //Generate receiver scan key pair
    let scan_privkey = SecretKey::new(&mut rand::thread_rng());
    let scan_pubkey = PublicKey::from_secret_key(&secp, &scan_privkey);

    //Generate receiver spend key pair
    let spend_privkey = SecretKey::new(&mut rand::thread_rng());
    let spend_pubkey = PublicKey::from_secret_key(&secp, &spend_privkey);

    //Generate silent payment address
    let silent_address = SilentPaymentAddress::new(scan_pubkey, spend_pubkey, Network::Mainnet, 0).unwrap();
    println!("Silent Payment Address (Hex): {}", silent_address.to_string());

    //Generate sender key pair
    let sender_privkey = SecretKey::new(&mut rand::thread_rng());
    let sender_pubkey = PublicKey::from_secret_key(&secp, &sender_privkey);
    
    let tweak = compute_silent_payment_key_tweak(&sender_pubkey, &silent_address.get_scan_key());
    let tweak_hex = hex::encode(tweak.to_be_bytes());
    println!("Tweak (Hex): {}", tweak_hex);

    let txn_out_pubkey = compute_spend_payment_address(tweak, &silent_address.get_scan_key());
    println!("Txn Out Pub Key (Hex): {}", hex::encode(txn_out_pubkey.serialize()));

}

//https://github.com/tokio-rs/axum
//https://github.com/rusqlite/rusqlite