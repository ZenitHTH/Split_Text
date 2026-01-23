use file_spliter::SplitConfig;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
pub enum SplitMode {
    Manual {
        ranges: Vec<String>,
        output_dir: Option<String>,
    },
    Auto {
        chunk_size: usize,
        output_dir: Option<String>,
    },
}

// [EXISTING HELPER] - No changes
fn validate_input_path(path_input: &String) -> Result<(), String> {
    let path = Path::new(&path_input);
    if !path.exists() {
        return Err(format!("Input path does not exist: '{}'", path_input));
    }
    if !path.is_file() {
        return Err(format!("Input path is not a file: '{}'", path_input));
    }
    Ok(())
}

// [EXISTING HELPER] - No changes
fn parse_range_string(range: String) -> Result<(usize, usize), String> {
    let parts: Vec<&str> = range.split('-').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid range '{}'", range));
    }
    let start = parts[0].parse::<usize>().map_err(|_| "Bad start")?;
    let end = parts[1].parse::<usize>().map_err(|_| "Bad end")?;
    Ok((start, end))
}

// [EXISTING HELPER] - No changes
fn generate_part_filename(parent: &Path, stem: &str, ext: &str, index: usize) -> String {
    let ext_string = if ext.is_empty() {
        String::new()
    } else {
        format!(".{}", ext)
    };
    let new_name = format!("{} - Part {}{}", stem, index + 1, ext_string);
    parent.join(new_name).to_string_lossy().to_string()
}

// [EXISTING HELPER] - No changes
fn count_total_lines(path_str: &String) -> Result<usize, String> {
    let file = File::open(path_str).map_err(|e| e.to_string())?;
    Ok(BufReader::new(file).lines().count())
}
// =========================================================================
// SPECIFIC LOGIC HANDLERS (Private)
// These do the actual heavy thinking for each mode.
// =========================================================================

/// Logic for: "1-100", "200-300"
fn plan_manual_split(
    ranges: &[String],
    parent: &Path,
    stem: &str,
    ext: &str,
) -> Result<Vec<SplitConfig>, String> {
    let mut configs = Vec::new();

    for (i, range_str) in ranges.iter().enumerate() {
        // 1. Parse numbers
        let (start, end) = parse_range_string(range_str.clone())?;

        // 2. Generate name
        let output = generate_part_filename(parent, stem, ext, i);

        // 3. Save
        configs.push(SplitConfig::new(start, end, output)?);
    }
    Ok(configs)
}

/// Logic for: "-n 3000"
fn plan_auto_split(
    input_path: &String,
    chunk_size: usize,
    parent: &Path,
    stem: &str,
    ext: &str,
) -> Result<Vec<SplitConfig>, String> {
    // 1. Count lines first
    let total_lines = count_total_lines(input_path)?;
    if total_lines == 0 {
        return Err("File is empty.".to_string());
    }

    let mut configs = Vec::new();
    let mut current_start = 1;
    let mut index = 0;

    // 2. Loop until we cover all lines
    while current_start <= total_lines {
        let mut current_end = current_start + chunk_size - 1;

        // Don't go past the end of the file
        if current_end > total_lines {
            current_end = total_lines;
        }

        let output = generate_part_filename(parent, stem, ext, index);
        configs.push(SplitConfig::new(current_start, current_end, output)?);

        current_start = current_end + 1;
        index += 1;
    }
    Ok(configs)
}

// =========================================================================
// PUBLIC CONTROLLER
// This is now clean and easy to read.
// =========================================================================
pub fn build_split_plan(input_path: String, mode: SplitMode) -> Result<Vec<SplitConfig>, String> {
    // 1. Validate & Prep (Common for both modes)
    validate_input_path(&input_path)?;

    let path_obj = Path::new(&input_path);

    // Extract output_dir based on mode
    let output_dir_opt = match &mode {
        SplitMode::Manual { output_dir, .. } => output_dir.clone(),
        SplitMode::Auto { output_dir, .. } => output_dir.clone(),
    };

    // Use provided output_dir or default to input file's parent
    let parent_dir = if let Some(ref dir) = output_dir_opt {
        Path::new(dir)
    } else {
        path_obj.parent().unwrap_or_else(|| Path::new("."))
    };

    let file_stem = path_obj
        .file_stem()
        .ok_or("Invalid filename")?
        .to_string_lossy();
    let extension = path_obj.extension().unwrap_or_default().to_string_lossy();

    // 2. Delegate to the specific function
    match mode {
        SplitMode::Manual { ranges, .. } => {
            plan_manual_split(&ranges, parent_dir, &file_stem, &extension)
        }
        SplitMode::Auto { chunk_size, .. } => {
            plan_auto_split(&input_path, chunk_size, parent_dir, &file_stem, &extension)
        }
    }
}
