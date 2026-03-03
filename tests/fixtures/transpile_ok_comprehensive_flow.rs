use std::cmp::{max as std_max};

struct Pair {
    pub left: i32,
    pub right: i32,
}
fn swap(pair: Pair) -> Pair {
    return Pair { left: pair.right, right: pair.left };
}
fn main() {
    let mut values = vec![3, 9, 1];
    let taken = std::mem::take(&mut values[1]);
    let pair = Pair { left: taken, right: std_max(values[0], values[2]) };
    let swapped = swap(pair);
    println!("{}", format!("swap {:?} {:?} values {:?}", swapped.left, swapped.right, values));
}

