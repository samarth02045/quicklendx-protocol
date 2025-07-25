use soroban_sdk::{contracttype, Address, BytesN, Env, String, Vec};

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
        env.storage()
            .instance()
            .get(key)
            .unwrap_or_else(|| Vec::new(env))
    }
    pub fn add_bid_to_invoice(env: &Env, invoice_id: &BytesN<32>, bid_id: &BytesN<32>) {
        let mut bids = Self::get_bids_for_invoice(env, invoice_id);
        bids.push_back(bid_id.clone());
        env.storage().instance().set(invoice_id, &bids);
    }
    /// Generates a unique 32-byte bid ID using a sequential counter stored in contract storage.
    /// The counter is incremented for each new bid, and its value is stored in the last 16 bytes
    /// of the ID (big-endian, zero-padded). This guarantees uniqueness and is efficient.
    /// Note: If randomness becomes available in the SDK, consider switching to a random approach.
    pub fn generate_unique_bid_id(env: &Env) -> BytesN<32> {
        let counter_key = "unique_bid_counter";
        let mut counter: u128 = env.storage().instance().get(&counter_key).unwrap_or(0u128);
        counter += 1;
        env.storage().instance().set(&counter_key, &counter);
        // The first 16 bytes are zeros, the last 16 bytes are the counter (big-endian)
        let mut bytes = [0u8; 32];
        bytes[16..].copy_from_slice(&counter.to_be_bytes());
        BytesN::from_array(env, &bytes)
    }
}
