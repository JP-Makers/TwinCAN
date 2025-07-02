mod rs_dbc;

use std::fs::File;
use std::io::Read;
use rs_dbc::Dbc;

fn load_dbc(path: &str) -> Result<Dbc, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    Dbc::from_slice_lossy(&buffer).map_err(|e| format!("Failed to parse DBC file '{}': {:?}", path, e).into())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dbc1 = load_dbc("1.dbc")?;
    let dbc2 = load_dbc("2.dbc")?;
    
    compare_dbc_to_csv(&dbc1, &dbc2)?;
    
    Ok(())
}

fn compare_dbc_to_csv(dbc1: &Dbc, dbc2: &Dbc) -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;
    use std::io::Write;
    
    // Create a CSV file
    let mut csv_file = std::fs::File::create("dbc_comparison.csv")?;
    
    // Write CSV header
    writeln!(csv_file, "Type,Message,Signal,Field,DBC1,DBC2")?;
    
    // Create maps for quick lookup by message name
    let mut dbc1_messages: HashMap<String, &rs_dbc::Message> = HashMap::new();
    let mut dbc2_messages: HashMap<String, &rs_dbc::Message> = HashMap::new();
    
    for msg in &dbc1.messages {
        dbc1_messages.insert(msg.message_name().to_string(), msg);
    }
    
    for msg in &dbc2.messages {
        dbc2_messages.insert(msg.message_name().to_string(), msg);
    }
    
    // Get all unique message names
    let mut all_message_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    all_message_names.extend(dbc1_messages.keys().cloned());
    all_message_names.extend(dbc2_messages.keys().cloned());
    
    // Sort message names alphabetically
    let mut sorted_message_names: Vec<String> = all_message_names.into_iter().collect();
    sorted_message_names.sort();
    
    for msg_name in &sorted_message_names {
        let msg1 = dbc1_messages.get(msg_name);
        let msg2 = dbc2_messages.get(msg_name);
        
        match (msg1, msg2) {
            (Some(m1), Some(m2)) => {
                // Both DBCs have this message - compare properties
                compare_message_properties(&mut csv_file, m1, m2)?;
                compare_signals(&mut csv_file, m1, m2)?;
            },
            (Some(m1), None) => {
                // Only DBC1 has this message
                write_message_only_in_dbc(&mut csv_file, m1, "DBC1")?;
            },
            (None, Some(m2)) => {
                // Only DBC2 has this message
                write_message_only_in_dbc(&mut csv_file, m2, "DBC2")?;
            },
            (None, None) => unreachable!(),
        }
    }
    
    println!("Comparison saved to dbc_comparison.csv");
    Ok(())
}

fn compare_message_properties(
    csv_file: &mut std::fs::File, 
    msg1: &rs_dbc::Message, 
    msg2: &rs_dbc::Message
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    
    let msg_name = msg1.message_name();
    
    // Compare message size
    if msg1.message_size() != msg2.message_size() {
        writeln!(csv_file, "Message,{},, Message Size,{},{}", 
                msg_name, msg1.message_size(), msg2.message_size())?;
    }
    
    // Compare cycle time
    if msg1.cycle_time() != msg2.cycle_time() {
        writeln!(csv_file, "Message,{},, Cycle Time,{},{}", 
                msg_name, msg1.cycle_time(), msg2.cycle_time())?;
    }
    
    // Compare transmitter
    if msg1.transmitter() != msg2.transmitter() {
        writeln!(csv_file, "Message,{},, Transmitter,{},{}", 
                msg_name, msg1.transmitter(), msg2.transmitter())?;
    }
    
    // Compare message ID
    let (id1, kind1) = msg1.message_id();
    let (id2, kind2) = msg2.message_id();
    if id1 != id2 {
        writeln!(csv_file, "Message,{},, Message ID,{},{}", 
                msg_name, id1, id2)?;
    }
    
    // Compare message ID kind
    if kind1 != kind2 {
        writeln!(csv_file, "Message,{},, Message ID Kind,{},{}", 
                msg_name, kind1, kind2)?;
    }
    
    Ok(())
}

fn compare_signals(
    csv_file: &mut std::fs::File, 
    msg1: &rs_dbc::Message, 
    msg2: &rs_dbc::Message
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    use std::collections::HashMap;
    
    let msg_name = msg1.message_name();
    
    // Create maps for signal lookup
    let mut signals1: HashMap<&str, &rs_dbc::Signal> = HashMap::new();
    let mut signals2: HashMap<&str, &rs_dbc::Signal> = HashMap::new();
    
    for signal in &msg1.signals {
        signals1.insert(signal.name(), signal);
    }
    
    for signal in &msg2.signals {
        signals2.insert(signal.name(), signal);
    }
    
    // Get all unique signal names
    let mut all_signal_names: std::collections::HashSet<&str> = std::collections::HashSet::new();
    all_signal_names.extend(signals1.keys());
    all_signal_names.extend(signals2.keys());
    
    // Sort signal names alphabetically
    let mut sorted_signal_names: Vec<&str> = all_signal_names.into_iter().collect();
    sorted_signal_names.sort();
    
    for &signal_name in &sorted_signal_names {
        let sig1 = signals1.get(signal_name);
        let sig2 = signals2.get(signal_name);
        
        match (sig1, sig2) {
            (Some(s1), Some(s2)) => {
                // Both DBCs have this signal - compare properties
                compare_signal_properties(csv_file, msg_name, s1, s2)?;
            },
            (Some(s1), None) => {
                // Only DBC1 has this signal
                writeln!(csv_file, "Signal,{},{}, Exists,Yes,No", msg_name, s1.name())?;
            },
            (None, Some(s2)) => {
                // Only DBC2 has this signal
                writeln!(csv_file, "Signal,{},{}, Exists,No,Yes", msg_name, s2.name())?;
            },
            (None, None) => unreachable!(),
        }
    }
    
    Ok(())
}

fn compare_signal_properties(
    csv_file: &mut std::fs::File,
    msg_name: &str,
    sig1: &rs_dbc::Signal,
    sig2: &rs_dbc::Signal
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    
    let signal_name = sig1.name();
    
    // Compare start bit
    if sig1.start_bit() != sig2.start_bit() {
        writeln!(csv_file, "Signal,{},{}, Start Bit,{},{}", 
                msg_name, signal_name, sig1.start_bit(), sig2.start_bit())?;
    }
    
    // Compare signal size
    if sig1.signal_size() != sig2.signal_size() {
        writeln!(csv_file, "Signal,{},{}, Signal Size,{},{}", 
                msg_name, signal_name, sig1.signal_size(), sig2.signal_size())?;
    }
    
    // Compare factor
    if (sig1.factor() - sig2.factor()).abs() > f64::EPSILON {
        writeln!(csv_file, "Signal,{},{}, Factor,{},{}", 
                msg_name, signal_name, sig1.factor(), sig2.factor())?;
    }
    
    // Compare offset
    if (sig1.offset() - sig2.offset()).abs() > f64::EPSILON {
        writeln!(csv_file, "Signal,{},{}, Offset,{},{}", 
                msg_name, signal_name, sig1.offset(), sig2.offset())?;
    }
    
    // Compare min value
    if (sig1.min() - sig2.min()).abs() > f64::EPSILON {
        writeln!(csv_file, "Signal,{},{}, Min Value,{},{}", 
                msg_name, signal_name, sig1.min(), sig2.min())?;
    }
    
    // Compare max value
    if (sig1.max() - sig2.max()).abs() > f64::EPSILON {
        writeln!(csv_file, "Signal,{},{}, Max Value,{},{}", 
                msg_name, signal_name, sig1.max(), sig2.max())?;
    }
    
    // Compare unit
    if sig1.unit() != sig2.unit() {
        writeln!(csv_file, "Signal,{},{}, Unit,{},{}", 
                msg_name, signal_name, sig1.unit(), sig2.unit())?;
    }
    
    Ok(())
}

fn write_message_only_in_dbc(
    csv_file: &mut std::fs::File,
    msg: &rs_dbc::Message,
    dbc_name: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    
    let msg_name = msg.message_name();
    let (dbc1_val, dbc2_val) = if dbc_name == "DBC1" { ("Yes", "No") } else { ("No", "Yes") };
    
    writeln!(csv_file, "Message,{},, Exists,{},{}", msg_name, dbc1_val, dbc2_val)?;
    
    // Only write signals if message exists in DBC1
    if dbc_name == "DBC1" {
        for signal in &msg.signals {
            writeln!(csv_file, "Signal,{},{}, Exists,{},{}", 
                    msg_name, signal.name(), dbc1_val, dbc2_val)?;
        }
    }
    
    Ok(())
}
