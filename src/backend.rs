use clap::Parser;
use rs_dbc::Dbc;
use rust_xlsxwriter::{Color, Format, FormatBorder, Workbook};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the first DBC file
    dbc1: String,

    /// Path to the second DBC file
    dbc2: String,

    /// Optional path to export the comparison as a XLSX file
    #[arg(short, long)]
    export_xlsx: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ComparisonResult {
    result_type: String,
    message: String,
    signal: String,
    field: String,
    dbc1: String,
    dbc2: String,
}

fn load_dbc(path: &str) -> Result<Dbc, Box<dyn Error>> {
    let mut file = File::open(path)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer)?;
    Dbc::from_slice_lossy(&buffer)
        .map_err(|e| format!("Failed to parse DBC file '{}': {:?}", path, e).into())
}

fn main() {
    let args = Args::parse();

    match (load_dbc(&args.dbc1), load_dbc(&args.dbc2)) {
        (Ok(dbc1), Ok(dbc2)) => {
            if let Some(xlsx_path) = args.export_xlsx {
                match compare_dbc_files(&dbc1, &dbc2) {
                    Ok(results) => match export_comparison_to_xlsx(&results, &xlsx_path) {
                        Ok(_) => {
                            println!(
                                r#"{{"status": "success", "message": "XLSX exported successfully"}}"#
                            );
                        }
                        Err(e) => {
                            eprintln!(
                                r#"{{"status": "error", "message": "Failed to export XLSX: {}"}}"#,
                                e
                            );
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!(
                            r#"{{"status": "error", "message": "Failed to compare DBC files: {}"}}"#,
                            e
                        );
                        std::process::exit(1);
                    }
                }
            } else {
                match compare_dbc_files(&dbc1, &dbc2) {
                    Ok(results) => match serde_json::to_string(&results) {
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            eprintln!(
                                r#"{{"status": "error", "message": "Failed to serialize results to JSON: {}"}}"#,
                                e
                            );
                            std::process::exit(1);
                        }
                    },
                    Err(e) => {
                        eprintln!(
                            r#"{{"status": "error", "message": "Failed to compare DBC files: {}"}}"#,
                            e
                        );
                        std::process::exit(1);
                    }
                }
            }
        }
        (Err(e), _) => {
            eprintln!(
                r#"{{"status": "error", "message": "Failed to load DBC1: {}"}}"#,
                e
            );
            std::process::exit(1);
        }
        (_, Err(e)) => {
            eprintln!(
                r#"{{"status": "error", "message": "Failed to load DBC2: {}"}}"#,
                e
            );
            std::process::exit(1);
        }
    }
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
            }
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
            }
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
            }
            (None, None) => unreachable!(),
        }
    }

    Ok(results)
}

fn compare_message_properties_for_results(
    results: &mut Vec<ComparisonResult>,
    msg1: &rs_dbc::Message,
    msg2: &rs_dbc::Message,
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

    // Compare Tx Method
    if msg1.tx_method() != msg2.tx_method() {
        results.push(ComparisonResult {
            result_type: "Message".to_string(),
            message: msg_name.to_string(),
            signal: "".to_string(),
            field: "Tx Method".to_string(),
            dbc1: msg1.tx_method().to_string(),
            dbc2: msg2.tx_method().to_string(),
        });
    }
}

fn compare_signals_for_results(
    results: &mut Vec<ComparisonResult>,
    msg1: &rs_dbc::Message,
    msg2: &rs_dbc::Message,
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
            }
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
            }
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
            }
            (None, None) => unreachable!(),
        }
    }
}

fn compare_signal_properties_for_results(
    results: &mut Vec<ComparisonResult>,
    msg_name: &str,
    sig1: &rs_dbc::Signal,
    sig2: &rs_dbc::Signal,
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
    let unit1 = if sig1.unit().trim().is_empty() {
        "No Unit"
    } else {
        sig1.unit()
    };
    let unit2 = if sig2.unit().trim().is_empty() {
        "No Unit"
    } else {
        sig2.unit()
    };
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

    // Compare Value Descriptions
    let vd1 = sig1.vector_value_descriptions();
    let vd2 = sig2.vector_value_descriptions();

    if vd1 != vd2 {
        let format_vd = |vd: &Vec<(String, String)>| -> String {
            if vd.is_empty() {
                "None".to_string()
            } else {
                vd.iter()
                    .map(|(k, v)| format!("{} = \"{}\"", k, v))
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        };

        results.push(ComparisonResult {
            result_type: "Signal".to_string(),
            message: msg_name.to_string(),
            signal: signal_name.to_string(),
            field: "Value Description".to_string(),
            dbc1: format_vd(&vd1),
            dbc2: format_vd(&vd2),
        });
    }
}

fn export_comparison_to_xlsx(
    results: &[ComparisonResult],
    path: &str,
) -> Result<(), Box<dyn Error>> {
    let mut workbook = Workbook::new();

    // Group results by logical category based on result_type and field
    let mut categorized_results: HashMap<String, Vec<&ComparisonResult>> = HashMap::new();

    for res in results {
        let category = match res.result_type.as_str() {
            "Message" => match res.field.as_str() {
                "Exists" => "Message Exists".to_string(),
                "DLC" => "Message DLC".to_string(),
                "Cycle Time" => "Message Cycle Time".to_string(),
                "Transmitter" => "Message Transmitter".to_string(),
                "Tx Method" => "Message Tx Method".to_string(),
                "Message ID" => "Message ID".to_string(),
                "ID Format" => "Message ID Format".to_string(),
                _ => "Other Messages".to_string(),
            },
            "Signal" => match res.field.as_str() {
                "Exists" => "Signal Exists".to_string(),
                "Start Bit (Vector)" | "Start Bit (Raw)" => "Start Bit".to_string(),
                "Length" => "Length".to_string(),
                "Factor" | "Offset" => "Factor & Offset".to_string(),
                "Min Value" | "Max Value" => "Min & Max".to_string(),
                "Initial Value (Vector)" | "Initial Value (Raw)" => "Initial Value".to_string(),
                "Byte Order" => "Byte Order".to_string(),
                "Unit" => "Unit".to_string(),
                "Receivers" => "Receivers".to_string(),
                "Value Type" | "Multiplexer Type" => "Type & Multiplexer".to_string(),
                f if f.starts_with("Value") && f.ends_with("Description") => {
                    "Value Descriptions".to_string()
                }
                _ => "Other".to_string(),
            },
            _ => "Unknown".to_string(),
        };

        categorized_results
            .entry(category)
            .or_insert_with(Vec::new)
            .push(res);
    }

    // Formats
    let header_format = Format::new()
        .set_bold()
        .set_font_color(Color::White)
        .set_background_color(Color::RGB(0x4F81BD))
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(0x8EA9DB));

    let cell_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(0x8EA9DB));

    let link_format = Format::new()
        .set_font_color(Color::Blue)
        .set_underline(rust_xlsxwriter::FormatUnderline::Single)
        .set_border(FormatBorder::Thin)
        .set_border_color(Color::RGB(0x8EA9DB));

    // Create worksheets for each category
    let mut categories: Vec<String> = categorized_results.keys().cloned().collect();
    categories.sort();

    // --- Main Index Sheet ---
    {
        let main_sheet = workbook.add_worksheet();
        main_sheet.set_name("Main")?;

        // Write headers
        main_sheet.write_string_with_format(0, 0, "Category", &header_format)?;
        main_sheet.write_string_with_format(0, 1, "Link", &header_format)?;
        main_sheet.write_string_with_format(0, 2, "Differences", &header_format)?;

        for (i, category) in categories.iter().enumerate() {
            let row = (i + 1) as u32;
            let safe_name: String = category.chars().take(31).collect();
            let count = categorized_results.get(category).unwrap().len();

            main_sheet.write_string_with_format(row, 0, category, &cell_format)?;

            let formula = format!(
                "=HYPERLINK(\"#'{}'!A1\", \"Go to '{}'\")",
                safe_name.replace("'", "''"),
                safe_name.replace("\"", "\"\"")
            );
            main_sheet.write_formula_with_format(row, 1, formula.as_str(), &link_format)?;

            main_sheet.write_number_with_format(row, 2, count as f64, &cell_format)?;
        }

        main_sheet.autofilter(0, 0, categories.len() as u32, 2)?;
        main_sheet.set_column_width(0, 30)?;
        main_sheet.set_column_width(1, 35)?;
        main_sheet.set_column_width(2, 15)?;
    }

    // --- Individual Category Sheets ---
    for category in categories {
        let sheet = workbook.add_worksheet();
        // Sheet names have a max length of 31 chars in Excel
        let safe_name: String = category.chars().take(31).collect();
        sheet.set_name(&safe_name)?;

        // Write headers
        sheet.write_string_with_format(0, 0, "Type", &header_format)?;
        sheet.write_string_with_format(0, 1, "Message", &header_format)?;
        sheet.write_string_with_format(0, 2, "Signal", &header_format)?;
        sheet.write_string_with_format(0, 3, "Field", &header_format)?;
        sheet.write_string_with_format(0, 4, "DBC1", &header_format)?;
        sheet.write_string_with_format(0, 5, "DBC2", &header_format)?;

        // Write rows
        let rows = categorized_results.get(&category).unwrap();
        for (row_idx, row_data) in rows.iter().enumerate() {
            let row = (row_idx + 1) as u32;
            sheet.write_string_with_format(row, 0, &row_data.result_type, &cell_format)?;
            sheet.write_string_with_format(row, 1, &row_data.message, &cell_format)?;
            sheet.write_string_with_format(row, 2, &row_data.signal, &cell_format)?;
            sheet.write_string_with_format(row, 3, &row_data.field, &cell_format)?;
            sheet.write_string_with_format(row, 4, &row_data.dbc1, &cell_format)?;
            sheet.write_string_with_format(row, 5, &row_data.dbc2, &cell_format)?;
        }

        // Apply autofilter to the table range
        sheet.autofilter(0, 0, rows.len() as u32, 5)?;

        // Autofit columns (approximate by setting max width)
        sheet.set_column_width(0, 10)?;
        sheet.set_column_width(1, 20)?;
        sheet.set_column_width(2, 20)?;
        sheet.set_column_width(3, 25)?;
        sheet.set_column_width(4, 20)?;
        sheet.set_column_width(5, 20)?;
    }

    workbook.save(path)?;
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

        // If there are multiple receivers, wrap in quotes
        if filtered_receivers.len() > 1 {
            format!("\"{}\"", joined)
        } else {
            joined
        }
    }
}
