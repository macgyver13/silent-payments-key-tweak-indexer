use bitcoin::script::Instruction;
use secp256k1::XOnlyPublicKey;
use bitcoin::consensus::encode::deserialize_hex;
use bitcoin::block::Block;
use bitcoin::{Script, Transaction};
use silentpayments::utils::receiving;
use silentpayments::secp256k1::PublicKey;
use std::process::Command;
use std::error::Error;

use crate::database;

#[derive(Debug)]
enum ChainError {
    TxOutputNotFound,
    PubKeyFromInput,
    SegWitVersionGE2,
}
impl std::error::Error for ChainError {}

impl std::fmt::Display for ChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChainError::TxOutputNotFound => write!(f, "Could not find previous output transaction"),
            ChainError::PubKeyFromInput => write!(f, "Pub Key From Input error"),
            ChainError::SegWitVersionGE2 => write!(f, "Segwit version 2 or higher not allowed"),
        }
    }
}

pub fn get_block_count() -> Result<String, String> {
    bcli(&["getblockcount"])
}

pub fn get_block_hash(height: u32) -> Result<String, String> {
    bcli(&["getblockhash", &height.to_string()])
}

pub fn get_block(block_hash: &str) -> Result<String, String> {
    bcli(&["getblock", block_hash, "0"])
}

pub fn get_transaction(txid: &str) -> Result<String, String> {
    bcli(&["getrawtransaction", txid])
}

pub fn bcli(args: &[&str]) -> Result<String, String> {
    let result = Command::new("bitcoin-cli")
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute bitcoin-cli: {}", e))?;

    if !result.status.success() {
        return Err(format!(
            "bitcoin-cli error: {}",
            String::from_utf8_lossy(&result.stderr)
        ));
    }

    return Ok(String::from_utf8(result.stdout).unwrap().trim().to_string());
}

pub struct Chain<'a> {
    db: &'a database::Database,
    block: Option<Block>,
}

impl<'a> Chain<'a> {
    pub fn new(db: &'a database::Database) -> Self {
        Self { db, block: None }
    }

    pub fn set_block(&mut self, block: Block) {
        self.block = Some(block);
    }

    pub fn get_block(&self) -> &Block {
        self.block.as_ref().unwrap()
    }

    fn block_hash_str(&self) -> String {
        self.get_block().block_hash().to_string()
    }

    fn save_tweak_data(&self, tweak: PublicKey, tx_id: &str) -> Result<(), ChainError> {
        let block_hash_str = self.block_hash_str().clone();
        let _ = self.db.insert_tweak(&database::Tweak { 
            block_hash: block_hash_str, 
            tweak: tweak.to_string(), 
            tx_id: tx_id.to_string(), 
        });
        Ok(())
    }

    fn is_segwit_gt_v1(&self, script_pubkey: &Script) -> bool {
        let mut instructions = script_pubkey.instructions();

        if let Some(Ok(Instruction::PushBytes(version))) = instructions.next() {
            if version.len() == 1 {
                return version[0] > 1;
            }
        }
        false
    }

    // Heavy inspiration from sp-client (https://github.com/cygnet3/sp-client) and rust-silentpayments (https://github.com/cygnet3/rust-silentpayments)
    fn process_transaction(&self, transaction: &Transaction) -> Result<bool, Box<dyn Error>> {

        //Calculate input pub keys
        let mut input_pubkeys: Vec<PublicKey> = vec![];    
        for input in transaction.input.iter() {
            if input.previous_output.is_null() {
                return Ok(false);
            }
            
            let previous_tx_hex = get_transaction(&input.previous_output.txid.to_string())?;

            let previous_tx: Transaction = deserialize_hex::<Transaction>(&previous_tx_hex)?;

            let previous_script = match previous_tx.output.get(input.previous_output.vout as usize){
                Some(output) => output.script_pubkey.clone(),
                None => return Err(Box::new(ChainError::TxOutputNotFound)),
            };
            
            // Filter transactions by BIP352 consensus on allowed transactions
            if self.is_segwit_gt_v1(&previous_script) {
                return Err(Box::new(ChainError::SegWitVersionGE2));
            }

            // Collect all input pub keys
            match receiving::get_pubkey_from_input(
                &input.script_sig.to_bytes(), 
                &input.witness.to_vec(), 
                &previous_script.to_bytes(),
            ) {
                Ok(Some(pubkey)) => {
                    input_pubkeys.push(pubkey);
                    // println!("  Input {}: Previous Output: {}:{}", input_index, input.previous_output.txid, input.previous_output.vout);
                }
                Ok(None) => {

                }
                Err(_) => {
                    return Err(Box::new(ChainError::PubKeyFromInput));
                }
            }
        }

        // Get the reference to a vector of public keys for further calculations
        let pubkeys_ref: Vec<&PublicKey> = input_pubkeys.iter().collect();

        //Calculate outpoints
        let outpoints: Vec<(String, u32)> = transaction
        .input
        .iter()
        .map(|i| {
            let outpoint = i.previous_output;
            (outpoint.txid.to_string(), outpoint.vout)
        })
        .collect();

        // Calculate the tweak data based on the public keys and outpoints
        let tweak_data = receiving::calculate_tweak_data(&pubkeys_ref, &outpoints)?;

        self.save_tweak_data(tweak_data, &transaction.compute_txid().to_string())?;
        Ok(true)
    }

    /// Deserializes a block but tracks how much data was consumed
    pub fn process_transactions(&mut self, block_hex: &String) -> Result<bool, Box<dyn Error>>{
        let block = deserialize_hex::<Block>(block_hex)
            .map_err(|e| format!("Failed to decode block: {}", e))?;
        self.set_block(block.clone());
        
        // println!("Header: {:?}", block.header);
        let mut has_tweaks: bool = false;
        for (i, tx) in block.txdata.iter().enumerate() {
            // Filter transactions by BIP352 consensus on allowed transactions
            // Only process transactions with outputs that have a valid P2TR scriptpubkey
            let mut has_taproot: bool = false;
            for output in tx.output.iter() {
                if output.script_pubkey.is_p2tr() && XOnlyPublicKey::from_slice(&output.script_pubkey.as_bytes()[2..]).is_ok()  {
                    has_taproot = true;
                }
            }
            if has_taproot {
                match self.process_transaction(tx) {
                    Ok(has_tweak) => has_tweaks |= has_tweak,
                    Err(err) => {
                        eprintln!("Error processing tx: {}, block: {}: err: {}", tx.compute_txid(), block.header.block_hash(), err);
                    }
                }
            }
        }

        Ok(has_tweaks)
    }
}
