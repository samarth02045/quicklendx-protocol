use soroban_sdk::{contracttype, Address, BytesN, Env, String, Vec};
use soroban_sdk::prng;

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BidStatus {
    Placed,
    Withdrawn,
    Accepted,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Bid {
    pub bid_id: BytesN<32>,
    pub invoice_id: BytesN<32>,
    pub investor: Address,
    pub bid_amount: i128,
    pub expected_return: i128,
    pub timestamp: u64,
    pub status: BidStatus,
}

pub struct BidStorage;

impl BidStorage {
    pub fn store_bid(env: &Env, bid: &Bid) {
        env.storage().instance().set(&bid.bid_id, bid);
    }
    pub fn get_bid(env: &Env, bid_id: &BytesN<32>) -> Option<Bid> {
        env.storage().instance().get(bid_id)
    }
    pub fn update_bid(env: &Env, bid: &Bid) {
        env.storage().instance().set(&bid.bid_id, bid);
    }
    pub fn get_bids_for_invoice(env: &Env, invoice_id: &BytesN<32>) -> Vec<BytesN<32>> {
        let key = invoice_id;
        env.storage().instance().get(key).unwrap_or_else(|| Vec::new(env))
    }
    pub fn add_bid_to_invoice(env: &Env, invoice_id: &BytesN<32>, bid_id: &BytesN<32>) {
        let mut bids = Self::get_bids_for_invoice(env, invoice_id);
        bids.push_back(bid_id.clone());
        env.storage().instance().set(invoice_id, &bids);
    }
    /// Generate a unique bid ID using Soroban's PRNG, retrying if a collision is found
    pub fn generate_unique_bid_id(env: &Env) -> BytesN<32> {
        let mut prng = prng::Prng::new(env);
        loop {
            let mut random_bytes = [0u8; 32];
            let prng_bytes = prng.bytes(32);
            for (i, b) in prng_bytes.iter().enumerate() {
                random_bytes[i] = *b;
            }
            let candidate = BytesN::from_array(env, &random_bytes);
            if Self::get_bid(env, &candidate).is_none() {
                return candidate;
            }
            // else, collision: try again
        }
    }
} 