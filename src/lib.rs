use std::str::FromStr;
use thiserror::Error;

// Custom errors for Bitcoin operations
#[derive(Error, Debug)]
pub enum BitcoinError {
    #[error("Invalid transaction format")]
    InvalidTransaction,
    #[error("Invalid script format")]
    InvalidScript,
    #[error("Invalid amount")]
    InvalidAmount,
    #[error("Parse error: {0}")]
    ParseError(String),
}

// Generic Point struct for Bitcoin addresses or coordinates
#[derive(Debug, Clone, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Point { x, y }
        // TODO: Implement constructor for Point
    }
}

// Custom serialization for Bitcoin transaction
pub trait BitcoinSerialize {
    fn serialize(&self) -> Vec<u8>;
}

// Legacy Bitcoin transaction
#[derive(Debug, Clone)]
pub struct LegacyTransaction {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl LegacyTransaction {
    pub fn builder() -> LegacyTransactionBuilder {
        LegacyTransactionBuilder::new()

        // TODO: Return a new builder for constructing a transaction
    }
}

// Transaction builder
pub struct LegacyTransactionBuilder {
    pub version: i32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub lock_time: u32,
}

impl Default for LegacyTransactionBuilder {
    fn default() -> Self {
        LegacyTransactionBuilder {
            version: 1,
            inputs: Vec::new(),
            outputs: Vec::new(),
            lock_time: 0,
        }

        // TODO: Implement default values
    }
}

impl LegacyTransactionBuilder {
    pub fn new() -> Self {
        Self::default()
        // TODO: Initialize new builder by calling default
    }

    pub fn version(mut self, version: i32) -> Self {
        self.version = version;
        self
        // TODO: Set the transaction version
    }

    pub fn add_input(mut self, input: TxInput) -> Self {
        self.inputs.push(input);
        self
        // TODO: Add input to the transaction
    }

    pub fn add_output(mut self, output: TxOutput) -> Self {
        self.outputs.push(output);
        self
        // TODO: Add output to the transaction
    }

    pub fn lock_time(mut self, lock_time: u32) -> Self {
        self.lock_time = lock_time;
        self
        // TODO: Set lock_time for transaction
    }

    pub fn build(self) -> LegacyTransaction {
        LegacyTransaction {
            version: self.version,
            inputs: self.inputs,
            outputs: self.outputs,
            lock_time: self.lock_time,
        }
        // TODO: Build and return the final LegacyTransaction
    }
}

// Transaction components
#[derive(Debug, Clone)]
pub struct TxInput {
    pub previous_output: OutPoint,
    pub script_sig: Vec<u8>,
    pub sequence: u32,
}

#[derive(Debug, Clone)]
pub struct TxOutput {
    pub value: u64, // in satoshis
    pub script_pubkey: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct OutPoint {
    pub txid: [u8; 32],
    pub vout: u32,
}

// Simple CLI argument parser
pub fn parse_cli_args(args: &[String]) -> Result<CliCommand, BitcoinError> {
    if args.is_empty() {
        return Err(BitcoinError::ParseError("No command provided".to_string()));
    }

    match args[0].as_str() {
        "send" => {
            if args.len() < 3 {
                return Err(BitcoinError::ParseError(
                    "Send command requires amount and address".to_string(),
                ));
            }

            let amount = args[1]
                .parse::<u64>()
                .map_err(|_| BitcoinError::InvalidAmount)?;

            let address = args[2].clone();

            Ok(CliCommand::Send { amount, address })
        }
        "balance" => {
            if args.len() > 1 {
                return Err(BitcoinError::ParseError(
                    "Balance command takes no arguments".to_string(),
                ));
            }
            Ok(CliCommand::Balance)
        }
        cmd => Err(BitcoinError::ParseError(format!(
            "Unknown command: {}. Use 'send' or 'balance'",
            cmd
        ))),
    }
    // TODO: Match args to "send" or "balance" commands and parse required arguments
}

pub enum CliCommand {
    Send { amount: u64, address: String },
    Balance,
}

// Decoding legacy transaction
impl TryFrom<&[u8]> for LegacyTransaction {
    type Error = BitcoinError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        if data.len() < 10 {
            return Err(BitcoinError::InvalidTransaction);
        }

        let mut offset = 0;

        // Version (4 bytes)
        let version = i32::from_le_bytes(data[offset..offset + 4].try_into().unwrap());
        offset += 4;

        // Input count and create dummy inputs
        let (input_count, size) = read_compact_size(data, offset)?;
        offset += size;
        let inputs = (0..input_count)
            .map(|_| TxInput {
                previous_output: OutPoint {
                    txid: [0; 32],
                    vout: 0,
                },
                script_sig: vec![],
                sequence: 0xffffffff,
            })
            .collect();

        // Output count and create dummy outputs
        let (output_count, size) = read_compact_size(data, offset)?;
        let outputs = (0..output_count)
            .map(|_| TxOutput {
                value: 0,
                script_pubkey: vec![],
            })
            .collect();

        // Lock time (last 4 bytes)
        let lock_time = u32::from_le_bytes(data[data.len() - 4..].try_into().unwrap());

        Ok(LegacyTransaction {
            version,
            inputs,
            outputs,
            lock_time,
        })
    }

    // TODO: Parse binary data into a LegacyTransaction
    // Minimum length is 10 bytes (4 version + 4 inputs count + 4 lock_time)
}

fn read_compact_size(data: &[u8], offset: usize) -> Result<(u64, usize), BitcoinError> {
    let first = *data.get(offset).ok_or(BitcoinError::InvalidTransaction)?;
    match first {
        0..=252 => Ok((first as u64, 1)),
        253 => Ok((
            u16::from_le_bytes(data[offset + 1..offset + 3].try_into().unwrap()) as u64,
            3,
        )),
        254 => Ok((
            u32::from_le_bytes(data[offset + 1..offset + 5].try_into().unwrap()) as u64,
            5,
        )),
        255 => Ok((
            u64::from_le_bytes(data[offset + 1..offset + 9].try_into().unwrap()),
            9,
        )),
    }
}

// Custom serialization for transaction
impl BitcoinSerialize for LegacyTransaction {
    fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        result.extend_from_slice(&self.version.to_le_bytes());
        result.extend_from_slice(&self.lock_time.to_le_bytes());

        result
        // TODO: Serialize only version and lock_time (simplified)
    }
}
