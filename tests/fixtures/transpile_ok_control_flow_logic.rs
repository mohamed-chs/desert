fn main() {
    let mut x = 0;
    let mut total = 0;
    while x < 10 {
        x = x + 1;
        if x % 2 == 0 && !(x == 6) {
            continue;
        } else {
            if x == 9 || x == 10 {
                break;
            } else {
                total = total + x;
            }
        }
    }
    println!("{}", format!("total {:?} x {:?}", total, x));
}

