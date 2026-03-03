fn main() {
    let pair = (10, 20);
    let (x, y) = (1, 2);
    for i in 0..5 {
        println!("{}", format!("{:?}", i));
    }
    for j in 1..=3 {
        println!("{}", format!("{:?}", j));
    }
}

