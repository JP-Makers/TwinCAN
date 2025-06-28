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
pub struct Messages {
    pub messages_name: String,
    pub messages_id: MessageID,
    pub cycle_time: u32,
}

impl Messages {
    pub fn cycle_time(&self) -> u32 {
        self.cycle_time
    }

    pub fn messages_id(&self) -> (u32, &'static str) {
        (self.messages_id.raw(), self.messages_id.kind())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dbc {
    pub messages: Vec<Messages>,
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
        let messages = parse_messages(dbc_input);

        if messages.is_empty() {
           return Err(Error::Invalid(Dbc { messages }, dbc_input.to_string()))
        }
        Ok(Dbc { messages })
    }
}

fn parse_messages(dbc_input: &str) -> Vec<Messages> {
    let default_cycles = default_cycle_time(dbc_input).unwrap_or(0);
    let explicit_cycles = explicit_cycle_time(dbc_input);
    
    let re_names = Regex::new(r#"BO_\s+(\d+)\s+(\w+):"#).unwrap();
    let mut messages = Vec::new();

    for cap in re_names.captures_iter(dbc_input) {
        let id = cap[1].parse::<u32>().unwrap();
        let messages_name = cap[2].to_string();
        let cycle_time = explicit_cycles.get(&id).copied().unwrap_or(default_cycles);

        let messages_id = if id < 0x800 {
            MessageID::Standard(id as u16)
        } else {
            MessageID::Extended(id)
        };

        messages.push(Messages {
            messages_id,
            cycle_time,
            messages_name,
        });
    }

    messages
}

fn default_cycle_time(dbc_input: &str) -> Option<u32> {
    let re_default = Regex::new(r#"BA_DEF_DEF_\s+"GenMsgCycleTime"\s+(\d+);"#).unwrap();
    if let Some(cap) = re_default.captures(dbc_input) {
        return cap[1].parse::<u32>().ok();
    }
    None
}

fn explicit_cycle_time(dbc_input: &str) -> collections::HashMap<u32, u32> {
    let re_explicit = Regex::new(r#"BA_ "GenMsgCycleTime" BO_ (\d+) (\d+);"#).unwrap();
    let mut map = collections::HashMap::new();

    for cap in re_explicit.captures_iter(dbc_input) {
      if let (Ok(id), Ok(cycle)) = (cap[1].parse::<u32>(), cap[2].parse::<u32>()) {
          map.insert(id, cycle);
      }
    }
    map
}
