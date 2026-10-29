use super::*;
use crate::cfg::mouse_axis_program::parse_motion_program;
use crate::mouse_axis::program::*;
use crate::mouse_axis::{
    AXIS_COUNT, MouseAxis, MouseAxisMap, MouseAxisMapMode, MouseAxisTransform,
};
use MouseAxis::*;

fn domain(mode: MouseAxisMapMode) -> AxisDomain {
    AxisDomain::with_axes(mode, &MouseAxis::ALL)
}

/// Exercise the real defvar compiler, but keep the motion entrypoint internal:
/// `(program VALUE)` is only this test harness, not new configuration syntax.
fn parse_source(source: &str, domain: AxisDomain) -> Result<MotionProgram> {
    let forms = sexpr::parse(source, "motion-test.kbd")?;
    let vars = parse_vars(
        &forms
            .iter()
            .filter(|form| form.t[0].atom(None) == Some("defvar"))
            .map(|form| &form.t)
            .collect::<Vec<_>>(),
        &mut LspHints::default(),
    )?;
    let form = forms
        .iter()
        .find(|form| form.t[0].atom(None) == Some("program"))
        .expect("test program form");
    assert_eq!(form.t.len(), 2);
    parse_motion_program(&form.t[1], &vars, domain)
}

fn parse_program(source: &str, domain: AxisDomain) -> MotionProgram {
    parse_source(&format!("(program {source})"), domain)
        .unwrap_or_else(|error| panic!("{source}: {error:?}"))
}

fn map2() -> MouseAxisMap {
    let mut map = MouseAxisMap::default();
    map.rows[0][..2].copy_from_slice(&[1.0, 2.0]);
    map.rows[1][..2].copy_from_slice(&[3.0, 4.0]);
    map
}

fn block(map: &MouseAxisMap) -> [[f64; 2]; 2] {
    [
        [map.rows[0][0], map.rows[0][1]],
        [map.rows[1][0], map.rows[1][1]],
    ]
}

#[test]
fn cell_matrix_and_scalar_denote_the_same_selected_values() {
    let domain = domain(MouseAxisMapMode::Linear);
    for operation in ["set", "add", "mul-elements"] {
        let single = parse_program(&format!("(({operation} (cell x y 2)))"), domain)
            .apply(&map2())
            .unwrap();
        let matrix = parse_program(&format!("(({operation} (matrix (x (y 2)))))"), domain)
            .apply(&map2())
            .unwrap();
        let scalar = parse_program(
            &format!("(({operation} (scalar 2 (outputs x) (inputs y))))"),
            domain,
        )
        .apply(&map2())
        .unwrap();
        assert!(single.semantic_eq(&matrix));
        assert!(single.semantic_eq(&scalar));
    }
}

#[test]
fn single_nonuniform_matrix_operation_preserves_null_and_missing_cells() {
    let domain = domain(MouseAxisMapMode::Linear);
    for (operation, expected) in [
        ("set", [[2.0, 2.0], [-1.0, 0.5]]),
        ("add", [[3.0, 2.0], [2.0, 4.5]]),
        ("mul-elements", [[2.0, 2.0], [-3.0, 2.0]]),
    ] {
        let p = parse_program(
            &format!("(({operation} (matrix (x (x 2) (y null)) (y (x -1) (y 0.5)))))"),
            domain,
        );
        let result = p.apply(&map2()).unwrap();
        assert_eq!(block(&result), expected);
        assert!(result.output_eq(&map2(), V));
        let MotionOperation::Edit(_, operand) = &p.instructions()[0].operation else {
            panic!("edit")
        };
        assert_eq!(
            operand
                .cells()
                .iter()
                .filter(|entry| entry.value.is_none())
                .count(),
            1
        );
    }
}

#[test]
fn scalar_broadcast_is_a_cartesian_block_not_a_diagonal_factor() {
    let domain = domain(MouseAxisMapMode::Linear);
    for (operation, expected) in [
        ("set", [[2.0, 2.0], [2.0, 2.0]]),
        ("add", [[3.0, 4.0], [5.0, 6.0]]),
        ("mul-elements", [[2.0, 4.0], [6.0, 8.0]]),
    ] {
        let p = parse_program(
            &format!("(({operation} (scalar 2 (outputs x y) (inputs x y))))"),
            domain,
        );
        let result = p.apply(&map2()).unwrap();
        assert_eq!(block(&result), expected);
        assert!(result.output_eq(&map2(), H));
    }
    let result = parse_program("((add (scalar 0.1 (outputs x y) (inputs x y))))", domain)
        .apply(&MouseAxisMap::default())
        .unwrap();
    assert_eq!(block(&result), [[1.1, 0.1], [0.1, 1.1]]);
    let result = parse_program("((mul-elements (scalar 0.5)))", domain)
        .apply(&MouseAxisMap::default())
        .unwrap();
    assert_eq!(result.rows[0][1], 0.0);
    assert_eq!(result.rows[Misc as usize][Misc as usize], 0.5);
    assert_eq!(result.rows[V as usize][V as usize], 0.5);
}

#[test]
fn scalar_selectors_and_exclusions_are_independent_of_selector_order() {
    let domain = domain(MouseAxisMapMode::Linear);
    let output = parse_program("((mul-elements (scalar 0.5 (outputs x))))", domain)
        .apply(&map2())
        .unwrap();
    assert_eq!(block(&output), [[0.5, 1.0], [3.0, 4.0]]);
    let input = parse_program("((mul-elements (scalar 2 (inputs y))))", domain)
        .apply(&map2())
        .unwrap();
    assert_eq!(block(&input), [[1.0, 4.0], [3.0, 8.0]]);
    for selectors in [
        "(outputs x y) (inputs x y) (except (x y))",
        "(except (x y)) (inputs x y) (outputs x y)",
    ] {
        let p = parse_program(
            &format!(
                "((mul-elements (scalar 2 {selectors})) (add (cell x y 1)) (set (cell y y null)))"
            ),
            domain,
        );
        assert_eq!(block(&p.apply(&map2()).unwrap()), [[2.0, 3.0], [6.0, 8.0]]);
    }
    let p = parse_program(
        "((mul-elements (scalar 2 (outputs x y) (inputs x y))) (set (cell x y null)) (add (cell x y 1)))",
        domain,
    );
    assert_eq!(block(&p.apply(&map2()).unwrap()), [[2.0, 5.0], [6.0, 8.0]]);
}

#[test]
fn precision_boost_and_wheel_scaling_preserve_quarter_turn_routes() {
    let domain = domain(MouseAxisMapMode::Linear);
    let mut base = MouseAxisMap::default();
    base.rows[0][..2].copy_from_slice(&[0.0, 1.0]);
    base.rows[1][..2].copy_from_slice(&[-1.0, 0.0]);
    let precision = parse_program("((mul-elements (scalar 0.25 (outputs x y))))", domain);
    let boost = parse_program("((mul-elements (scalar 2 (outputs x y))))", domain);
    let wheel = parse_program("((mul-elements (scalar 2 (outputs v h))))", domain);
    for (programs, expected) in [
        (vec![], [4, -8, 120]),
        (vec![&precision], [1, -2, 120]),
        (vec![&boost], [8, -16, 120]),
        (vec![&precision, &boost], [2, -4, 120]),
        (vec![&precision, &wheel], [1, -2, 240]),
    ] {
        let map = programs
            .into_iter()
            .fold(base, |map, program| program.apply(&map).unwrap());
        let mut transform = MouseAxisTransform {
            map,
            ..Default::default()
        };
        let mut input = [0; AXIS_COUNT];
        input[..3].copy_from_slice(&[8, 4, 120]);
        assert_eq!(&transform.apply(input)[..3], &expected);
    }
}

#[test]
fn sparse_set_and_explicit_clear_then_fill_are_different() {
    let domain = domain(MouseAxisMapMode::Linear);
    let sparse = parse_program("((set (matrix (x (y 7)))))", domain)
        .apply(&map2())
        .unwrap();
    let full = parse_program("((set (scalar 0)) (set (matrix (x (y 7)))))", domain)
        .apply(&map2())
        .unwrap();
    assert_eq!(block(&sparse), [[1.0, 7.0], [3.0, 4.0]]);
    assert_eq!(block(&full), [[0.0, 7.0], [0.0, 0.0]]);
    assert_eq!(sparse.rows[Misc as usize][Misc as usize], 1.0);
    assert_eq!(full.rows[Misc as usize][Misc as usize], 0.0);
}

#[test]
fn empty_null_and_bare_matrix_rows_never_reset_the_inherited_map() {
    let domain = domain(MouseAxisMapMode::Linear);
    for source in [
        "null",
        "()",
        "((set (matrix)))",
        "((set (matrix (x) (y null))))",
        "((add (scalar null)))",
    ] {
        assert!(
            parse_program(source, domain)
                .apply(&map2())
                .unwrap()
                .semantic_eq(&map2()),
            "{source}"
        );
    }
}

#[test]
fn four_axis_domain_never_edits_hidden_native_storage() {
    // Runs on Linux too: native scalar defaults must not touch phantom Windows
    // Z/RX/RY/RZ/dial/misc outputs or columns.
    for mode in [MouseAxisMapMode::Linear, MouseAxisMapMode::Directional] {
        let domain = AxisDomain::with_axes(mode, &[X, Y, V, H]);
        let base = MouseAxisMap {
            mode,
            ..Default::default()
        };
        let p = parse_program("((set (scalar 2)))", domain);
        let result = p.apply(&base).unwrap();
        for output in MouseAxis::ALL {
            for input in MouseAxis::ALL {
                let visible = domain.supports(output) && domain.supports(input);
                assert_eq!(
                    result.rows[output as usize][input as usize],
                    if visible {
                        2.0
                    } else {
                        base.rows[output as usize][input as usize]
                    }
                );
                if mode == MouseAxisMapMode::Directional {
                    assert_eq!(
                        result.negative_rows[output as usize][input as usize],
                        if visible {
                            2.0
                        } else {
                            base.negative_rows[output as usize][input as usize]
                        }
                    );
                }
            }
        }
        let p = parse_program(
            "((compose-output (factor (from x) (to y) (values (3)))))",
            domain,
        );
        let result = p.apply(&base).unwrap();
        assert!(result.output_eq(&base, Z));
        for axis in [Z, Rx, Ry, Rz, Dial, Misc] {
            assert!(!result.maps_input(axis));
        }
        let input_source = if mode == MouseAxisMapMode::Directional {
            "((set (cell x z+ 2)))"
        } else {
            "((set (cell x z 2)))"
        };
        for source in [
            "((set (cell z x null)))",
            input_source,
            "((add (scalar 2 (outputs dial))))",
            "((compose-output (factor (from z) (to x) (values (1)))))",
        ] {
            let error = parse_source(&format!("(program {source})"), domain).unwrap_err();
            assert!(error.msg.contains("unsupported"), "{error:?}");
        }
    }
}

#[test]
fn native_domain_follows_backend_support_and_canonical_order() {
    for mode in [MouseAxisMapMode::Linear, MouseAxisMapMode::Directional] {
        let domain = AxisDomain::native(mode);
        let axes = domain.outputs().collect::<Vec<_>>();
        assert_eq!(
            axes,
            MouseAxis::ALL
                .into_iter()
                .filter(|axis| axis.supported())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            domain.inputs().count(),
            axes.len()
                * if mode == MouseAxisMapMode::Linear {
                    1
                } else {
                    2
                }
        );
    }
}

#[test]
fn factor_named_axes_preserve_user_value_order_and_complete_rectangular_rows() {
    let domain = domain(MouseAxisMapMode::Linear);
    let p = parse_program(
        "((compose-output (factor (values (3 2)) (to z) (from y x))))",
        domain,
    );
    let result = p.apply(&map2()).unwrap();
    assert_eq!(result.rows[Z as usize][X as usize], 11.0);
    assert_eq!(result.rows[Z as usize][Y as usize], 16.0);
    assert_eq!(result.rows[Z as usize][Z as usize], 0.0);
    assert!(result.output_eq(&map2(), X));
    assert!(result.output_eq(&map2(), Misc));
    // To order is equally significant: first row is Y, second X.
    let p = parse_program(
        "((compose-output (factor (from x) (to y x) (values (2) (3)))))",
        domain,
    );
    let result = p.apply(&map2()).unwrap();
    assert_eq!(block(&result), [[3.0, 6.0], [2.0, 4.0]]);
}

#[test]
fn parser_preserves_the_distinct_left_right_and_elementwise_meanings() {
    let domain = domain(MouseAxisMapMode::Linear);
    let factor = "(factor (from x y) (to x y) (values (1 1) (0 1)))";
    let left = parse_program(&format!("((compose-output {factor}))"), domain)
        .apply(&map2())
        .unwrap();
    let right = parse_program(&format!("((compose-input {factor}))"), domain)
        .apply(&map2())
        .unwrap();
    let elements = parse_program(
        "((mul-elements (matrix (x (x 1) (y 1)) (y (x 0) (y 1)))))",
        domain,
    )
    .apply(&map2())
    .unwrap();
    assert_eq!(block(&left), [[4.0, 6.0], [3.0, 4.0]]);
    assert_eq!(block(&right), [[1.0, 3.0], [3.0, 7.0]]);
    assert_eq!(block(&elements), [[1.0, 2.0], [0.0, 4.0]]);
}

#[test]
fn directional_selectors_name_explicit_branches_and_post_compose_both() {
    let domain = domain(MouseAxisMapMode::Directional);
    let base = MouseAxisMap {
        mode: MouseAxisMapMode::Directional,
        ..Default::default()
    };
    let p = parse_program(
        "((compose-output (factor (from x y) (to x y) (values (1 1) (0 1)))) (set (cell x x- 0.5)) (mul-elements (scalar 2 (inputs y-) (except (x y-)))))",
        domain,
    );
    let result = p.apply(&base).unwrap();
    let mut transform = MouseAxisTransform {
        map: result,
        ..Default::default()
    };
    let mut input = [0; AXIS_COUNT];
    input[..2].copy_from_slice(&[-2, 3]);
    assert_eq!(&transform.apply(input)[..2], &[4, 3]);
    input[..2].copy_from_slice(&[2, -3]);
    assert_eq!(&transform.apply(input)[..2], &[-1, -6]);
}

#[test]
fn variables_supply_whole_programs_instructions_operands_operators_and_values() {
    let source = "(defvar
        output x input y number 7 value $number nil null operator set
        operand (cell $output $input $value)
        instruction ($operator $operand) alias $instruction
        selection (outputs $output)
        scale (mul-elements (scalar 2 $selection (inputs $input)))
        instructions ($alias $scale (set (cell x y $nil)))
        whole $instructions)
        (program $whole)";
    // All Rc-backed input expressions/variables are dropped inside parse_source.
    let p = parse_source(source, domain(MouseAxisMapMode::Linear)).unwrap();
    assert_eq!(p.apply(&map2()).unwrap().rows[0][1], 14.0);
    let source_ref = &p.instructions()[0].source;
    assert_eq!(
        source_ref.program.start.byte,
        source.rfind("$whole").unwrap()
    );
    assert_eq!(
        source_ref.use_site.start.byte,
        source.find("$alias").unwrap()
    );
    assert_eq!(
        source_ref.definition.as_ref().unwrap().start.byte,
        source.find("($operator").unwrap()
    );
    assert_eq!(source_ref.program.file_name.as_ref(), "motion-test.kbd");
}

#[test]
fn variables_resolve_at_matrix_rows_entries_null_rows_and_scalar_exclusions() {
    let source = "(defvar
        axis x other y value 2 nil null
        pair ($other $value) row ($axis $pair) nullrow (y $nil)
        matrix (matrix $row $nullrow)
        hole ($axis $other) exclusions (except $hole)
        scalar (scalar 3 (outputs $axis) (inputs $other) $exclusions))
        (program ((set $matrix) (add $scalar)))";
    let result = parse_source(source, domain(MouseAxisMapMode::Linear))
        .unwrap()
        .apply(&map2())
        .unwrap();
    assert_eq!(block(&result), [[1.0, 2.0], [3.0, 4.0]]);
    let source =
        "(defvar branch x- val 0.5 selection (inputs $branch) scalar (scalar $val $selection))
                  (program ((mul-elements $scalar)))";
    let base = MouseAxisMap {
        mode: MouseAxisMapMode::Directional,
        ..Default::default()
    };
    let result = parse_source(source, domain(MouseAxisMapMode::Directional))
        .unwrap()
        .apply(&base)
        .unwrap();
    assert_eq!(result.negative_rows[0][0], -0.5);
    assert_eq!(result.rows[0][0], 1.0);
}

#[test]
fn variables_resolve_at_factor_fields_rows_axes_and_numbers() {
    let source = "(defvar
        a x b y from (from $a $b) to (to $a)
        two 2 three 3 row ($two $three) values (values $row)
        operand (factor $from $to $values)
        instruction (compose-output $operand))
        (program ($instruction))";
    let p = parse_source(source, domain(MouseAxisMapMode::Linear)).unwrap();
    assert_eq!(
        block(&p.apply(&map2()).unwrap()),
        [[11.0, 16.0], [3.0, 4.0]]
    );
    let p = parse_source(
        "(defvar n null alias $n) (program $alias)",
        domain(MouseAxisMapMode::Linear),
    )
    .unwrap();
    assert!(p.instructions().is_empty());
}

#[test]
fn source_locations_use_zero_based_bytes_lines_and_columns() {
    let source = "(program (\n  (set (cell x y 2))\n))";
    let p = parse_source(source, domain(MouseAxisMapMode::Linear)).unwrap();
    let source = &p.instructions()[0].source;
    assert_eq!(
        source.use_site.start,
        SourcePosition {
            byte: 13,
            line: 1,
            column: 2
        }
    );
    assert_eq!(source.use_site.end.line, 1);
    assert!(source.definition.is_none());
}

#[test]
fn malformed_and_duplicate_operands_have_source_spanned_errors() {
    let domain = domain(MouseAxisMapMode::Linear);
    for (source, expected) in [
        ("7", "instruction list"),
        ("(())", "nonempty"),
        ("((set))", "exactly one"),
        ("((set (cell x x 1) (cell y y 2)))", "exactly one"),
        ("((mul (cell x x 1)))", "unknown mouse-axis operation"),
        ("((set (cell x x)))", "OUTPUT INPUT VALUE"),
        ("((set (cell x x (add 2))))", "finite number or null"),
        ("((set (cell x+ x 1)))", "unknown mouse axis"),
        ("((set (cell x x+ 1)))", "unknown mouse axis"),
        ("((set (cell nope x 1)))", "unknown mouse axis"),
        (
            "((set (factor (from x) (to y) (values (1)))))",
            "coefficient operand",
        ),
        ("((set (matrix ())))", "needs an output"),
        ("((set (matrix (x) (x (y 1)))))", "duplicate matrix output"),
        (
            "((set (matrix (x (y null) (y 1)))))",
            "duplicate matrix input",
        ),
        ("((set (matrix (x (y 1 2)))))", "INPUT VALUE"),
        ("((set (matrix (x null (y 1)))))", "matrix entry"),
        ("((add (scalar)))", "requires a value"),
        ("((add (scalar 1 (outputs))))", "nonempty"),
        ("((add (scalar 1 (inputs))))", "nonempty"),
        ("((add (scalar 1 (outputs x x))))", "duplicate outputs axis"),
        ("((add (scalar 1 (inputs x x))))", "duplicate inputs axis"),
        (
            "((add (scalar 1 (outputs x) (outputs y))))",
            "duplicate scalar selector",
        ),
        (
            "((add (scalar 1 (inputs x) (inputs y))))",
            "duplicate scalar selector",
        ),
        (
            "((add (scalar 1 (except) (except))))",
            "duplicate scalar selector",
        ),
        (
            "((add (scalar 1 (except (x x) (x x)))))",
            "duplicate exclusion",
        ),
        (
            "((add (scalar 1 (outputs x) (except (y x)))))",
            "outside the scalar selection",
        ),
        (
            "((add (scalar 1 (inputs y) (except (x x)))))",
            "outside the scalar selection",
        ),
        ("((add (scalar 1 (except (x)))))", "OUTPUT INPUT"),
        ("((add (scalar 1 (except (x y 1)))))", "OUTPUT INPUT"),
        ("((add (scalar 1 (rows x))))", "unknown scalar selector"),
    ] {
        let source = format!("(program {source})");
        let error = parse_source(&source, domain).unwrap_err();
        assert!(
            error.msg.contains(expected),
            "{source}: expected {expected}: {error:?}"
        );
        let span = error.span.expect("source diagnostic");
        assert_eq!(span.file_name.as_ref(), "motion-test.kbd");
        assert!(span.end() > span.start());
        assert!(span.end() <= source.len());
    }
}

#[test]
fn malformed_factors_reject_null_dimensions_duplicates_and_wrong_types() {
    let domain = domain(MouseAxisMapMode::Linear);
    for (factor, expected) in [
        ("(matrix (x (x 1)))", "numeric factor"),
        ("(factor)", "requires from"),
        ("(factor (from x))", "requires to"),
        ("(factor (from x) (to y))", "requires values"),
        ("(factor (from) (to y) (values (1)))", "nonempty"),
        ("(factor (from x) (to) (values))", "nonempty"),
        (
            "(factor (from x x) (to y) (values (1 2)))",
            "duplicate from",
        ),
        (
            "(factor (from x) (to y y) (values (1) (2)))",
            "duplicate to",
        ),
        (
            "(factor (from x) (from y) (to y) (values (1)))",
            "duplicate factor field",
        ),
        (
            "(factor (from x) (to y) (values (1)) (values (2)))",
            "duplicate factor field",
        ),
        (
            "(factor (from x) (to y) (values (1)) (unknown 1))",
            "unknown factor field",
        ),
        ("(factor (from x y) (to y) (values (1)))", "2 columns"),
        ("(factor (from x) (to y) (values (1 2)))", "1 columns"),
        ("(factor (from x) (to y x) (values (1)))", "2 rows"),
        ("(factor (from x) (to y) (values 1))", "numeric factor row"),
        (
            "(factor (from x) (to y) (values (null)))",
            "null is not allowed",
        ),
        (
            "(factor (from x) (to y) (values ((add 1))))",
            "finite number",
        ),
    ] {
        let error =
            parse_source(&format!("(program ((compose-output {factor})))"), domain).unwrap_err();
        assert!(
            error.msg.contains(expected),
            "{factor}: expected {expected}: {error:?}"
        );
        assert!(error.span.is_some());
    }
}

#[test]
fn directional_precomposition_and_ambiguous_branch_names_are_rejected() {
    let domain = domain(MouseAxisMapMode::Directional);
    for (source, expected) in [
        (
            "((compose-input (factor (from x) (to x) (values (1)))))",
            "compose-input is not supported",
        ),
        ("((set (cell x x 2)))", "explicit + or - branch"),
        ("((set (scalar 2 (inputs x))))", "explicit + or - branch"),
        ("((set (matrix (x (y 1)))))", "explicit + or - branch"),
        (
            "((compose-output (factor (from x+) (to x) (values (1)))))",
            "unknown mouse axis",
        ),
        (
            "((set (matrix (x (y+ 1) (y+ null)))))",
            "duplicate matrix input",
        ),
    ] {
        let error = parse_source(&format!("(program {source})"), domain).unwrap_err();
        assert!(error.msg.contains(expected), "{source}: {error:?}");
    }
}

#[test]
fn nonfinite_literals_are_rejected_at_the_actual_value_span() {
    let domain = domain(MouseAxisMapMode::Linear);
    for value in ["NaN", "inf", "-inf", "1e999"] {
        for operand in [
            format!("(cell x y {value})"),
            format!("(matrix (x (y {value})))"),
            format!("(scalar {value})"),
        ] {
            let source = format!("(program ((add {operand})))");
            let error = parse_source(&source, domain).unwrap_err();
            assert!(error.msg.contains("finite number"));
            assert_eq!(&source[error.span.unwrap()], value);
        }
        let source =
            format!("(program ((compose-output (factor (from x) (to y) (values ({value}))))))");
        let error = parse_source(&source, domain).unwrap_err();
        assert!(error.msg.contains("finite number"));
        assert_eq!(&source[error.span.unwrap()], value);
    }
}

#[test]
fn unknown_and_cyclic_variable_errors_are_preserved_at_nested_boundaries() {
    let domain = domain(MouseAxisMapMode::Linear);
    for source in [
        "(program $missing)",
        "(program ($missing))",
        "(program ((set $missing)))",
        "(program ((set (cell x x $missing))))",
        "(program ((add (scalar 1 $missing))))",
        "(program ((compose-output (factor (from x) (to y) (values $missing)))))",
    ] {
        let error = parse_source(source, domain).unwrap_err();
        assert!(
            error.msg.contains("unknown mouse-axis variable $missing"),
            "{error:?}"
        );
        assert_eq!(&source[error.span.unwrap()], "$missing");
    }
    for source in [
        "(defvar a $b b $a) (program $a)",
        "(defvar a $a) (program ((add (scalar $a))))",
        "(defvar a $b b $a) (program ((set $a)))",
    ] {
        let error = parse_source(source, domain).unwrap_err();
        assert!(
            error.msg.contains("cyclic mouse-axis variable"),
            "{error:?}"
        );
        assert!(error.span.is_some());
    }
}

#[test]
fn inherited_context_is_evaluated_only_when_the_program_is_applied() {
    let domain = domain(MouseAxisMapMode::Linear);
    let p = parse_program("((mul-elements (cell x x 1e308)))", domain);
    let mut inherited = MouseAxisMap::default();
    inherited.rows[0][0] = 1e-308;
    assert!((p.apply(&inherited).unwrap().rows[0][0] - 1.0).abs() < 1e-15);
    inherited.rows[0][0] = 1e308;
    let error = p.apply(&inherited).unwrap_err();
    assert_eq!(error.instruction_index, Some(0));
    assert_eq!(
        error.source.unwrap().use_site.file_name.as_ref(),
        "motion-test.kbd"
    );
}

#[test]
fn public_layer_headers_resolve_nested_variables_and_preserve_icons() {
    let cfg = parse_cfg(
        r#"
      (defvar name base image "base.png" axis x value 2
        operand (cell $axis y $value) instruction (add $operand)
        program ($instruction) header ($name 🖼 $image mouse-axis $program))
      (defsrc a)
      (deflayer $header a)
      (deflayermap (second icon second.ico mouse-axis ((mul-elements (scalar 3 (outputs x))))) a b)
    "#,
    )
    .unwrap();
    assert_eq!(cfg.layer_info[0].name, "base");
    assert_eq!(cfg.layer_info[0].icon.as_deref(), Some("base.png"));
    assert_eq!(
        resolve_motion(&cfg.options.mouse_axis_map, &cfg.layer_info, &[0, 1])
            .unwrap()
            .rows[0][1],
        6.0
    );
    for header in [
        "mouse-axis () mouse-axis null",
        "icon a 🖼 b",
        "mouse-axis",
        "unknown ()",
    ] {
        assert!(parse_cfg(&format!("(defsrc) (deflayer (base {header}))")).is_err());
    }
}

#[test]
fn initial_arithmetic_is_checked_but_dormant_programs_keep_their_inherited_context() {
    let error = parse_cfg("(defcfg mouse-axis-map ((x (x 1e308)))) (defsrc) (deflayer (base mouse-axis ((mul-elements (cell x x 2)))))").unwrap_err();
    assert!(
        error.msg.contains("initial mouse-axis resolution"),
        "{error:?}"
    );
    assert!(error.msg.contains("instruction 1"), "{error:?}");
    let cfg = parse_cfg("(defcfg mouse-axis-map ((x (x 1e308)))) (defsrc) (deflayer base) (deflayer (lower mouse-axis ((set (cell x x 1e-308))))) (deflayer (upper mouse-axis ((mul-elements (cell x x 1e308)))))").unwrap();
    assert!(resolve_motion(&cfg.options.mouse_axis_map, &cfg.layer_info, &[0, 2]).is_err());
    assert!(resolve_motion(&cfg.options.mouse_axis_map, &cfg.layer_info, &[0, 1, 2]).is_ok());
}

#[test]
fn capture_analysis_uses_all_declarations_but_not_empty_or_null_edits() {
    let cfg = parse_cfg("(defsrc) (deflayer (base mouse-axis null)) (deflayer (idle mouse-axis ((set (matrix (x null))) (add (scalar null)))))").unwrap();
    assert!(
        !CaptureRequirements::for_programs(
            &cfg.options.mouse_axis_map,
            cfg.layer_info.iter().map(|l| &l.mouse_axis)
        )
        .any()
    );
    for instruction in [
        "(set (cell x x 0))",
        "(add (cell x y 0))",
        "(mul-elements (cell x y 2))",
        "(compose-input (factor (from x) (to y) (values (1))))",
    ] {
        let cfg = parse_cfg(&format!(
            "(defsrc) (deflayer base) (deflayer (inactive mouse-axis ({instruction})))"
        ))
        .unwrap();
        let required = CaptureRequirements::for_programs(
            &cfg.options.mouse_axis_map,
            cfg.layer_info.iter().map(|l| &l.mouse_axis),
        );
        assert!(required.movement() && required.wheel());
        for axis in MouseAxis::ALL {
            assert_eq!(required.inputs[axis as usize], axis.supported());
        }
    }
}

#[cfg(feature = "lsp")]
#[test]
fn lsp_variable_references_include_nested_program_boundaries() {
    LSP_VARIABLE_REFERENCES.with_borrow_mut(|refs| refs.0.clear());
    let source = "(defvar axis x val 2 operand (cell $axis y $val) inst (add $operand) whole ($inst)) (program $whole)";
    parse_source(source, domain(MouseAxisMapMode::Linear)).unwrap();
    LSP_VARIABLE_REFERENCES.with_borrow(|refs| {
        for name in ["axis", "val", "operand", "inst", "whole"] {
            assert!(
                refs.0
                    .get(name)
                    .is_some_and(|locations| !locations.is_empty()),
                "missing {name}"
            );
        }
    });
}

#[test]
fn on_demand_cell_explanation_replays_order_and_product_contributions() {
    let program = parse_program(
        "((add (cell x x 1)) (compose-output (factor (from x y) (to x) (values (1 1)))))",
        domain(MouseAxisMapMode::Linear),
    );
    let trace = program
        .explain_cell(
            &map2(),
            crate::mouse_axis::program::Cell {
                output: X,
                input: AxisInput {
                    axis: X,
                    branch: InputBranch::Signed,
                },
            },
        )
        .unwrap();
    assert_eq!((trace[0].before, trace[0].after), (1.0, 2.0));
    assert_eq!((trace[1].before, trace[1].after), (2.0, 5.0));
    assert_eq!(trace[1].operation, "compose-output");
    assert_eq!(trace[1].products.len(), AXIS_COUNT);
    assert_eq!(
        trace[1].products[0],
        ProductContribution {
            via: X,
            left: 1.0,
            right: 2.0
        }
    );
    assert_eq!(
        trace[1].products[1],
        ProductContribution {
            via: Y,
            left: 1.0,
            right: 3.0
        }
    );
    assert_eq!(
        trace[1].source.use_site.file_name.as_ref(),
        "motion-test.kbd"
    );
}
