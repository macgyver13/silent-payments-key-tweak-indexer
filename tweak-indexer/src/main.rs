use secp256k1::{rand, PublicKey, SecretKey, Scalar, Secp256k1};
use sha2::{Sha256, Digest};

fn compute_silent_payment_key_tweak(sender_pubkey: &PublicKey, receiver_scan_pubkey: &PublicKey) -> Scalar {
    let mut hasher = Sha256::new();
    hasher.update(receiver_scan_pubkey.serialize());
    hasher.update(sender_pubkey.serialize());
    let tweak_bytes = hasher.finalize();

    Scalar::from_be_bytes(tweak_bytes.try_into().expect("32 bytes required")).expect("Tweak convert failed")
}

fn main() {
    let secp = Secp256k1::new();

    //Generate sender key pair
    let sender_privkey = SecretKey::new(&mut rand::thread_rng());
    let sender_pubkey = PublicKey::from_secret_key(&secp, &sender_privkey);
    
    //Generate receiver scan key pair
    let receiver_privkey = SecretKey::new(&mut rand::thread_rng());
    let receiver_pubkey = PublicKey::from_secret_key(&secp, &receiver_privkey);

    let tweak = compute_silent_payment_key_tweak(&sender_pubkey, &receiver_pubkey);

    let tweak_hex = hex::encode(tweak.to_be_bytes());
    println!("Tweak (Hex): {}", tweak_hex);
}

