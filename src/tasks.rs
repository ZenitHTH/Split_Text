use file_spliter::SplitConfig;
use std::path::Path;

// =========================================================================
// 1. VALIDATION HELPER (Private)
// =========================================================================
fn validate_input_path(path_input: String) -> Result<(), String> {
    let path = Path::new(&path_input);

    if !path.exists() {
        return Err(format!("Input path does not exist: '{}'", path_input));
    }
    if !path.is_file() {
        return Err(format!("Input path is not a file: '{}'", path_input));
    }
    Ok(())
}

// =========================================================================
// 2. PARSING HELPER (Private)
// =========================================================================
fn parse_range_string(range: String) -> Result<(usize, usize), String> {
    let parts: Vec<&str> = range.split('-').collect();

    if parts.len() != 2 {
        return Err(format!(
            "Invalid range format '{}'. Use 'start-end'.",
            range
        ));
    }

    let start = parts[0]
        .parse::<usize>()
        .map_err(|_| format!("Invalid start number in '{}'", range))?;

    let end = parts[1]
        .parse::<usize>()
        .map_err(|_| format!("Invalid end number in '{}'", range))?;

    Ok((start, end))
}

// =========================================================================
// 3. NAMING HELPER (Private)
// =========================================================================
fn generate_part_filename(parent: &Path, stem: String, ext: String, index: usize) -> String {
    let ext_string = if ext.is_empty() {
        String::new()
    } else {
        format!(".{}", ext)
    };

    let new_name = format!("{} - Part {}{}", stem, index + 1, ext_string);
    parent.join(new_name).to_string_lossy().to_string()
}

// =========================================================================
// 4. MAIN TASK CREATOR (Public)
// This is the only function 'main.rs' needs to see.
// =========================================================================
pub fn create_task_list(
    input_path: String,
    range_args: Vec<String>,
) -> Result<Vec<SplitConfig>, String> {
    // Step A: Validate
    validate_input_path(input_path.clone())?;

    // Step B: Extract File Details
    let path_obj = Path::new(&input_path);
    let parent_dir = path_obj.parent().unwrap_or_else(|| Path::new("."));

    let file_stem = path_obj
        .file_stem()
        .ok_or("Invalid filename")?
        .to_string_lossy()
        .to_string();

    let extension = path_obj
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Step C: Build Configs
    let mut configs = Vec::new();

    for (i, range_str) in range_args.iter().enumerate() {
        let (start, end) = parse_range_string(range_str.clone())?;

        let output_path =
            generate_part_filename(parent_dir, file_stem.clone(), extension.clone(), i);

        configs.push(SplitConfig::new(start, end, output_path)?);
    }

    Ok(configs)
}
