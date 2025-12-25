use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;
#[derive(Debug)]
pub struct SplitConfig {
    pub start: usize,
    pub end: usize,
    pub output_path: String,
}

impl SplitConfig {
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

// =========================================================================
// HELPER 1: VALIDATION
// Checks if the input file exists and has content.
// =========================================================================
fn validate_file(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("Input file not found: {}", path.display()));
    }

    let metadata = fs::metadata(path).map_err(|e| e.to_string())?;
    if metadata.len() == 0 {
        return Err(format!("Input file is empty (0 bytes): {}", path.display()));
    }

    Ok(())
}

// =========================================================================
// HELPER 2: CREATE WRITERS
// Opens all output files at once and prepares them for writing.
// =========================================================================
fn create_writers(parts: &[SplitConfig]) -> Result<Vec<BufWriter<File>>, String> {
    let mut writers = Vec::new();

    for part in parts {
        let f = File::create(&part.output_path)
            .map_err(|e| format!("Cannot create output file '{}': {}", part.output_path, e))?;
        writers.push(BufWriter::new(f));
    }

    Ok(writers)
}

// =========================================================================
// HELPER 3: CORE PROCESSING LOOP
// Reads input line-by-line and writes to the correct output(s).
// Returns the total number of lines read.
// =========================================================================
fn process_lines(
    reader: BufReader<File>,
    parts: &[SplitConfig],
    mut writers: Vec<BufWriter<File>>,
) -> Result<usize, String> {
    let mut total_lines = 0;

    for (index, line_result) in reader.lines().enumerate() {
        // Read the line safely
        let line = line_result.map_err(|e| format!("Read error at line {}: {}", index + 1, e))?;
        let current_line = index + 1;
        total_lines = current_line;

        // Check which file needs this line
        for (i, config) in parts.iter().enumerate() {
            if current_line >= config.start && current_line <= config.end {
                // Write to the specific writer
                writeln!(writers[i], "{}", line)
                    .map_err(|e| format!("Write error to '{}': {}", config.output_path, e))?;
            }
        }
    }

    // Flush all buffers to disk to ensure data is saved
    for mut w in writers {
        w.flush().map_err(|e| format!("Disk save error: {}", e))?;
    }

    Ok(total_lines)
}

// =========================================================================
// HELPER 4: CLEANUP
// Checks if any file turned out empty because the input was too short.
// =========================================================================
fn verify_and_cleanup(parts: &[SplitConfig], total_lines: usize) -> Result<(), String> {
    let mut errors = Vec::new();

    for part in parts {
        // If the file ended before this part even started
        if total_lines < part.start {
            errors.push(format!(
                "❌ Range {}-{} failed: Input file only has {} lines.",
                part.start, part.end, total_lines
            ));

            // Delete the empty garbage file
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

    Ok(())
}

// =========================================================================
// MAIN PUBLIC FUNCTION
// Now acts as a simple "Coordinator" calling the steps above.
// =========================================================================
pub fn split_file<P: AsRef<Path>>(input_path: P, parts: &[SplitConfig]) -> Result<String, String> {
    let path_ref = input_path.as_ref();

    // Step 1: Validate Input
    validate_file(path_ref)?;

    // Step 2: Open Input Reader
    let input_file = File::open(path_ref).map_err(|e| format!("Open error: {}", e))?;
    let reader = BufReader::new(input_file);

    // Step 3: Prepare Output Writers
    let writers = create_writers(parts)?;

    // Step 4: Run the Processing Loop
    let total_lines = process_lines(reader, parts, writers)?;

    // Step 5: Post-Process Verification
    verify_and_cleanup(parts, total_lines)?;

    Ok(format!("Success! Processed {} lines.", total_lines))
}
