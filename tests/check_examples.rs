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
        .stdout(predicates::str::contains(
            "tests/fixtures/check_fail_type_mismatch.ds:2: in Desert source",
        ))
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
        .stderr(predicates::str::contains(
            "Parsing error at line 1, column 1",
        ))
        .stderr(predicates::str::contains("near token Def"));
}

#[test]
fn check_reports_lexer_errors_with_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_lex_unknown_token.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains(
            "Lexing error at line 2, column 15",
        ))
        .stderr(predicates::str::contains("near '^'"));
}

#[test]
fn check_reports_move_mutability_failure_with_desert_line_mapping() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_move_requires_mut.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 3, column 5: `move` requires mutable binding `xs`",
    ));
}

#[test]
fn check_reports_unique_ref_mutability_failure_with_desert_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_unique_ref_requires_mut.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 2, column 1: `~` requires mutable binding `x`",
    ));
}

#[test]
fn check_reports_move_non_place_failure_with_desert_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_move_non_place.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 2, column 5: `move` expects a mutable place expression",
    ));
}

#[test]
fn check_reports_unique_ref_non_place_failure_with_desert_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_unique_ref_non_place.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 2, column 5: `~` expects a mutable place expression",
    ));
}

#[test]
fn check_allows_unique_ref_write_through_for_move_and_borrow() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_ok_unique_ref_write_through.ds");
    cmd.assert().success();
}

#[test]
fn check_reports_assignment_requires_mutable_binding() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_assign_requires_mut.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 3, column 5: assignment requires mutable binding `x`",
    ));
}

#[test]
fn check_reports_assignment_requires_place_expression() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_assign_non_place.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 3, column 5: assignment expects a place expression",
    ));
}

#[test]
fn check_reports_constructor_unknown_field() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_constructor_unknown_field.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 6, column 5: constructor `Pair` has no field `c`",
    ));
}

#[test]
fn check_reports_constructor_duplicate_field() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_constructor_duplicate_field.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 6, column 5: constructor `Pair` received duplicate field `a`",
    ));
}

#[test]
fn check_reports_constructor_too_many_positional_arguments() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_constructor_too_many_positional.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 6, column 5: constructor `Pair` received too many positional arguments",
    ));
}

#[test]
fn check_reports_constructor_missing_fields() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_constructor_missing_fields.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 6, column 5: constructor `Pair` is missing fields: b",
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
        .stdout(predicates::str::contains(
            "tests/fixtures/check_fail_method_not_found.ds:3: in Desert source",
        ))
        .stderr(predicates::str::contains(
            "Rust check failed with translated diagnostics.",
        ));
}

#[test]
fn check_project_diagnostic_reports_imported_file_path() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/project_check_fail_imported_diag");
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains(
            "no method named `nope` found for type `i32`",
        ))
        .stdout(predicates::str::contains(
            "tests/fixtures/project_check_fail_imported_diag/src/util/math.ds:2: in Desert source",
        ))
        .stderr(predicates::str::contains(
            "Rust check failed with translated diagnostics.",
        ));
}

#[test]
fn check_project_directory_with_default_entry() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check").arg("tests/fixtures/project_ok");
    cmd.assert().success();
}

#[test]
fn check_project_directory_with_custom_entry() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check").arg("tests/fixtures/project_custom_entry");
    cmd.assert().success();
}

#[test]
fn transpile_project_directory_with_default_entry() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("transpile").arg("tests/fixtures/project_ok");
    cmd.assert().success().stdout(predicates::str::contains(
        "println!(\"{}\", \"project ok\".to_string())",
    ));
}

#[test]
fn check_project_directory_with_import_graph() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check").arg("tests/fixtures/project_import_graph");
    cmd.assert().success();
}

#[test]
fn transpile_project_directory_with_import_graph() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("transpile")
        .arg("tests/fixtures/project_import_graph");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("fn plus("))
        .stdout(predicates::str::contains("fn add("))
        .stdout(predicates::str::contains("fn main("));
}

#[test]
fn check_project_directory_reports_import_cycle() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check").arg("tests/fixtures/project_import_cycle");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("import cycle detected:"));
}

#[test]
fn graph_project_directory_with_import_graph_order() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("graph").arg("tests/fixtures/project_import_graph");
    cmd.assert().success().stdout(predicates::str::contains(
        "src/util/ops.ds\nsrc/util/math.ds\nsrc/main.ds\n",
    ));
}

#[test]
fn graph_rejects_file_input() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("graph").arg("examples/hello_world.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("must be a project directory"));
}
