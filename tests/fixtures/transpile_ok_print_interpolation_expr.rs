fn main() {
    let mut xs = vec![1, 2, 3];
    println!("{}", format!("head {:?} rest {:?}", std::mem::take(&mut xs[0]), xs));
}

