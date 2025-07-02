use std::str;
use std::collections;
use std::convert::TryFrom;
use regex::Regex;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    Invalid(Dbc, String),
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum MessageID {
    Standard(u16),
    Extended(u32),
}

impl MessageID {
    pub fn raw(&self) -> u32 {
        match self {
            MessageID::Standard(id) => *id as u32,
            MessageID::Extended(id) => *id | (1 << 31),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            MessageID::Standard(_) => "CAN Standard",
            MessageID::Extended(_) => "CAN Extended",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Signal {
     pub name: String,
     pub start_bit: u64,
     pub signal_size: u64,
     pub factor: f64,
     pub offset: f64,
     pub min: f64,
     pub max: f64,
     pub unit: String,
}

impl Signal {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn start_bit(&self) -> u64 {
        self.start_bit
    }

    pub fn signal_size(&self) -> u64 {
        self.signal_size
    }

    pub fn factor(&self) -> f64 {
        self.factor
    }

    pub fn offset(&self) -> f64 {
        self.offset
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub message_name: String,
    pub message_id: MessageID,
    pub message_size: u64,
    pub cycle_time: u32,
    pub transmitter: String,
    pub signals: Vec<Signal>,
}

impl Message {
    pub fn message_name(&self) -> &str {
        &self.message_name
    }

    pub fn message_id(&self) -> (u32, &'static str) {
        (self.message_id.raw(), self.message_id.kind())
    }

    pub fn message_size(&self) -> u64 {
        self.message_size
    }

    pub fn cycle_time(&self) -> u32 {
        self.cycle_time
    }

    pub fn transmitter(&self) -> &str {
        if self.transmitter.starts_with("Vector__XXX") {
            "No Transmitter"
        } else {
            &self.transmitter
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dbc {
    pub messages: Vec<Message>,
}

impl Dbc {
    pub fn from_slice(buffer: &[u8]) -> Result<Dbc, Error> {
        let dbc_input = str::from_utf8(buffer).unwrap();
        Self::try_from(dbc_input)
    }

    pub fn from_slice_lossy(buffer: &[u8]) -> Result<Dbc, Error> {
        let dbc_input = String::from_utf8_lossy(buffer);
        Self::try_from(dbc_input.as_ref())
    }
}

impl TryFrom<&str> for Dbc {
    type Error = Error;

    fn try_from(dbc_input: &str) -> Result<Self, Self::Error> {
        let messages = parse_message(dbc_input);

        if messages.is_empty() {
           return Err(Error::Invalid(Dbc { messages }, dbc_input.to_string()))
        }
        Ok(Dbc { messages })
    }
}

fn parse_message(dbc_input: &str) -> Vec<Message> {
    let message_names = parse_message_name(dbc_input);
    let message_size = parse_message_size(dbc_input);
    let message_transmitters = parse_message_transmitters(dbc_input);
    let default_cycles = parse_default_cycle_time(dbc_input).unwrap_or(0);
    let explicit_cycles = parse_explicit_cycle_time(dbc_input);
    let signals = parse_signals(dbc_input);

    let mut message = Vec::new();

    for (id, message_name) in message_names {
        let cycle_time = explicit_cycles.get(&id).copied().unwrap_or(default_cycles);
        let message_size = message_size.get(&id).copied().unwrap_or(0);
        let message_signals = signals.get(&id).cloned().unwrap_or_else(Vec::new);
        let transmitter = message_transmitters.get(&id).cloned().unwrap_or_else(|| "Vector__XXX".to_string());

        let message_id = if id < 0x800 {
            MessageID::Standard(id as u16)
        } else {
           MessageID::Extended(id)
        };

        message.push(Message {
            message_name,
            message_id,
            message_size,
            cycle_time,
            transmitter,
            signals: message_signals,
        });
    }

    message
}

fn parse_message_name(dbc_input: &str) -> collections::HashMap<u32, String> {
    let re_name = Regex::new(r#"BO_\s+(\d+)\s+(\w+):"#).unwrap();
    let mut map = collections::HashMap::new();

    for cap in re_name.captures_iter(dbc_input) {
        if let (Ok(id), Ok(name)) = (cap[1].parse::<u32>(), cap[2].parse::<String>()) {
            map.insert(id, name);
        }
    }
    map
}

fn parse_message_size(dbc_input: &str) -> collections::HashMap<u32, u64> {
    let re_size = Regex::new(r#"BO_\s+(\d+)\s+\w+:\s+(\d+)"#).unwrap();
    let mut map = collections::HashMap::new();

    for cap in re_size.captures_iter(dbc_input) {
        if let (Ok(id), Ok(size)) = (cap[1].parse::<u32>(), cap[2].parse::<u64>()) {
            map.insert(id, size);
        }
    }
    map
}

fn parse_message_transmitters(dbc_input: &str) -> collections::HashMap<u32, String> {
    let re_transmitter = Regex::new(r#"BO_\s+(\d+)\s+\w+:\s+\d+\s+(\w+)"#).unwrap();
    let mut map = collections::HashMap::new();

    for cap in re_transmitter.captures_iter(dbc_input) {
        if let Ok(id) = cap[1].parse::<u32>() {
            let transmitter = cap[2].to_string();
            map.insert(id, transmitter);
        }
    }
    map
}

fn parse_default_cycle_time(dbc_input: &str) -> Option<u32> {
    let re_default = Regex::new(r#"BA_DEF_DEF_\s+"GenMsgCycleTime"\s+(\d+);"#).unwrap();
    if let Some(cap) = re_default.captures(dbc_input) {
        return cap[1].parse::<u32>().ok();
    }
    None
}

fn parse_explicit_cycle_time(dbc_input: &str) -> collections::HashMap<u32, u32> {
    let re_explicit = Regex::new(r#"BA_ "GenMsgCycleTime" BO_ (\d+) (\d+);"#).unwrap();
    let mut map = collections::HashMap::new();

    for cap in re_explicit.captures_iter(dbc_input) {
      if let (Ok(id), Ok(cycle)) = (cap[1].parse::<u32>(), cap[2].parse::<u32>()) {
          map.insert(id, cycle);
      }
    }
    map
}

fn parse_signals(dbc_input: &str) -> collections::HashMap<u32, Vec<Signal>> {
    let re_signal = Regex::new(r#"SG_\s+(\w+)\s*:\s*(\d+)\|(\d+)@([01])([+-])\s*\(([^,]+),([^)]+)\)\s*\[([^|]+)\|([^\]]+)\]\s*"([^"]*)""#).unwrap();
    let mut signals_map: collections::HashMap<u32, Vec<Signal>> = collections::HashMap::new();
    let mut current_message_id = 0u32;
    let lines: Vec<&str> = dbc_input.lines().collect();

    for line in lines {
        if let Some(msg_cap) = Regex::new(r#"BO_\s+(\d+)\s+\w+:"#).unwrap().captures(line) {
            if let Ok(id) = msg_cap[1].parse::<u32>() {
                current_message_id = id;
                signals_map.entry(current_message_id).or_insert_with(Vec::new);
            }
        }

        if let Some(cap) = re_signal.captures(line) {
            if let (Ok(start_bit), Ok(signal_size), Ok(factor), Ok(offset), Ok(min), Ok(max)) = (
                cap[2].parse::<u64>(),
                cap[3].parse::<u64>(),
                cap[6].parse::<f64>(),
                cap[7].parse::<f64>(),
                cap[8].parse::<f64>(),
                cap[9].parse::<f64>()
            ) {
                let signal = Signal {
                    name: cap[1].to_string(),
                    start_bit,
                    signal_size,
                    factor,
                    offset,
                    min,
                    max,
                    unit: cap[10].to_string(),
                };

                if let Some(signals) = signals_map.get_mut(&current_message_id) {
                    signals.push(signal);
                }
            }
        }
    }

    signals_map
}