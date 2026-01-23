# File Spliter

ไลบรารี Rust สำหรับแบ่งไฟล์ข้อความ (Text-based files) ออกเป็นส่วนๆ ตามช่วงบรรทัดที่กำหนด

## ฟีเจอร์ (Features)

- **Range-Based Splitting**: แยกเฉพาะบรรทัดที่ต้องการออกมาเป็นไฟล์ใหม่ (เช่น บรรทัดที่ 1-100 ไปที่ `part1.txt`)
- **Efficient Processing**: อ่านไฟล์ทีละบรรทัดด้วย `BufReader` ทำให้กินแรมน้อย แม้ไฟล์ต้นฉบับจะใหญ่มาก
- **Validation**: มีระบบตรวจสอบไฟล์ต้นฉบับว่ามีอยู่จริงและไม่ว่างเปล่า
- **Cleanup**: ลบไฟล์ปลายทางทิ้งให้อัตโนมัติ หากไฟล์ต้นฉบับจบก่อนถึงช่วงบรรทัดที่กำหนด (ป้องกันไฟล์ขยะว่างเปล่า)

## การใช้งาน (Usage)

กำหนดค่า `SplitConfig` สำหรับแต่ละส่วนที่ต้องการ แล้วเรียกใช้ฟังก์ชัน `split_file`

```rust
use file_spliter::{split_file, SplitConfig};

fn main() {
    let input = "large_log.txt";
    
    // กำหนดช่วงที่ต้องการแบ่ง
    let configs = vec![
        SplitConfig::new(1, 100, "part1.txt".to_string()).unwrap(),
        SplitConfig::new(101, 200, "part2.txt".to_string()).unwrap(),
    ];

    // สั่งแบ่งไฟล์
    match split_file(input, &configs) {
        Ok(msg) => println!("{}", msg),
        Err(e) => eprintln!("Split failed: {}", e),
    }
}
```
