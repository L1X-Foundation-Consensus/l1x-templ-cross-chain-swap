use std::str::FromStr;

use borsh::{BorshDeserialize, BorshSerialize};
use ethers;
use ethers::abi::{ethabi, ParamType, Token};
use ethers::prelude::{parse_log, EthEvent};
use ethers::types::{Address, Signature};
use l1x_sdk::types::{U128, U256, U64};
use l1x_sdk::{contract, store::LookupMap};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const STORAGE_CONTRACT_KEY: &[u8; 21] = b"cross-chain-swap-flow";
const STORAGE_EVENTS_KEY: &[u8; 6] = b"events";
const STORAGE_STATE_KEY: &[u8; 8] = b"payloads";

const PAYLOAD_1: &str = "execute_swap";
const PAYLOAD_2: &str = "finalize_swap";

const INITIATE_EVENT: &str = "SwapInitiated";
const EXECUTE_EVENT: &str = "SwapExecuted";

const ETHEREUM_TOKEN_ADDRESS: &str = "0x4603e703309cd6c0b8bada1e724312242ef36ecb";
const OPTIMISIM_TOKEN_ADDRESS: &str = "0x853f409f60d477b5e4ecdff2f2094d4670afa0a1";

const OPTIMISIM_PROVIDER: &str =
    "https://optimism-goerli.infura.io/v3/904a9154641d44348e7fab88570219e9";
const ETHEREUM_PROVIDER: &str = "https://goerli.infura.io/v3/904a9154641d44348e7fab88570219e9";

const OPTIMISIM_SMART_CONTRACT_ADDRESS: &str = "0x44436A43330122a61A4877E51bA54084D5BD0aC6";
const ETHEREUM_SMART_CONTRACT_ADDRESS: &str = "0xDa4140B906044aCFb1aF3b34C94A2803D90e96aA";

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Event {
    SwapInitiated(SwapInitiatedEvent),
    SwapExecuted(ExecuteSwap),
}

#[derive(Clone, Debug, EthEvent)]
#[ethevent(name = "SwapInitiated")]
struct SwapInitiatedSolidityEvent {
    #[ethevent(indexed)]
    global_tx_id: [u8; 32],
    #[ethevent(indexed)]
    in_token_address: ethers::types::Address,
    in_amount: ethers::types::U256,
    source_chain: String,
    destination_chain: String,
    out_token_address: ethers::types::Address,
    out_amount_min: ethers::types::U256,
    receiving_address: ethers::types::Address,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct SwapInitiatedEvent {
    global_tx_id: [u8; 32],
    in_token_address: l1x_sdk::types::Address,
    in_amount: l1x_sdk::types::U256,
    source_chain: String,
    destination_chain: String,
    out_token_address: l1x_sdk::types::Address,
    out_amount_min: l1x_sdk::types::U256,
    receiving_address: l1x_sdk::types::Address,
}

#[derive(Clone, Debug, EthEvent, Serialize, Deserialize)]
#[ethevent(name = "SwapExecuted")]
pub struct SwapExecutedSolidityEvent {
    #[ethevent(indexed)]
    global_tx_id: [u8; 32],
    user: ethers::types::Address,
    token_address: ethers::types::Address,
    amount: ethers::types::U256,
    receiving_address: ethers::types::Address,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ExecuteSwap {
    global_tx_id: [u8; 32],
    user: l1x_sdk::types::Address,
    token_address: l1x_sdk::types::Address,
    amount: l1x_sdk::types::U256,
    receiving_address: l1x_sdk::types::Address,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Payload {
    ExecuteSwap(ExecuteSwap),
    FinalizeSwap(FinalizeSwapPayload),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GetPayloadResponse {
    input_data: String,
    provider: String,
    to: Address,
    from: Address,
}

#[derive(Clone, Debug, EthEvent, Serialize, Deserialize)]
#[ethevent(name = "FinalizeSwapPayload")]
pub struct FinalizeSwapSolidityPayload {
    #[ethevent(indexed)]
    global_tx_id: [u8; 32],
    user: ethers::types::Address,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Serialize, Deserialize)]
pub struct FinalizeSwapPayload {
    global_tx_id: [u8; 32],
    user: l1x_sdk::types::Address,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct CrossChainSwapFlow {
    events: LookupMap<String, Event>,
    payloads: LookupMap<String, Payload>,
    total_events: u64,
}

impl From<SwapInitiatedSolidityEvent> for SwapInitiatedEvent {
    fn from(v: SwapInitiatedSolidityEvent) -> Self {
        let mut in_amount = vec![0u8; 32];
        let mut out_amount_min = vec![0u8; 32];
        v.in_amount.to_little_endian(&mut in_amount);
        v.out_amount_min.to_little_endian(&mut out_amount_min);
        Self {
            global_tx_id: v.global_tx_id,
            in_token_address: l1x_sdk::types::Address::from(v.in_token_address.0),
            in_amount: l1x_sdk::types::U256::from_little_endian(&in_amount),
            source_chain: v.source_chain,
            destination_chain: v.destination_chain,
            out_token_address: l1x_sdk::types::Address::from(v.out_token_address.0),
            out_amount_min: l1x_sdk::types::U256::from_little_endian(&out_amount_min),
            receiving_address: l1x_sdk::types::Address::from(v.receiving_address.0),
        }
    }
}

impl From<SwapExecutedSolidityEvent> for ExecuteSwap {
    fn from(v: SwapExecutedSolidityEvent) -> Self {
        let mut amount = vec![0u8; 32];
        v.amount.to_little_endian(&mut amount);
        Self {
            global_tx_id: v.global_tx_id,
            user: l1x_sdk::types::Address::from(v.user.0),
            token_address: l1x_sdk::types::Address::from(v.token_address.0),
            amount: l1x_sdk::types::U256::from_little_endian(&amount),
            receiving_address: l1x_sdk::types::Address::from(v.receiving_address.0),
        }
    }
}

impl From<ExecuteSwap> for SwapExecutedSolidityEvent {
    fn from(v: ExecuteSwap) -> Self {
        let mut amount = vec![0u8; 32];
        v.amount.to_little_endian(&mut amount);
        Self {
            global_tx_id: v.global_tx_id,
            user: ethers::types::Address::from_slice(v.user.as_bytes()),
            amount: ethers::types::U256::from_little_endian(&amount),
            token_address: ethers::types::Address::from_slice(v.token_address.as_bytes()),
            receiving_address: ethers::types::Address::from_slice(v.receiving_address.as_bytes()),
        }
    }
}

impl From<FinalizeSwapPayload> for FinalizeSwapSolidityPayload {
    fn from(v: FinalizeSwapPayload) -> Self {
        Self {
            global_tx_id: v.global_tx_id,
            user: ethers::types::Address::from_slice(v.user.as_bytes()),
        }
    }
}

impl Default for CrossChainSwapFlow {
    fn default() -> Self {
        Self {
            events: LookupMap::new(STORAGE_EVENTS_KEY.to_vec()),
            payloads: LookupMap::new(STORAGE_STATE_KEY.to_vec()),
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

    pub fn save_event_data(global_tx_id: String, source_id: U64, event_data: String) {
        let mut contract = Self::load();
        let event_data =
            base64::decode(event_data.as_bytes()).expect("Can't decode base64 event_data");

        match source_id.0 {
            0 => {
                let log: ethers::types::Log =
                    serde_json::from_slice(&event_data).expect("Can't deserialize Log object");
                let event = parse_log::<SwapInitiatedSolidityEvent>(log)
                    .expect("Can't parse SwapInitiatedSolidityEvent");
                let key = Self::to_key(&global_tx_id, &INITIATE_EVENT.to_string());

                let event_data: SwapInitiatedEvent = event.clone().into();
                contract
                    .events
                    .insert(key, Event::SwapInitiated(event_data.clone()));

                let execute_swap = ExecuteSwap {
                    global_tx_id: event_data.global_tx_id,
                    user: event_data.receiving_address,
                    token_address: event_data.in_token_address,
                    amount: event_data.out_amount_min,
                    receiving_address: event_data.receiving_address,
                };
                contract.payloads.insert(
                    global_tx_id.to_owned() + PAYLOAD_1,
                    Payload::ExecuteSwap(execute_swap),
                );

                contract.total_events += 1;
            }
            1 => {
                let log: ethers::types::Log =
                    serde_json::from_slice(&event_data).expect("Can't deserialize Log object");
                let event: ExecuteSwap = parse_log::<SwapExecutedSolidityEvent>(log)
                    .expect("Can't parse SwapExecutedSolidityEvent")
                    .into();
                let key = Self::to_key(&global_tx_id, &EXECUTE_EVENT.to_string());

                let event_data: ExecuteSwap = event.clone().into();
                contract
                    .events
                    .insert(key, Event::SwapExecuted(event_data.clone()));

                let finalize_swap = FinalizeSwapPayload {
                    global_tx_id: event_data.global_tx_id,
                    user: event_data.user.clone(),
                };
                contract.payloads.insert(
                    global_tx_id.to_owned() + PAYLOAD_2,
                    Payload::FinalizeSwap(finalize_swap),
                );
                contract.total_events += 1;
            }
            _ => panic!("Unknown source id: {}", source_id.0),
        };

        contract.save();
    }

    pub fn get_payload_hash_to_sign(global_tx_id: String) -> String {
        let contract = Self::load();

        if let Some(payloads) = contract
            .payloads
            .get(&(global_tx_id.to_owned() + PAYLOAD_2))
        {
            if let Payload::FinalizeSwap(data) = payloads {

                //return PayloadResponse::FinalizeSwap(data.clone().into());
            }
        } else if let Some(payloads) = contract
            .payloads
            .get(&(global_tx_id.to_owned() + PAYLOAD_1))
        {
            if let Payload::ExecuteSwap(data) = payloads {
                let payload: SwapExecutedSolidityEvent = data.clone().into();
                let string_payload_user = format!("{:X}", payload.user);
                let string_payload_token_address = format!("{:X}", payload.token_address);
                let string_payload_amount = format!("{:X}", payload.amount);
                let string_payload_receiving_address = format!("{:X}", payload.receiving_address);

                let data = &(Self::bytes_to_hex_string(&payload.global_tx_id)
                    + &string_payload_user.to_ascii_lowercase()
                    + &string_payload_token_address.to_ascii_lowercase()
                    + &Self::zero_pad_string(&string_payload_amount.to_ascii_lowercase())
                    + &string_payload_receiving_address.to_ascii_lowercase())[2..];

                let payload_hash = ethers::utils::keccak256(hex::decode(data).unwrap());
                return hex::encode(payload_hash);
            }
        }
        panic!("invalid global transaction id: {}", global_tx_id);
    }

    pub fn get_pay_load(global_tx_id: String, signature: String) -> GetPayloadResponse {
        let contract = Self::load();
        let signature: Signature = Signature::from_str(&signature).unwrap();

        if let Some(payloads) = contract
            .payloads
            .get(&(global_tx_id.to_owned() + PAYLOAD_2))
        {
            if let Payload::FinalizeSwap(data) = payloads {
                //return PayloadResponse::FinalizeSwap(data.clone().into());
            }
        } else if let Some(payloads) = contract
            .payloads
            .get(&(global_tx_id.to_owned() + PAYLOAD_1))
        {
            if let Payload::ExecuteSwap(data) = payloads {
                let payload: SwapExecutedSolidityEvent = data.clone().into();
                let function_selector = hex::encode(ethabi::short_signature(
                    "executeSwap",
                    &[
                        ParamType::Tuple(vec![
                            ParamType::FixedBytes(32),
                            ParamType::Address,
                            ParamType::Address,
                            ParamType::Uint(256),
                            ParamType::Address,
                        ]),
                        ParamType::Bytes,
                    ],
                ));

                // Construct the transaction data for encoding
                let transaction_data = vec![
                    Token::FixedBytes(payload.global_tx_id.to_vec()),
                    Token::Address(payload.user),
                    Token::Address(payload.token_address),
                    Token::Uint(payload.amount),
                    Token::Address(payload.receiving_address),
                    Token::Bytes(signature.into()),
                ];

                // Encode the transaction data into bytes
                let encoded_transaction_data = ethabi::encode(&transaction_data);
                let data_without_function_signature = hex::encode(&encoded_transaction_data);
                let data = function_selector.to_owned() + &data_without_function_signature;
                let mut _provider = ETHEREUM_PROVIDER;
                let mut _to = ETHEREUM_SMART_CONTRACT_ADDRESS;

                if payload.token_address.to_string().to_ascii_lowercase()
                    == OPTIMISIM_TOKEN_ADDRESS
                {
                    _provider = OPTIMISIM_PROVIDER;
                    _to = OPTIMISIM_SMART_CONTRACT_ADDRESS;
                } else {

                }
                return GetPayloadResponse {
                    input_data: data,
                    provider: _provider.to_string(),
                    to: _to.to_string().parse::<Address>().unwrap(),
                    from: "0xc31beb2a223435a38141Ee15C157672A9fA2997D"
                        .parse::<Address>()
                        .unwrap(),
                };
            }
        }
        panic!("invalid global transaction id: {}", global_tx_id);
    }

    fn bytes_to_hex_string(bytes: &[u8]) -> String {
        let hex_chars: Vec<String> = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        let hex_string = hex_chars.join("");
        format!("0x{:0<64}", hex_string)
    }

    fn zero_pad_string(input: &str) -> String {
        let input_len = input.len();
        if input_len >= 64 {
            return input.to_string();
        }

        let zero_padding = "0".repeat(64 - input_len);
        let zero_padded_string = format!("{}{}", zero_padding, input);

        zero_padded_string
    }

    pub fn total_events() -> U64 {
        let contract = Self::load();

        contract.total_events.into()
    }
}
