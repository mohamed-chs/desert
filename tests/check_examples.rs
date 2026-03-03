use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::PredicateBooleanExt;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn run_check(example: &str) {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check").arg(format!("examples/{example}"));
    cmd.assert().success();
}

fn run_transpile(input: &str) -> String {
    let mut cmd = cargo_bin_cmd!("desert");
    let output = cmd
        .arg("transpile")
        .arg(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("transpile output should be valid UTF-8")
}

fn assert_transpile_matches_fixture(input: &str, expected_output: &str) {
    let actual = run_transpile(input);
    let expected =
        fs::read_to_string(expected_output).expect("expected fixture should be readable");
    assert_eq!(actual, expected);
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
fn check_imports_example() {
    run_check("imports.ds");
}

#[test]
fn check_imports_libraries_example() {
    run_check("imports_libraries.ds");
}

#[test]
fn check_match_option_example() {
    run_check("match_option.ds");
}

#[test]
fn check_mutable_pipeline_example() {
    run_check("mutable_pipeline.ds");
}

#[test]
fn check_shape_protocol_example() {
    run_check("shape_protocol.ds");
}

#[test]
fn check_struct_constructors_example() {
    run_check("struct_constructors.ds");
}

#[test]
fn check_transpile_showcase_example() {
    run_check("transpile_showcase.ds");
}

#[test]
fn check_operator_precedence_example() {
    run_check("operator_precedence.ds");
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
            "tests/fixtures/check_fail_type_mismatch.ds:2:12: in Desert source",
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
fn check_stage_syntax_reports_parser_errors_with_location() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("--stage")
        .arg("syntax")
        .arg("tests/fixtures/check_fail_parse_missing_colon.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains(
            "Parsing error at line 1, column 1",
        ))
        .stderr(predicates::str::contains("near token Def"));
}

#[test]
fn check_stage_syntax_allows_semantic_error_fixture() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("--stage")
        .arg("syntax")
        .arg("tests/fixtures/check_fail_move_requires_mut.ds");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Syntax check passed."));
}

#[test]
fn check_stage_semantic_reports_semantic_failures() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("--stage")
        .arg("semantic")
        .arg("tests/fixtures/check_fail_move_requires_mut.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 3, column 5: `move` requires mutable binding `xs`",
    ));
}

#[test]
fn check_stage_semantic_skips_rust_type_checking() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("--stage")
        .arg("semantic")
        .arg("tests/fixtures/check_fail_type_mismatch.ds");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Semantic check passed."));
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
fn check_reports_nested_import_requires_top_level() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_nested_import.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 2, column 5: `import` is only allowed at top level",
    ));
}

#[test]
fn check_file_input_with_imports_resolves_symbols() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_ok_file_import_entry.ds");
    cmd.assert().success();
}

#[test]
fn check_file_input_with_rust_import_resolves_symbols() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_ok_rust_import_std_max.ds");
    cmd.assert().success();
}

#[test]
fn check_file_input_with_rust_from_import_alias_resolves_symbols() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_ok_rust_from_import_alias.ds");
    cmd.assert().success();
}

#[test]
fn check_file_input_with_rust_core_from_import_alias_resolves_symbols() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_ok_rust_from_import_core_alias.ds");
    cmd.assert().success();
}

#[test]
fn check_rejects_unsupported_rust_import_root() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_rust_import_unsupported_root.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "unsupported rust import root `serde` (only std/core/alloc are supported)",
    ));
}

#[test]
fn check_rejects_aliasing_file_import() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_file_import_alias_unsupported.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "aliasing non-rust imports is unsupported (use plain `import \"path\"`)",
    ));
}

#[test]
fn check_rejects_aliasing_file_from_import_items() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_file_from_import_alias_unsupported.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "aliasing non-rust from-import items is unsupported (remove `as ...`)",
    ));
}

#[test]
fn check_rejects_non_rust_from_import_items() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_file_from_import_unsupported.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 1, column 1: non-rust `from ... import ...` is unsupported (use plain `import \"path\"`)",
    ));
}

#[test]
fn check_rejects_non_rust_from_import_before_graph_resolution() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_file_from_import_missing_target.ds");
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains(
            "Semantic error at line 1, column 1: non-rust `from ... import ...` is unsupported",
        ))
        .stderr(predicates::str::contains("failed to resolve import").not());
}

#[test]
fn check_rejects_duplicate_from_import_items() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_from_import_duplicate_items.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 1, column 1: duplicate from-import item `max`",
    ));
}

#[test]
fn check_reports_return_outside_def() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_return_outside_def.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 1, column 1: `return` is only allowed inside `def` bodies",
    ));
}

#[test]
fn check_reports_duplicate_function_parameters() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_duplicate_params.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 1, column 1: duplicate parameter `x` in function signature",
    ));
}

#[test]
fn check_reports_duplicate_local_binding_in_same_scope() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_duplicate_local_binding.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 3, column 5: duplicate local binding `x` in same scope",
    ));
}

#[test]
fn check_reports_unknown_identifier_in_expression() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_unknown_identifier.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 2, column 5: unknown identifier `missing`",
    ));
}

#[test]
fn check_reports_top_level_function_call_arity_mismatch() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_call_arity_top_level.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 5, column 5: call to `add` expects 2 argument(s), found 1",
    ));
}

#[test]
fn check_reports_local_forward_function_call_arity_mismatch() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_call_arity_local_forward.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 2, column 5: call to `helper` expects 1 argument(s), found 2",
    ));
}

#[test]
fn check_reports_generic_function_call_arity_mismatch() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_call_arity_generic.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 5, column 5: call to `id` expects 1 argument(s), found 2",
    ));
}

#[test]
fn check_reports_duplicate_local_def_in_same_scope() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_duplicate_local_def.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 4, column 5: duplicate local name `helper` in same scope",
    ));
}

#[test]
fn check_reports_duplicate_top_level_function_names() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_duplicate_top_level_def.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 4, column 1: duplicate top-level function `foo`",
    ));
}

#[test]
fn check_reports_top_level_name_collision_across_declaration_kinds() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_top_level_name_collision.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 4, column 1: top-level name `Foo` is already declared as function",
    ));
}

#[test]
fn check_reports_duplicate_struct_fields() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_struct_duplicate_fields.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 1, column 1: duplicate field `x` in struct `Pair`",
    ));
}

#[test]
fn check_reports_duplicate_impl_method_names() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_duplicate_method.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 8, column 5: duplicate method `bump` in impl for `Counter`",
    ));
}

#[test]
fn check_reports_impl_body_requires_method_declarations() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_non_method_statement.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 5, column 5: impl for `Counter` can only contain `def` method declarations",
    ));
}

#[test]
fn check_reports_impl_unknown_protocol() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_unknown_protocol.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 4, column 1: impl references unknown protocol `Speak`",
    ));
}

#[test]
fn check_reports_impl_unknown_target_type() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_unknown_target.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 5, column 1: impl target `Dog` must be a declared struct",
    ));
}

#[test]
fn check_reports_impl_protocol_unknown_method() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_unknown_protocol_method.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 9, column 5: impl for protocol `Speak` on `Dog` defines unknown method `bark`",
    ));
}

#[test]
fn check_reports_impl_protocol_missing_method() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_missing_protocol_method.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "Semantic error at line 11, column 1: impl for protocol `Runner` on `Worker` is missing methods: stop",
    ));
}

#[test]
fn check_reports_impl_protocol_parameter_count_mismatch() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_protocol_param_count.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "incompatible signature for method `add`: expected 3 parameter(s), found 2",
    ));
}

#[test]
fn check_reports_impl_protocol_parameter_mutability_mismatch() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_protocol_mutability_mismatch.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "incompatible signature for method `bump`: parameter 1 mutability mismatch",
    ));
}

#[test]
fn check_reports_impl_protocol_return_type_mismatch() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check")
        .arg("tests/fixtures/check_fail_impl_protocol_return_type_mismatch.ds");
    cmd.assert().failure().stderr(predicates::str::contains(
        "incompatible signature for method `label`: return type mismatch",
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
            "tests/fixtures/check_fail_method_not_found.ds:3:7: in Desert source",
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
            "tests/fixtures/project_check_fail_imported_diag/src/util/math.ds:2:14: in Desert source",
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
fn check_project_directory_without_manifest_uses_fallback_entry() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("check").arg("tests/fixtures/project_no_manifest");
    cmd.assert().success();
}

#[test]
fn check_defaults_to_current_directory() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.current_dir("tests/fixtures/project_ok");
    cmd.arg("check");
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
fn transpile_project_directory_without_manifest_uses_fallback_entry() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("transpile")
        .arg("tests/fixtures/project_no_manifest");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("fn add("))
        .stdout(predicates::str::contains("fn main("));
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
fn transpile_fixture_print_interpolation_desert_expression_matches_expected() {
    assert_transpile_matches_fixture(
        "tests/fixtures/transpile_ok_print_interpolation_expr.ds",
        "tests/fixtures/transpile_ok_print_interpolation_expr.rs",
    );
}

#[test]
fn transpile_fixture_comprehensive_flow_matches_expected() {
    assert_transpile_matches_fixture(
        "tests/fixtures/transpile_ok_comprehensive_flow.ds",
        "tests/fixtures/transpile_ok_comprehensive_flow.rs",
    );
}

#[test]
fn transpile_fixture_operator_precedence_matches_expected() {
    assert_transpile_matches_fixture(
        "tests/fixtures/transpile_ok_operator_precedence.ds",
        "tests/fixtures/transpile_ok_operator_precedence.rs",
    );
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
fn graph_project_directory_without_manifest_uses_fallback_entry() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("graph").arg("tests/fixtures/project_no_manifest");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("src/util/math.ds\nsrc/main.ds\n"));
}

#[test]
fn graph_defaults_to_current_directory() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.current_dir("tests/fixtures/project_import_graph");
    cmd.arg("graph");
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

#[test]
fn run_file_executes_program() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("run").arg("examples/hello_world.ds");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Hello, Desert!"));
}

#[test]
fn run_project_executes_program() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("run").arg("tests/fixtures/project_ok");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("project ok"));
}

#[test]
fn run_project_directory_without_manifest_uses_fallback_entry() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("run").arg("tests/fixtures/project_no_manifest");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("7"));
}

#[test]
fn run_defaults_to_current_directory() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.current_dir("tests/fixtures/project_ok");
    cmd.arg("run");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("project ok"));
}

#[test]
fn run_reports_translated_diagnostics_for_compile_failure() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("run")
        .arg("tests/fixtures/check_fail_type_mismatch.ds");
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains("mismatched types"))
        .stderr(predicates::str::contains(
            "Rust compile failed with translated diagnostics.",
        ));
}

fn unique_temp_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos))
}

#[test]
fn new_scaffolds_project_that_checks() {
    let project_dir = unique_temp_path("desert_new_project");
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("new").arg(&project_dir);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Created Desert project"));

    let manifest = fs::read_to_string(project_dir.join("desert.toml")).unwrap();
    assert!(manifest.contains("entry = \"src/main.ds\""));
    let main_source = fs::read_to_string(project_dir.join("src/main.ds")).unwrap();
    assert!(main_source.contains("def main():"));

    let mut check_cmd = cargo_bin_cmd!("desert");
    check_cmd.arg("check").arg(&project_dir);
    check_cmd.assert().success();

    let _ = fs::remove_dir_all(&project_dir);
}

#[test]
fn new_rejects_non_empty_directory_without_force() {
    let project_dir = unique_temp_path("desert_new_nonempty");
    fs::create_dir_all(&project_dir).unwrap();
    fs::write(project_dir.join("existing.txt"), "keep").unwrap();

    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("new").arg(&project_dir);
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("is not empty"));

    let _ = fs::remove_dir_all(&project_dir);
}

#[test]
fn fmt_rewrites_unformatted_file() {
    let file = unique_temp_path("desert_fmt_file").with_extension("ds");
    fs::write(
        &file,
        "def main():\n    mut x=1\n    if x>0:\n        $print(\"ok\")\n",
    )
    .unwrap();

    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("fmt").arg(&file);
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Formatted 1 file(s)."));

    let formatted = fs::read_to_string(&file).unwrap();
    assert_eq!(
        formatted,
        "def main():\n    mut x = 1\n    if x > 0:\n        $print(\"ok\")\n"
    );

    let _ = fs::remove_file(&file);
}

#[test]
fn fmt_defaults_to_current_directory() {
    let dir = unique_temp_path("desert_fmt_dir");
    fs::create_dir_all(&dir).unwrap();
    let file = dir.join("main.ds");
    fs::write(&file, "def main():\n    mut x=1\n").unwrap();

    let mut cmd = cargo_bin_cmd!("desert");
    cmd.current_dir(&dir);
    cmd.arg("fmt");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("Formatted 1 file(s)."));

    let formatted = fs::read_to_string(&file).unwrap();
    assert_eq!(formatted, "def main():\n    mut x = 1\n");

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn fmt_check_fails_when_file_needs_formatting() {
    let file = unique_temp_path("desert_fmt_check").with_extension("ds");
    fs::write(&file, "def main():\n    mut x=1\n").unwrap();

    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("fmt").arg(&file).arg("--check");
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains(file.display().to_string()))
        .stderr(predicates::str::contains("format check failed"));

    let _ = fs::remove_file(&file);
}

#[test]
fn doctor_reports_environment_without_input() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("doctor");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("rustc:"))
        .stdout(predicates::str::contains("environment: ok"));
}

#[test]
fn doctor_validates_project_input() {
    let mut cmd = cargo_bin_cmd!("desert");
    cmd.arg("doctor").arg("tests/fixtures/project_ok");
    cmd.assert()
        .success()
        .stdout(predicates::str::contains("project: ok"));
}
