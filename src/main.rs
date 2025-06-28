mod rs_dbc;

use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::collections::HashMap;
use rs_dbc::Dbc;

fn load_dbc(path: &str) -> Result<Dbc, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    Dbc::from_slice_lossy(&buffer).map_err(|e| format!("Failed to parse DBC file '{}': {:?}", path, e).into())
}

fn build_message_map(dbc: Dbc) -> HashMap<u32, (String, u32, String)> {
    let mut map = HashMap::new();
    for msg in dbc.messages {
        let raw_id = msg.messages_id.raw();
        let kind = msg.messages_id.kind().to_string();
        map.insert(raw_id, (kind, msg.cycle_time, msg.messages_name));
    }
    map
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dbc1 = load_dbc("1.dbc")?;
    let dbc2 = load_dbc("2.dbc")?;

    let map1 = build_message_map(dbc1);
    let map2 = build_message_map(dbc2);

    let mut output = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("output.csv")?;

    writeln!(output, "Message,Message-ID,ID-Format,DBC1,DBC2")?;

    let mut all_keys: Vec<u32> = map1.keys().chain(map2.keys()).copied().collect();
    all_keys.sort_unstable();
    all_keys.dedup();

    for id in all_keys {
        match (map1.get(&id), map2.get(&id)) {
            (Some((kind1, time1, name1)), Some((_, time2, _))) => {
                if time1 != time2 {
                    writeln!(output, "{},0x{:X},{},{},{}", name1, id, kind1, time1, time2)?;
                }
            }
            (Some((kind, time1, name)), None) => {
                writeln!(output, "{},0x{:X},{},{},", name, id, kind, time1)?;
            }
            (None, Some((kind, time2, name))) => {
                writeln!(output, "{},0x{:X},{},,{}", name, id, kind, time2)?;
            }
            _ => {}
        }
    }

    Ok(())
}
