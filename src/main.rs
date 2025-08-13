#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod rs_dbc;

use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::collections::{HashMap, HashSet};
use rs_dbc::Dbc;
use rfd::FileDialog;
use slint::{ComponentHandle, VecModel, ModelRc, Model};

slint::include_modules!();

#[derive(Clone, Debug)]
struct ComparisonResult {
    result_type: String,
    message: String,
    signal: String,
    field: String,
    dbc1: String,
    dbc2: String,
}

impl From<ComparisonResult> for ComparisonResultItem {
    fn from(result: ComparisonResult) -> Self {
        ComparisonResultItem {
            r#type: result.result_type.into(),
            message: result.message.into(),
            signal: result.signal.into(),
            field: result.field.into(),
            dbc1: result.dbc1.into(),
            dbc2: result.dbc2.into(),
        }
    }
}

fn load_dbc(path: &str) -> Result<Dbc, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    Dbc::from_slice_lossy(&buffer).map_err(|e| format!("Failed to parse DBC file '{}': {:?}", path, e).into())
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;

    let ui_weak = ui.as_weak();
    ui.on_select_dbc1_file(move || {
        let ui = ui_weak.unwrap();
        if let Some(path) = FileDialog::new()
            .add_filter("DBC files", &["dbc"])
            .pick_file()
            {
                ui.set_dbc1_path(path.to_string_lossy().to_string().into());
            }
    });

    let ui_weak = ui.as_weak();
    ui.on_select_dbc2_file(move || {
        let ui = ui_weak.unwrap();
        if let Some(path) = FileDialog::new()
            .add_filter("DBC files", &["dbc"])
            .pick_file()
            {
                ui.set_dbc2_path(path.to_string_lossy().to_string().into());
            }
    });

    let ui_weak = ui.as_weak();
    ui.on_compare_files(move || {
        let ui = ui_weak.unwrap();
        let dbc1_path = ui.get_dbc1_path().to_string();
        let dbc2_path = ui.get_dbc2_path().to_string();

        ui.set_status("Comparing files...".into());

        match (load_dbc(&dbc1_path), load_dbc(&dbc2_path)) {
            (Ok(dbc1), Ok(dbc2)) => {
                match compare_dbc_files(&dbc1, &dbc2) {
                    Ok(results) => {
                        let slint_results: Vec<ComparisonResultItem> = results
                        .into_iter()
                        .map(Into::into)
                        .collect();
                        let model = ModelRc::new(VecModel::from(slint_results)); // ✅ FIXED
                        ui.set_comparison_results(model);
                        ui.set_status(format!(
                            "Comparison complete. Found {} differences.",
                            ui.get_comparison_results().row_count() // ✅ FIXED
                        ).into());
                    }
                    Err(e) => {
                        ui.set_status(format!("Error during comparison: {}", e).into());
                    }
                }
            }
            (Err(e), _) => ui.set_status(format!("Error loading DBC1: {}", e).into()),
                        (_, Err(e)) => ui.set_status(format!("Error loading DBC2: {}", e).into()),
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_export_csv(move || {
        let ui = ui_weak.unwrap();
        if let Some(path) = FileDialog::new()
            .add_filter("CSV files", &["csv"])
            .set_file_name("dbc_comparison.csv")
            .save_file()
            {
                let dbc1_path = ui.get_dbc1_path().to_string();
                let dbc2_path = ui.get_dbc2_path().to_string();

                match (load_dbc(&dbc1_path), load_dbc(&dbc2_path)) {
                    (Ok(dbc1), Ok(dbc2)) => {
                        match export_comparison_to_csv(&dbc1, &dbc2, &path.to_string_lossy()) {
                            Ok(_) => {
                                ui.set_status(format!("CSV exported to: {}", path.to_string_lossy()).into());
                            }
                            Err(e) => {
                                ui.set_status(format!("Error exporting CSV: {}", e).into());
                            }
                        }
                    }
                    (Err(e), _) => ui.set_status(format!("Error loading DBC1: {}", e).into()),
                     (_, Err(e)) => ui.set_status(format!("Error loading DBC2: {}", e).into()),
                }
            }
    });

    ui.run()
}

fn compare_dbc_files(dbc1: &Dbc, dbc2: &Dbc) -> Result<Vec<ComparisonResult>, Box<dyn Error>> {
    
    let mut results = Vec::new();
    
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
    let mut all_message_names: HashSet<String> = HashSet::new();
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
                compare_message_properties_for_results(&mut results, m1, m2);
                compare_signals_for_results(&mut results, m1, m2);
            },
            (Some(m1), None) => {
                // Only DBC1 has this message
                results.push(ComparisonResult {
                    result_type: "Message".to_string(),
                    message: m1.message_name().to_string(),
                    signal: "".to_string(),
                    field: "Exists".to_string(),
                    dbc1: "Yes".to_string(),
                    dbc2: "No".to_string(),
                });
            },
            (None, Some(m2)) => {
                // Only DBC2 has this message
                results.push(ComparisonResult {
                    result_type: "Message".to_string(),
                    message: m2.message_name().to_string(),
                    signal: "".to_string(),
                    field: "Exists".to_string(),
                    dbc1: "No".to_string(),
                    dbc2: "Yes".to_string(),
                });
            },
            (None, None) => unreachable!(),
        }
    }
    
    Ok(results)
}

fn compare_message_properties_for_results(
    results: &mut Vec<ComparisonResult>,
    msg1: &rs_dbc::Message, 
    msg2: &rs_dbc::Message
) {
    let msg_name = msg1.message_name();
    
    // Compare message size
    if msg1.message_size() != msg2.message_size() {
        results.push(ComparisonResult {
            result_type: "Message".to_string(),
            message: msg_name.to_string(),
            signal: "".to_string(),
            field: "DLC".to_string(),
            dbc1: msg1.message_size().to_string(),
            dbc2: msg2.message_size().to_string(),
        });
    }
    
    // Compare cycle time
    if msg1.cycle_time() != msg2.cycle_time() {
        results.push(ComparisonResult {
            result_type: "Message".to_string(),
            message: msg_name.to_string(),
            signal: "".to_string(),
            field: "Cycle Time".to_string(),
            dbc1: msg1.cycle_time().to_string(),
            dbc2: msg2.cycle_time().to_string(),
        });
    }
    
    // Compare transmitter
    if msg1.transmitter() != msg2.transmitter() {
        results.push(ComparisonResult {
            result_type: "Message".to_string(),
            message: msg_name.to_string(),
            signal: "".to_string(),
            field: "Transmitter".to_string(),
            dbc1: msg1.transmitter().to_string(),
            dbc2: msg2.transmitter().to_string(),
        });
    }
    
    // Compare message ID
    let (id1, kind1) = msg1.message_id();
    let (id2, kind2) = msg2.message_id();
    if id1 != id2 {
        results.push(ComparisonResult {
            result_type: "Message".to_string(),
            message: msg_name.to_string(),
            signal: "".to_string(),
            field: "Message ID".to_string(),
            dbc1: format!("0x{:X}", id1),
            dbc2: format!("0x{:X}", id2),
        });
    }
    
    // Compare message ID kind
    if kind1 != kind2 {
        results.push(ComparisonResult {
            result_type: "Message".to_string(),
            message: msg_name.to_string(),
            signal: "".to_string(),
            field: "ID Format".to_string(),
            dbc1: format!("{:?}", kind1),
            dbc2: format!("{:?}", kind2),
        });
    }
}

fn compare_signals_for_results(
    results: &mut Vec<ComparisonResult>,
    msg1: &rs_dbc::Message, 
    msg2: &rs_dbc::Message
) {
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
    let mut all_signal_names: HashSet<&str> = HashSet::new();
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
                compare_signal_properties_for_results(results, msg_name, s1, s2);
            },
            (Some(s1), None) => {
                // Only DBC1 has this signal
                results.push(ComparisonResult {
                    result_type: "Signal".to_string(),
                    message: msg_name.to_string(),
                    signal: s1.name().to_string(),
                    field: "Exists".to_string(),
                    dbc1: "Yes".to_string(),
                    dbc2: "No".to_string(),
                });
            },
            (None, Some(s2)) => {
                // Only DBC2 has this signal
                results.push(ComparisonResult {
                    result_type: "Signal".to_string(),
                    message: msg_name.to_string(),
                    signal: s2.name().to_string(),
                    field: "Exists".to_string(),
                    dbc1: "No".to_string(),
                    dbc2: "Yes".to_string(),
                });
            },
            (None, None) => unreachable!(),
        }
    }
}

fn compare_signal_properties_for_results(
    results: &mut Vec<ComparisonResult>,
    msg_name: &str,
    sig1: &rs_dbc::Signal,
    sig2: &rs_dbc::Signal
) {
    let signal_name = sig1.name();
    
    // Check both raw and Vector start bits
    let raw_bit1 = sig1.start_bit();
    let raw_bit2 = sig2.start_bit();
    let vector_bit1 = sig1.vector_start_bit();
    let vector_bit2 = sig2.vector_start_bit();
    
    let raw_different = raw_bit1 != raw_bit2;
    let vector_different = vector_bit1 != vector_bit2;
    
    // If Vector bits are different, show Vector output only
    if vector_different {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Start Bit (Vector)".to_string(),
            dbc1: vector_bit1.to_string(),
            dbc2: vector_bit2.to_string(),
        });
    }
    // If Vector bits are same but raw bits are different, show raw output only
    if raw_different {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Start Bit (Raw)".to_string(),
            dbc1: raw_bit1.to_string(),
            dbc2: raw_bit2.to_string(),
        });
    }
    
    // Compare signal size
    if sig1.signal_size() != sig2.signal_size() {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Length".to_string(),
            dbc1: sig1.signal_size().to_string(),
            dbc2: sig2.signal_size().to_string(),
        });
    }
    
    // Compare factor
    if (sig1.factor() - sig2.factor()).abs() > f64::EPSILON {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Factor".to_string(),
            dbc1: sig1.factor().to_string(),
            dbc2: sig2.factor().to_string(),
        });
    }
    
    // Compare offset
    if (sig1.offset() - sig2.offset()).abs() > f64::EPSILON {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Offset".to_string(),
            dbc1: sig1.offset().to_string(),
            dbc2: sig2.offset().to_string(),
        });
    }
    
    // Compare min value
    if (sig1.min() - sig2.min()).abs() > f64::EPSILON {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Min Value".to_string(),
            dbc1: sig1.min().to_string(),
            dbc2: sig2.min().to_string(),
        });
    }
    
    // Compare max value
    if (sig1.max() - sig2.max()).abs() > f64::EPSILON {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Max Value".to_string(),
            dbc1: sig1.max().to_string(),
            dbc2: sig2.max().to_string(),
        });
    }
    
    // Compare unit
    let unit1 = if sig1.unit().trim().is_empty() { "No Unit" } else { sig1.unit() };
    let unit2 = if sig2.unit().trim().is_empty() { "No Unit" } else { sig2.unit() };
    if unit1 != unit2 {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Unit".to_string(),
            dbc1: unit1.to_string(),
            dbc2: unit2.to_string(),
        });
    }
    
    // Compare byte order
    if sig1.byte_order() != sig2.byte_order() {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Byte Order".to_string(),
            dbc1: format!("{:?}", sig1.byte_order()),
            dbc2: format!("{:?}", sig2.byte_order()),
        });
    }
    
    // Compare value type
    if sig1.value_type() != sig2.value_type() {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Value Type".to_string(),
            dbc1: format!("{:?}", sig1.value_type()),
            dbc2: format!("{:?}", sig2.value_type()),
        });
    }
    
    // Compare receivers
    if sig1.receivers() != sig2.receivers() {
        let receivers1 = format_receivers(sig1.receivers());
        let receivers2 = format_receivers(sig2.receivers());
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Receivers".to_string(),
            dbc1: receivers1,
            dbc2: receivers2,
        });
    }
    
    // Compare multiplexer type
    if sig1.multiplexer_type() != sig2.multiplexer_type() {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Multiplexer Type".to_string(),
            dbc1: format!("{:?}", sig1.multiplexer_type()),
            dbc2: format!("{:?}", sig2.multiplexer_type()),
        });
    }
    
    // Compare initial values
    let raw_initial1 = sig1.initial_value();
    let raw_initial2 = sig2.initial_value();
    let vector_initial1 = sig1.vector_initial_value();
    let vector_initial2 = sig2.vector_initial_value();
    
    let raw_initial_different = (raw_initial1 - raw_initial2).abs() > f64::EPSILON;
    let vector_initial_different = (vector_initial1 - vector_initial2).abs() > f64::EPSILON;
    
    // If Vector initial values are different, show Vector output only
    if vector_initial_different {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Initial Value (Vector)".to_string(),
            dbc1: vector_initial1.to_string(),
            dbc2: vector_initial2.to_string(),
        });
    }
    // If Vector initial values are same but raw initial values are different, show raw output only
    if raw_initial_different {
        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Initial Value (Raw)".to_string(),
            dbc1: raw_initial1.to_string(),
            dbc2: raw_initial2.to_string(),
        });
    }
}

fn export_comparison_to_csv(dbc1: &Dbc, dbc2: &Dbc, path: &str) -> Result<(), Box<dyn Error>> {
    let mut csv_file = File::create(path)?;
    writeln!(csv_file, "Type,Message,Signal,Field,DBC1,DBC2")?;
    compare_dbc_to_csv(dbc1, dbc2, &mut csv_file)?;
    Ok(())
}

fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

fn compare_dbc_to_csv(dbc1: &Dbc, dbc2: &Dbc, csv_file: &mut File) -> Result<(), Box<dyn Error>> {
	
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
    let mut all_message_names: HashSet<String> = HashSet::new();
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
                compare_message_properties(csv_file, m1, m2)?;
                compare_signals(csv_file, m1, m2)?;
            },
            (Some(m1), None) => {
                // Only DBC1 has this message
                write_message_only_in_dbc(csv_file, m1, "DBC1")?;
            },
            (None, Some(m2)) => {
                // Only DBC2 has this message
                write_message_only_in_dbc(csv_file, m2, "DBC2")?;
            },
            (None, None) => unreachable!(),
        }
    }
	
    Ok(())
}

fn compare_message_properties(
    csv_file: &mut File, 
    msg1: &rs_dbc::Message, 
    msg2: &rs_dbc::Message
) -> Result<(), Box<dyn Error>> {
    
    let msg_name = msg1.message_name();
    
    // Compare message size
    if msg1.message_size() != msg2.message_size() {
        writeln!(csv_file, "Message,{},, DLC,{},{}", 
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
        writeln!(csv_file, "Message,{},, Message ID,0x{:X},0x{:X}", 
                msg_name, id1, id2)?;
    }
    
    // Compare message ID kind
    if kind1 != kind2 {
        writeln!(csv_file, "Message,{},, ID Format,{},{}", 
                msg_name, kind1, kind2)?;
    }
    
    Ok(())
}

fn compare_signals(
    csv_file: &mut File, 
    msg1: &rs_dbc::Message, 
    msg2: &rs_dbc::Message
) -> Result<(), Box<dyn Error>> {
    
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
    let mut all_signal_names: HashSet<&str> = HashSet::new();
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
    csv_file: &mut File,
    msg_name: &str,
    sig1: &rs_dbc::Signal,
    sig2: &rs_dbc::Signal
) -> Result<(), Box<dyn Error>> {
    
    let signal_name = sig1.name();
    
    // Check both raw and Vector start bits
    let raw_bit1 = sig1.start_bit();
    let raw_bit2 = sig2.start_bit();
    let vector_bit1 = sig1.vector_start_bit();
    let vector_bit2 = sig2.vector_start_bit();
    
    let raw_different = raw_bit1 != raw_bit2;
    let vector_different = vector_bit1 != vector_bit2;
    
    // If Vector bits are different, show Vector output only
    if vector_different {
        writeln!(csv_file, "Signal,{},{}, Start Bit (Vector),{},{}", 
                msg_name, signal_name, vector_bit1, vector_bit2)?;
    }
    // If Vector bits are same but raw bits are different, show raw output only
    if raw_different {
        writeln!(csv_file, "Signal,{},{}, Start Bit (Raw),{},{}", 
                msg_name, signal_name, raw_bit1, raw_bit2)?;
    }
    
    // Compare signal size
    if sig1.signal_size() != sig2.signal_size() {
        writeln!(csv_file, "Signal,{},{}, Length,{},{}", 
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
    let unit1 = if sig1.unit().trim().is_empty() { "No Unit" } else { sig1.unit() };
    let unit2 = if sig2.unit().trim().is_empty() { "No Unit" } else { sig2.unit() };
    if unit1 != unit2 {
        writeln!(csv_file, "Signal,{},{}, Unit,{},{}", 
                msg_name, signal_name, unit1, unit2)?;
    }
    
    // Compare byte order
    if sig1.byte_order() != sig2.byte_order() {
        writeln!(csv_file, "Signal,{},{}, Byte Order,{},{}", 
                msg_name, signal_name, sig1.byte_order(), sig2.byte_order())?;
    }
    
    // Compare value type
    if sig1.value_type() != sig2.value_type() {
        writeln!(csv_file, "Signal,{},{}, Value Type,{},{}", 
                msg_name, signal_name, sig1.value_type(), sig2.value_type())?;
    }
    
    // Compare receivers
    if sig1.receivers() != sig2.receivers() {
        let receivers1 = format_receivers(sig1.receivers());
        let receivers2 = format_receivers(sig2.receivers());
        writeln!(csv_file, "Signal,{},{}, Receivers,{},{}", 
                msg_name, signal_name, receivers1, receivers2)?;
    }
    
    // Compare multiplexer type
    if sig1.multiplexer_type() != sig2.multiplexer_type() {
        writeln!(csv_file, "Signal,{},{}, Multiplexer Type,{},{}", 
                msg_name, signal_name, sig1.multiplexer_type(), sig2.multiplexer_type())?;
    }
    
    // Compare initial values
    let raw_initial1 = sig1.initial_value();
    let raw_initial2 = sig2.initial_value();
    let vector_initial1 = sig1.vector_initial_value();
    let vector_initial2 = sig2.vector_initial_value();
    
    let raw_initial_different = (raw_initial1 - raw_initial2).abs() > f64::EPSILON;
    let vector_initial_different = (vector_initial1 - vector_initial2).abs() > f64::EPSILON;
    
    // If Vector initial values are different, show Vector output only
    if vector_initial_different {
        writeln!(csv_file, "Signal,{},{}, Initial Value (Vector),{},{}", 
                msg_name, signal_name, vector_initial1, vector_initial2)?;
    }
    // If Vector initial values are same but raw initial values are different, show raw output only
    if raw_initial_different {
        writeln!(csv_file, "Signal,{},{}, Initial Value (Raw),{},{}", 
                msg_name, signal_name, raw_initial1, raw_initial2)?;
    }
    
    // Compare value descriptions
    compare_value_descriptions(csv_file, msg_name, signal_name, sig1, sig2)?;
    
    Ok(())
}

fn normalize(text: &str) -> String {

    let replaced = text
        .to_lowercase()
        .replace('-', " ")
        .replace('_', " ");

    let mut words: Vec<String> = replaced
        .to_lowercase()
        .split_whitespace()
        .map(|word| word.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|word| !word.is_empty())
        .map(|word| word.to_string())
        .collect();
    
    words.sort(); // Alphabetical sorting
    words.join(" ")
}

fn is_equivalent(term1: &str, term2: &str) -> bool {
    normalize(term1) == normalize(term2)
}

fn calculate_similarity(s1: &str, s2: &str) -> f64 {
    // First check if they're equivalent after normalization
    if is_equivalent(s1, s2) {
        return 1.0;
    }
    
    // If not equivalent, use Jaro-Winkler for partial similarity
    let normalized_s1 = normalize(s1);
    let normalized_s2 = normalize(s2);
    
    strsim::jaro_winkler(&normalized_s1, &normalized_s2)
}

fn compare_value_descriptions(
    csv_file: &mut File,
    msg_name: &str,
    signal_name: &str,
    sig1: &rs_dbc::Signal,
    sig2: &rs_dbc::Signal
) -> Result<(), Box<dyn Error>> {
    
    let val_desc1 = sig1.value_descriptions();
    let val_desc2 = sig2.value_descriptions();
    
    // Get all unique values from both signals
    let mut all_values: HashSet<u64> = HashSet::new();
    all_values.extend(val_desc1.keys());
    all_values.extend(val_desc2.keys());
    
    // Compare each value description
    for &value in &all_values {
        let desc1 = val_desc1.get(&value);
        let desc2 = val_desc2.get(&value);
        
        match (desc1, desc2) {
            (Some(d1), Some(d2)) => {
                // Handle empty or whitespace-only descriptions
                let desc1_display = if d1.trim().is_empty() { "No Description" } else { d1 };
                let desc2_display = if d2.trim().is_empty() { "No Description" } else { d2 };
                
                // Calculate similarity between descriptions using Jaro-Winkler
                let similarity = calculate_similarity(desc1_display, desc2_display);
                // Only report as different if similarity is below threshold (0.85 = 85% similar)
                // Jaro-Winkler is better at handling common prefixes and minor variations
                if similarity < 0.85 {
                    writeln!(csv_file, "Signal,{},{}, Value 0x{:X} Description,{},{}", 
                            msg_name, signal_name, value, 
                            escape_csv_field(desc1_display), 
                            escape_csv_field(desc2_display))?;
                }
            },
            (Some(d1), None) => {
                let desc1_display = if d1.trim().is_empty() { "No Description" } else { d1 };
                writeln!(csv_file, "Signal,{},{}, Value 0x{:X} Description,{},No Description", 
                        msg_name, signal_name, value, escape_csv_field(desc1_display))?;
            },
            (None, Some(d2)) => {
                let desc2_display = if d2.trim().is_empty() { "No Description" } else { d2 };
                writeln!(csv_file, "Signal,{},{}, Value 0x{:X} Description,No Description,{}", 
                        msg_name, signal_name, value, escape_csv_field(desc2_display))?;
            },
            (None, None) => unreachable!(),
        }
    }
    
    Ok(())
}

fn format_receivers(receivers: &Vec<String>) -> String {
    if receivers.is_empty() {
        return "No Receivers".to_string();
    }
    
    let filtered_receivers: Vec<&String> = receivers
        .iter()
        .filter(|r| !r.starts_with("Vector__XXX"))
        .collect();
    
    if filtered_receivers.is_empty() {
        "No Receivers".to_string()
    } else {
        let joined = filtered_receivers
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>()
            .join(",");
        
        // If there are multiple receivers, wrap in quotes to prevent CSV issues
        if filtered_receivers.len() > 1 {
            format!("\"{}\"", joined)
        } else {
            joined
        }
    }
}

fn write_message_only_in_dbc(
    csv_file: &mut File,
    msg: &rs_dbc::Message,
    dbc_name: &str
) -> Result<(), Box<dyn Error>> {
    
    let msg_name = msg.message_name();
    let (dbc1_val, dbc2_val) = if dbc_name == "DBC1" { ("Yes", "No") } else { ("No", "Yes") };
    
    writeln!(csv_file, "Message,{},, Exists,{},{}", msg_name, dbc1_val, dbc2_val)?;
    
    Ok(())
}
