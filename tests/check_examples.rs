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

#[test]
fn check_collections_example() {
    run_check("collections.ds");
}

#[test]
fn check_error_handling_example() {
    run_check("error_handling.ds");
}

#[test]
fn check_oop_example() {
    run_check("oop.ds");
}

#[test]
fn check_ai_interop_example() {
    run_check("ai_interop.ds");
}

#[test]
fn check_neural_network_example() {
    run_check("neural_network.ds");
}

#[test]
fn check_borrows_example() {
    run_check("borrows.ds");
}

#[test]
fn check_data_processing_example() {
    run_check("data_processing.ds");
}

#[test]
fn check_linked_list_example() {
    run_check("linked_list.ds");
}

#[test]
fn check_reports_translated_diagnostics_for_type_mismatch_fixture() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_type_mismatch.ds");
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains("mismatched types"))
        .stdout(predicates::str::contains("Line 2: in Desert source"))
        .stderr(predicates::str::contains(
            "Rust check failed with translated diagnostics.",
        ));
}
