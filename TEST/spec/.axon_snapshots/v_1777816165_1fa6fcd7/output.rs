// === AXON GENERATED CODE ===
// Agent: junior-agent-1
// Task : 1fa6fcd7-d4ad-4ab3-9ac5-76c0716862db
// File : output.rs
// ===========================

pub fn display_age(age: i32) {
    if age < 0 {
        println!("나이는 음수일 수 없습니다.");
    } else if age == 0 {
        println!("나이가 0인 것은 불가능합니다.");
    } else {
        println!("당신의 나이는 {}입니다.", age);
    }
}
