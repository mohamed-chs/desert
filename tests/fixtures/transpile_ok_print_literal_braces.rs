fn main() {
    let mut xs = vec![1, 2, 3];
    println!("{}", format!("raw {{}} value {:?}", xs[0]));
    println!("{}", format!("next {{}} moved {:?} tail {:?}", std::mem::take(&mut xs[1]), xs));
}

