use file_spliter::{SplitConfig, split_file};
use std::env;
use std::path::Path;
use std::process;

// =========================================================================
// 1. VALIDATION HELPER
// INPUT:  String (Owned)
// =========================================================================
fn validate_input_path(path_input: String) -> Result<(), String> {
    // Temporarily view as path for checking
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
// 2. PARSING HELPER
// INPUT:  String (Owned)
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
// 3. NAMING HELPER
// INPUT:  String (Owned) for everything
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
// 4. MAIN TASK CREATOR
// INPUT: String (Owned Path) and Vec<String> (Owned List of Ranges)
// This function now OWNS all its data. It is completely independent.
// =========================================================================
fn create_task_list(
    input_path: String,
    range_args: Vec<String>,
) -> Result<Vec<SplitConfig>, String> {
    // Step A: Validate
    validate_input_path(input_path.clone())?;

    // Step B: Extract File Details to Owned Strings
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

    // We iterate over the owned Vector
    for (i, range_str) in range_args.iter().enumerate() {
        // 1. Parse (Pass a clone of the string)
        let (start, end) = parse_range_string(range_str.clone())?;

        // 2. Generate Name (Pass clones of stem/ext)
        let output_path =
            generate_part_filename(parent_dir, file_stem.clone(), extension.clone(), i);

        configs.push(SplitConfig::new(start, end, output_path)?);
    }

    Ok(configs)
}
// -------------------------------------------------------------------------
// MAIN EXECUTION
// -------------------------------------------------------------------------
fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        return Err(format!(
            "Missing arguments.\nUsage: {} <file> <start-end>...",
            args[0]
        ));
    }

    // Clone the path string so 'create_task_list' can own it safely
    let input_path = args[1].clone();
    // 2. Clone the ranges into a new Vector (Vec<String>)
    // args[2..] is a slice. .to_vec() creates a new, independent Vector.
    let range_args_vec = args[2..].to_vec();

    // 3. Prepare (Pass Ownership of both)
    let configs = create_task_list(input_path.clone(), range_args_vec)?;

    println!("✅ Configuration Valid. Processing '{}'...", input_path);

    // 2. Execute Split
    let success_msg = split_file(&input_path, &configs)?;

    println!("✅ {}", success_msg);
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\n❌ ERROR: {}", e);
        process::exit(1);
    }
}
