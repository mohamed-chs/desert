use assert_cmd::cargo::cargo_bin_cmd;

fn run_check(example: &str) {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check").arg(format!("examples/{example}"));
    cmd.assert().success();
}

#[test]
fn check_hello_world_example() {
    run_check("hello_world.ds");
}

#[test]
fn check_math_example() {
    run_check("math.ds");
}
