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

#[test]
fn check_reports_parser_errors_with_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_parse_missing_colon.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Parsing error at line 1, column 1"))
        .stderr(predicates::str::contains("near token Def"));
}

#[test]
fn check_reports_lexer_errors_with_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_lex_unknown_token.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Lexing error at line 2, column 15"))
        .stderr(predicates::str::contains("near '^'"));
}

#[test]
fn check_reports_move_mutability_failure_with_desert_line_mapping() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_move_requires_mut.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains(
            "Semantic error at line 3, column 5: `move` requires mutable binding `xs`",
        ));
}

#[test]
fn check_reports_unique_ref_mutability_failure_with_desert_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_unique_ref_requires_mut.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains(
            "Semantic error at line 2, column 1: `~` requires mutable binding `x`",
        ));
}

#[test]
fn check_reports_move_non_place_failure_with_desert_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_move_non_place.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains(
            "Semantic error at line 2, column 5: `move` expects a mutable place expression",
        ));
}

#[test]
fn check_reports_unique_ref_non_place_failure_with_desert_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_unique_ref_non_place.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains(
            "Semantic error at line 2, column 5: `~` expects a mutable place expression",
        ));
}

#[test]
fn check_reports_method_resolution_failure_with_desert_line_mapping() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_method_not_found.ds");
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains(
            "no method named `nope` found for type `i32`",
        ))
        .stdout(predicates::str::contains("Line 3: in Desert source"))
        .stderr(predicates::str::contains(
            "Rust check failed with translated diagnostics.",
        ));
}
