struct Counter {
    pub value: i32,
}
fn main() {
    let mut counter = Counter { value: 1 };
    counter.value = counter.value - (3 - 1);
    let mixed = 1 + 2 * 3;
    let grouped = (1 + 2) * 3;
    println!("{}", format!("vals {:?} {:?} {:?}", counter.value, mixed, grouped));
}

