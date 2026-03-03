fn main() {
    let maybe = Some(3);
    match maybe {
        Some(v) => {
            println!("{}", format!("some {:?}", v));
        }
        None => {
            println!("{}", "none".to_string());
        }
    }
    let outcome: Result<i32, String> = Ok(9);
    match outcome {
        Ok(v) => {
            println!("{}", format!("ok {:?}", v));
        }
        Err(e) => {
            println!("{}", format!("err {:?}", e));
        }
    }
    let flag = true;
    match flag {
        true => {
            println!("{}", "yes".to_string());
        }
        false => {
            println!("{}", "no".to_string());
        }
    }
}

