mod tasks;

use file_spliter::split_file;
use std::env;
use std::process;
use tasks::create_task_list;

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        return Err(format!(
            "Missing arguments.\nUsage: {} <file> <start-end>...",
            args[0]
        ));
    }

    // 1. Capture Arguments (Owned)
    let input_path = args[1].clone();
    let range_args_vec = args[2..].to_vec();

    // 2. Prepare Tasks (Calls src/tasks.rs)
    let configs = create_task_list(input_path.clone(), range_args_vec)?;

    println!(
        "✅ File found. Configuration Valid. Processing '{}'...",
        input_path
    );

    // 3. Execute Split (Calls src/lib.rs)
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
