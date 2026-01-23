# File Spliter

A generic Rust library for splitting text-based files into multiple chunks based on line ranges.

## Features

- **Range-Based Splitting**: Extract specific lines into separate files (e.g., lines 1-100 to `part1.txt`, 200-300 to `part2.txt`).
- **Efficient Processing**: Reads the input file line-by-line using `BufReader`, making it memory efficient even for large files.
- **Validation**: Automatically checks if the input file exists and is not empty.
- **Cleanup**: Automatically removes output files if the input file ends before a specified range starts, preventing empty garbage files.

## Usage

Define `SplitConfig`s for each chunk you want to extract and pass them to `split_file`.

```rust
use file_spliter::{split_file, SplitConfig};

fn main() {
    let input = "large_log.txt";
    
    // Define how you want to split the file
    let configs = vec![
        SplitConfig::new(1, 100, "part1.txt".to_string()).unwrap(),
        SplitConfig::new(101, 200, "part2.txt".to_string()).unwrap(),
    ];

    // Execute the split
    match split_file(input, &configs) {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("Split failed: {}", e),
    }
}
```
