use borsh::{BorshDeserialize, BorshSerialize};
use l1x_sdk::types::{U128, U64};
use l1x_sdk::{contract, store::LookupMap};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const STORAGE_CONTRACT_KEY: &[u8] = b"cross-chain-swap-flow";
const STORAGE_EVENTS_KEY: &[u8] = b"events";
const STORAGE_STATE_KEY: &[u8] = b"state";

const STATE_1: &str = "execute_swap";
const STATE_2: &str = "finalize_swap";

const START_EVENT: &str = "SwapInitiated";
const EXECUTE_EVENT: &str = "SwapExecuted";

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Event {
    SwapInitiated(SwapInitiated),
    SwapExecuted(SwapExecuted),
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct SwapInitiated {
    in_token_address: String,
    in_amount: U128,
    source_chain: String,
    destination_chain: String,
    out_token_address: String,
    out_amount_min: U128,
    receiving_address: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct SwapExecuted {
    in_token_address: String,
    user: String,
    in_amount: U128,
    out_token_address: String,
    out_amount: U128,
    receiving_address: String,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum State {
    ExecuteSwapData(ExecuteSwapData),
    FinalizeSwapData(FinalizeSwapData),
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Payload {
    ExecuteSwap(ExecuteSwap),
    FinalizeSwap(FinalizeSwap),
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct CrossChainSwapFlow {
    events: LookupMap<String, Event>,
    state: LookupMap<String, State>,
    total_events: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct ExecuteSwap {
    data: ExecuteSwapData,
    hash: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct ExecuteSwapData {
    global_tx_id: String,
    token_address: String,
    amount: U128,
    receiving_address: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct FinalizeSwap {
    data: FinalizeSwapData,
    hash: String,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct FinalizeSwapData {
    global_tx_id: String,
    user: String,
}

impl Default for CrossChainSwapFlow {
    fn default() -> Self {
        Self {
            events: LookupMap::new(STORAGE_EVENTS_KEY.to_vec()),
            state: LookupMap::new(STORAGE_STATE_KEY.to_vec()),
            total_events: u64::default(),
        }
    }
}

#[contract]
impl CrossChainSwapFlow {
    fn load() -> Self {
        match l1x_sdk::storage_read(STORAGE_CONTRACT_KEY) {
            Some(bytes) => Self::try_from_slice(&bytes).unwrap(),
            None => panic!("The contract isn't initialized"),
        }
    }

    fn save(&mut self) {
        let encoded_contract = borsh::BorshSerialize::try_to_vec(self).unwrap();
        l1x_sdk::storage_write(STORAGE_CONTRACT_KEY, &encoded_contract);
    }

    fn to_key(global_tx_id: &String, event_type: &String) -> String {
        global_tx_id.clone() + event_type
    }

    pub fn new() {
        let mut contract = Self::default();
        contract.save();
    }

    pub fn save_event_data(
        global_tx_id: String,
        event_type: String,
        event_data: Event,
    ) {
        let mut contract = Self::load();

        let key = Self::to_key(&global_tx_id, &event_type);
        contract.events.insert(key, event_data);
        contract.total_events += 1;

        if contract.state.get(&(global_tx_id.to_owned() + STATE_1)).is_none() {
            if let Event::SwapInitiated(start) = contract
                .events
                .get(&(global_tx_id.to_owned() + START_EVENT))
                .unwrap()
            {
                let execute_swap = ExecuteSwapData {
                    global_tx_id: global_tx_id.clone(),
                    token_address: start.in_token_address.clone(),
                    amount: start.in_amount.clone(),
                    receiving_address: start.receiving_address.clone(),
                };
                contract.state.insert(
                    global_tx_id.to_owned() + STATE_1,
                    State::ExecuteSwapData(execute_swap),
                );
            } else {
                panic!("This is not an SwapInitiated variant.");
            }
        } else if contract
            .state
            .get(&(global_tx_id.to_owned() + STATE_2))
            .is_none()
        {
            if let Event::SwapExecuted(execute_event) = contract
                .events
                .get(&(global_tx_id.to_owned() + EXECUTE_EVENT))
                .unwrap()
            {
                let finalize_swap = FinalizeSwapData {
                    global_tx_id: global_tx_id.clone(),
                    user: execute_event.user.clone(),
                };
                contract.state.insert(
                    global_tx_id.to_owned() + STATE_2,
                    State::FinalizeSwapData(finalize_swap),
                );
            } else {
                panic!("This is not an SwapExecuted variant.");
            }
        } else {
            panic!("invalid global transaction id: {}", global_tx_id);
        }

        contract.save();
    }

    pub fn get_payload_to_sign(global_tx_id: String) -> Payload {
        let contract = Self::load();

        if let Some(state) =
            contract.state.get(&(global_tx_id.to_owned() + STATE_2))
        {
            if let State::FinalizeSwapData(data) = state {
                let mut hasher = Sha256::new();
                hasher.update(
                    (data.global_tx_id.clone() + &data.user).as_bytes(),
                );
                let hash_result = format!("{:X}", hasher.finalize());

                let output =
                    FinalizeSwap { data: data.clone(), hash: hash_result };

                return Payload::FinalizeSwap(output);
            }
        } else if let Some(state) =
            contract.state.get(&(global_tx_id.to_owned() + STATE_1))
        {
            if let State::ExecuteSwapData(data) = state {
                let mut hasher = Sha256::new();

                // TODO: Need Review - Quick fix to resolve build error
                // hasher.update(
                //     (data.global_tx_id.clone()
                //         + &data.token_address
                //         + &data.amount.to_string()
                //         + &data.receiving_address)
                //         .as_bytes(),
                // );

                hasher.update(data.global_tx_id.as_bytes());
                hasher.update(data.token_address.as_bytes());
                hasher.update(format!("{:#?}", data.amount).as_bytes());
                hasher.update(data.receiving_address.as_bytes());

                let hash_result = format!("{:X}", hasher.finalize());

                let output =
                    ExecuteSwap { data: data.clone(), hash: hash_result };
                return Payload::ExecuteSwap(output);
            }
        }
        panic!("invalid global transaction id: {}", global_tx_id);
    }

    pub fn total_events() -> U64 {
        let contract = Self::load();

        contract.total_events.into()
    }
}
