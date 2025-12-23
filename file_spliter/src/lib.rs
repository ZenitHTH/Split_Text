use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

/// # SplitConfig
/// Holds the configuration for a single output file.
/// We derive Serialize/Deserialize so this works with Tauri/JSON.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct SplitConfig {
    pub start: usize,
    pub end: usize,
    pub output_path: String,
}

impl SplitConfig {
    /// Creates a new configuration with strict validation.
    pub fn new(start: usize, end: usize, output_path: String) -> Result<Self, String> {
        if start == 0 {
            return Err(format!(
                "Line 0 is invalid for {}. Start at 1.",
                output_path
            ));
        }
        if start > end {
            return Err(format!(
                "Logic Error: Start ({}) > End ({}) for {}",
                start, end, output_path
            ));
        }
        Ok(SplitConfig {
            start,
            end,
            output_path,
        })
    }
}

/// # Split File Function
/// 1. Checks file existence & empty status.
/// 2. splits the file.
/// 3. Verifies if requested lines actually existed.
pub fn split_file<P: AsRef<Path>>(input_path: P, parts: &[SplitConfig]) -> Result<String, String> {
    let path_ref = input_path.as_ref();

    // ---------------------------------------------------------
    // 1. PRE-CHECK: Does file exist and have data?
    // ---------------------------------------------------------
    if !path_ref.exists() {
        return Err(format!("Input file not found: {}", path_ref.display()));
    }

    let metadata = fs::metadata(path_ref).map_err(|e| e.to_string())?;
    if metadata.len() == 0 {
        return Err(format!(
            "Input file is empty (0 bytes): {}",
            path_ref.display()
        ));
    }

    let input_file = File::open(path_ref).map_err(|e| format!("Open error: {}", e))?;
    let reader = BufReader::new(input_file);

    // ---------------------------------------------------------
    // 2. PREPARE WRITERS
    // ---------------------------------------------------------
    let mut writers: Vec<BufWriter<File>> = Vec::new();
    for part in parts {
        let f = File::create(&part.output_path)
            .map_err(|e| format!("Cannot create {}: {}", part.output_path, e))?;
        writers.push(BufWriter::new(f));
    }

    // ---------------------------------------------------------
    // 3. PROCESS LINES
    // ---------------------------------------------------------
    let mut total_lines_read = 0;

    for (index, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|e| format!("Read error at line {}: {}", index + 1, e))?;
        let current_line_num = index + 1;
        total_lines_read = current_line_num;

        for (i, config) in parts.iter().enumerate() {
            if current_line_num >= config.start && current_line_num <= config.end {
                writeln!(writers[i], "{}", line).map_err(|e| format!("Write error: {}", e))?;
            }
        }
    }

    // Flush buffers to ensure data is on disk
    for mut w in writers {
        w.flush().map_err(|e| format!("Disk error: {}", e))?;
    }

    // ---------------------------------------------------------
    // 4. POST-CHECK: Did the file have enough lines?
    // ---------------------------------------------------------
    let mut errors = Vec::new();

    for part in parts {
        // If the file ended before this part even started
        if total_lines_read < part.start {
            errors.push(format!(
                "❌ Range {}-{} failed: Input file only has {} lines.",
                part.start, part.end, total_lines_read
            ));

            // Delete the empty file to keep things clean
            if let Err(e) = fs::remove_file(&part.output_path) {
                eprintln!(
                    "Warning: Could not cleanup empty file {}: {}",
                    part.output_path, e
                );
            }
        }
    }

    if !errors.is_empty() {
        return Err(errors.join("\n"));
    }

    Ok(format!(
        "Success! Processed {} lines into {} files.",
        total_lines_read,
        parts.len()
    ))
}
