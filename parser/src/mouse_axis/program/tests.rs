use super::*;
use crate::mouse_axis::MouseAxisTransform;
use MouseAxis::*;

fn domain(mode: MouseAxisMapMode) -> AxisDomain {
    AxisDomain::with_axes(mode, &MouseAxis::ALL)
}

fn cell(output: MouseAxis, axis: MouseAxis, branch: InputBranch) -> Cell {
    Cell {
        output,
        input: AxisInput { axis, branch },
    }
}

fn linear_cell(output: MouseAxis, input: MouseAxis) -> Cell {
    cell(output, input, InputBranch::Signed)
}

fn edit(domain: AxisDomain, op: EditOperation, entries: &[(Cell, Option<f64>)]) -> MotionOperation {
    MotionOperation::Edit(
        op,
        CoefficientOperand::new(
            domain,
            entries
                .iter()
                .map(|&(cell, value)| CellValue { cell, value })
                .collect(),
        )
        .unwrap(),
    )
}

fn program(domain: AxisDomain, operations: Vec<MotionOperation>) -> MotionProgram {
    MotionProgram::new(
        domain,
        operations
            .into_iter()
            .map(|operation| MotionInstruction {
                operation,
                source: InstructionSource::default(),
            })
            .collect(),
    )
    .unwrap()
}

fn factor(domain: AxisDomain, values: [[f64; 2]; 2]) -> NumericFactor {
    NumericFactor::from_block(domain, vec![X, Y], vec![X, Y], values.map(Vec::from).into()).unwrap()
}

fn map2(values: [[f64; 2]; 2]) -> MouseAxisMap {
    let mut map = MouseAxisMap::default();
    for (output, values) in values.iter().enumerate() {
        map.rows[output][..2].copy_from_slice(values);
    }
    map
}

fn block(map: &MouseAxisMap) -> [[f64; 2]; 2] {
    [
        [map.rows[0][0], map.rows[0][1]],
        [map.rows[1][0], map.rows[1][1]],
    ]
}

#[test]
fn individual_cell_and_nonuniform_matrix_examples() {
    let domain = domain(MouseAxisMapMode::Linear);
    let base = map2([[1.0, 2.0], [3.0, 4.0]]);
    for (op, value, expected) in [
        (EditOperation::Set, 9.0, [[1.0, 9.0], [3.0, 4.0]]),
        (EditOperation::Add, -0.5, [[1.0, 1.5], [3.0, 4.0]]),
        (EditOperation::MulElements, 0.5, [[1.0, 1.0], [3.0, 4.0]]),
    ] {
        let p = program(
            domain,
            vec![edit(domain, op, &[(linear_cell(X, Y), Some(value))])],
        );
        assert_eq!(block(&p.apply(&base).unwrap()), expected);
        let null = program(domain, vec![edit(domain, op, &[(linear_cell(X, Y), None)])]);
        assert!(null.apply(&base).unwrap().semantic_eq(&base));
    }
    for (op, expected) in [
        (EditOperation::Set, [[2.0, 2.0], [-1.0, 0.5]]),
        (EditOperation::Add, [[3.0, 2.0], [2.0, 4.5]]),
        (EditOperation::MulElements, [[2.0, 2.0], [-3.0, 2.0]]),
    ] {
        let entries = [
            (linear_cell(X, X), Some(2.0)),
            (linear_cell(Y, X), Some(-1.0)),
            (linear_cell(Y, Y), Some(0.5)),
            (linear_cell(X, Y), None),
        ];
        let explicit_null = program(domain, vec![edit(domain, op, &entries)])
            .apply(&base)
            .unwrap();
        let omitted = program(domain, vec![edit(domain, op, &entries[..3])])
            .apply(&base)
            .unwrap();
        assert_eq!(block(&explicit_null), expected);
        assert!(omitted.semantic_eq(&explicit_null));
        // A genuinely zero operand is not a no-edit marker for set/multiply.
        let zero = program(
            domain,
            vec![edit(domain, op, &[(linear_cell(X, Y), Some(0.0))])],
        )
        .apply(&base)
        .unwrap();
        assert_eq!(
            zero.rows[0][1],
            if op == EditOperation::Add { 2.0 } else { 0.0 }
        );
    }
}

#[test]
fn edits_read_preceding_results_and_null_does_not_restore() {
    let domain = domain(MouseAxisMapMode::Linear);
    let base = map2([[1.0, 2.0], [3.0, 4.0]]);
    let times_two = edit(
        domain,
        EditOperation::MulElements,
        &[
            (linear_cell(X, X), Some(2.0)),
            (linear_cell(X, Y), Some(2.0)),
            (linear_cell(Y, X), Some(2.0)),
            (linear_cell(Y, Y), Some(2.0)),
        ],
    );
    let plus_one = edit(
        domain,
        EditOperation::Add,
        &[(linear_cell(X, Y), Some(1.0))],
    );
    let null = edit(domain, EditOperation::Set, &[(linear_cell(Y, Y), None)]);
    assert_eq!(
        block(
            &program(domain, vec![times_two.clone(), plus_one.clone(), null])
                .apply(&base)
                .unwrap()
        ),
        [[2.0, 5.0], [6.0, 8.0]]
    );
    assert_eq!(
        block(
            &program(domain, vec![plus_one, times_two.clone()])
                .apply(&base)
                .unwrap()
        ),
        [[2.0, 6.0], [6.0, 8.0]]
    );
    let set = edit(
        domain,
        EditOperation::Set,
        &[(linear_cell(X, Y), Some(7.0))],
    );
    assert_eq!(
        program(domain, vec![times_two.clone(), set.clone()])
            .apply(&base)
            .unwrap()
            .rows[0][1],
        7.0
    );
    assert_eq!(
        program(domain, vec![set, times_two])
            .apply(&base)
            .unwrap()
            .rows[0][1],
        14.0
    );
}

#[test]
fn left_and_right_products_are_not_elementwise_multiplication() {
    let domain = domain(MouseAxisMapMode::Linear);
    let rotation = map2([[0.0, 1.0], [-1.0, 0.0]]);
    let scale = factor(domain, [[2.0, 0.0], [0.0, 1.0]]);
    for (op, expected, motion) in [
        (
            MotionOperation::ComposeOutput(scale.clone()),
            [[0.0, 2.0], [-1.0, 0.0]],
            [6, -2],
        ),
        (
            MotionOperation::ComposeInput(scale),
            [[0.0, 1.0], [-2.0, 0.0]],
            [3, -4],
        ),
    ] {
        let result = program(domain, vec![op]).apply(&rotation).unwrap();
        assert_eq!(block(&result), expected);
        let mut transform = MouseAxisTransform {
            map: result,
            ..Default::default()
        };
        let mut input = [0; AXIS_COUNT];
        input[..2].copy_from_slice(&[2, 3]);
        assert_eq!(&transform.apply(input)[..2], &motion);
    }
    let r = factor(domain, [[0.0, 1.0], [-1.0, 0.0]]);
    let d = factor(domain, [[2.0, 0.0], [0.0, 1.0]]);
    let identity = MouseAxisMap::default();
    let output = program(
        domain,
        vec![
            MotionOperation::ComposeOutput(r.clone()),
            MotionOperation::ComposeOutput(d.clone()),
        ],
    )
    .apply(&identity)
    .unwrap();
    let input = program(
        domain,
        vec![
            MotionOperation::ComposeInput(r),
            MotionOperation::ComposeInput(d),
        ],
    )
    .apply(&identity)
    .unwrap();
    assert_eq!(block(&output), [[0.0, 2.0], [-1.0, 0.0]]);
    assert_eq!(block(&input), [[0.0, 1.0], [-2.0, 0.0]]);
}

#[test]
fn products_read_old_rows_and_columns_not_partially_written_results() {
    let domain = domain(MouseAxisMapMode::Linear);
    let base = map2([[1.0, 2.0], [3.0, 4.0]]);
    let q = factor(domain, [[1.0, 1.0], [2.0, 3.0]]);
    let left = program(domain, vec![MotionOperation::ComposeOutput(q.clone())])
        .apply(&base)
        .unwrap();
    let right = program(domain, vec![MotionOperation::ComposeInput(q)])
        .apply(&base)
        .unwrap();
    assert_eq!(block(&left), [[4.0, 6.0], [11.0, 16.0]]);
    assert_eq!(block(&right), [[5.0, 7.0], [11.0, 15.0]]);
    assert_eq!(block(&base), [[1.0, 2.0], [3.0, 4.0]]);
}

#[test]
fn products_match_unquantized_stages_followed_by_one_quantization() {
    let domain = domain(MouseAxisMapMode::Linear);
    let base = map2([[0.5, 0.25], [-0.5, 0.75]]);
    let q = factor(domain, [[0.5, 1.0], [-0.25, 0.5]]);
    let p = factor(domain, [[1.0, 0.5], [-0.5, 0.25]]);
    let output = program(domain, vec![MotionOperation::ComposeOutput(q)])
        .apply(&base)
        .unwrap();
    let input = program(domain, vec![MotionOperation::ComposeInput(p)])
        .apply(&base)
        .unwrap();
    let inherited_motion = |[x, y]: [f64; 2]| [0.5 * x + 0.25 * y, -0.5 * x + 0.75 * y];
    let output_stage = |[x, y]: [f64; 2]| [0.5 * x + y, -0.25 * x + 0.5 * y];
    let input_stage = |[x, y]: [f64; 2]| [x + 0.5 * y, -0.5 * x + 0.25 * y];
    for motion in [[3, 0], [7, -2], [-5, 3]] {
        for (map, expected) in [
            (
                output,
                output_stage(inherited_motion(motion.map(f64::from))),
            ),
            (input, inherited_motion(input_stage(motion.map(f64::from)))),
        ] {
            let mut transform = MouseAxisTransform {
                map,
                ..Default::default()
            };
            let mut vector = [0; AXIS_COUNT];
            vector[..2].copy_from_slice(&motion);
            assert_eq!(
                &transform.apply(vector)[..2],
                &expected.map(|value| value.trunc() as i32)
            );
        }
    }
    // Rounding the first stage would lose motion: C(3,0) = (1.5,-1.5),
    // then Q gives (-0.75,-1.125), whose Y output is -1 rather than zero.
    let early = output_stage(inherited_motion([3.0, 0.0]).map(f64::trunc));
    assert_eq!(early.map(|value| value.trunc() as i32), [0, 0]);
}

#[test]
fn named_rectangular_factor_preserves_unlisted_rows_and_clears_listed_rows() {
    let domain = domain(MouseAxisMapMode::Linear);
    let mut base = MouseAxisMap::default();
    base.rows[X as usize] = [0.0; AXIS_COUNT];
    base.rows[X as usize][V as usize] = 1.0;
    base.rows[Y as usize] = [0.0; AXIS_COUNT];
    base.rows[Y as usize][H as usize] = 1.0;
    base.rows[Z as usize] = [0.0; AXIS_COUNT];
    base.rows[Z as usize][V as usize] = 1.0;
    base.rows[Z as usize][H as usize] = 1.0;
    base.rows[Z as usize][Dial as usize] = 9.0;
    // Deliberately reversed from order: values must not be silently axis-sorted.
    let q = NumericFactor::from_block(domain, vec![Y, X], vec![Z], vec![vec![3.0, 2.0]]).unwrap();
    assert_eq!(q.from_axes(), &[Y, X]);
    assert_eq!(q.to_axes(), &[Z]);
    let result = program(domain, vec![MotionOperation::ComposeOutput(q)])
        .apply(&base)
        .unwrap();
    assert_eq!(result.rows[X as usize], base.rows[X as usize]);
    assert_eq!(result.rows[Y as usize], base.rows[Y as usize]);
    assert_eq!(result.rows[Z as usize][V as usize], 2.0);
    assert_eq!(result.rows[Z as usize][H as usize], 3.0);
    assert_eq!(result.rows[Z as usize][Dial as usize], 0.0);
    assert_eq!(result.rows[Misc as usize], base.rows[Misc as usize]);
    let mut transform = MouseAxisTransform {
        map: result,
        ..Default::default()
    };
    let mut input = [0; AXIS_COUNT];
    input[V as usize] = 2;
    input[H as usize] = 4;
    let output = transform.apply(input);
    assert_eq!(
        [output[X as usize], output[Y as usize], output[Z as usize]],
        [2, 4, 16]
    );
}

#[test]
fn directional_output_products_apply_to_both_magnitude_branches() {
    let domain = domain(MouseAxisMapMode::Directional);
    let identity = MouseAxisMap {
        mode: MouseAxisMapMode::Directional,
        ..Default::default()
    };
    let q = factor(domain, [[1.0, 1.0], [0.0, 1.0]]);
    let shear = MotionOperation::ComposeOutput(q.clone());
    let result = program(domain, vec![shear.clone()])
        .apply(&identity)
        .unwrap();
    assert_eq!(block(&result), [[1.0, 1.0], [0.0, 1.0]]);
    assert_eq!(&result.negative_rows[0][..2], &[-1.0, -1.0]);
    let mut input = [0; AXIS_COUNT];
    input[..2].copy_from_slice(&[-2, 3]);
    let mut transform = MouseAxisTransform {
        map: result,
        ..Default::default()
    };
    assert_eq!(&transform.apply(input)[..2], &[1, 3]);
    let negative = edit(
        domain,
        EditOperation::Set,
        &[(cell(X, X, InputBranch::Negative), Some(0.5))],
    );
    let result = program(domain, vec![shear, negative])
        .apply(&identity)
        .unwrap();
    transform.map = result;
    assert_eq!(&transform.apply(input)[..2], &[4, 3]);
    input[..2].copy_from_slice(&[2, -3]);
    assert_eq!(&transform.apply(input)[..2], &[-1, -3]);
    assert!(
        MotionProgram::new(
            domain,
            vec![MotionInstruction {
                operation: MotionOperation::ComposeInput(q),
                source: InstructionSource::default()
            }]
        )
        .is_err()
    );
}

#[test]
fn nonfinite_edits_products_and_partial_sums_fail_without_mutating_base() {
    let domain = domain(MouseAxisMapMode::Linear);
    let mut base = MouseAxisMap::default();
    base.rows[0][0] = f64::MAX;
    for op in [EditOperation::Add, EditOperation::MulElements] {
        let p = program(
            domain,
            vec![edit(domain, op, &[(linear_cell(X, X), Some(f64::MAX))])],
        );
        let error = p.apply(&base).unwrap_err();
        assert_eq!(error.instruction_index, Some(0));
        assert!(matches!(
            error.cause,
            ProgramError::NonFinite {
                stage: "coefficient edit",
                ..
            }
        ));
    }
    for output_side in [true, false] {
        let f = factor(domain, [[2.0, -2.0], [1.0, 1.0]]);
        let op = if output_side {
            MotionOperation::ComposeOutput(f)
        } else {
            MotionOperation::ComposeInput(f)
        };
        let error = program(domain, vec![op]).apply(&base).unwrap_err();
        assert!(matches!(
            error.cause,
            ProgramError::NonFinite {
                stage: "product",
                via: Some(X),
                ..
            }
        ));
    }
    // Every term is finite; the second partial sum is not. A later negative
    // contribution must not hide that invalid intermediate arithmetic.
    base.rows[Y as usize][X as usize] = f64::MAX;
    base.rows[Z as usize][X as usize] = f64::MAX;
    let f = NumericFactor::from_block(domain, vec![X, Y, Z], vec![X], vec![vec![1.0, 1.0, -1.0]])
        .unwrap();
    let error = program(domain, vec![MotionOperation::ComposeOutput(f)])
        .apply(&base)
        .unwrap_err();
    assert!(matches!(
        error.cause,
        ProgramError::NonFinite {
            stage: "product partial sum",
            via: Some(Y),
            ..
        }
    ));
    assert_eq!(base.rows[0][0], f64::MAX);

    let mut base = MouseAxisMap::default();
    for input in [X, Y, Z] {
        base.rows[X as usize][input as usize] = f64::MAX;
    }
    let f = NumericFactor::from_block(
        domain,
        vec![X],
        vec![X, Y, Z],
        vec![vec![1.0], vec![1.0], vec![-1.0]],
    )
    .unwrap();
    let error = program(domain, vec![MotionOperation::ComposeInput(f)])
        .apply(&base)
        .unwrap_err();
    assert!(matches!(
        error.cause,
        ProgramError::NonFinite {
            stage: "product partial sum",
            via: Some(Y),
            ..
        }
    ));
}

#[test]
fn directional_negative_product_is_finite_checked_independently() {
    let domain = domain(MouseAxisMapMode::Directional);
    let mut base = MouseAxisMap {
        mode: MouseAxisMapMode::Directional,
        ..Default::default()
    };
    base.negative_rows[0][0] = f64::MAX;
    let f = factor(domain, [[2.0, 0.0], [0.0, 1.0]]);
    let error = program(domain, vec![MotionOperation::ComposeOutput(f)])
        .apply(&base)
        .unwrap_err();
    assert!(matches!(
        error.cause,
        ProgramError::NonFinite {
            cell: Cell {
                input: AxisInput {
                    branch: InputBranch::Negative,
                    ..
                },
                ..
            },
            stage: "product",
            ..
        }
    ));
    assert_eq!(base.negative_rows[0][0], f64::MAX);
}

#[test]
fn product_sum_order_is_canonical_not_factor_declaration_order() {
    let domain = domain(MouseAxisMapMode::Linear);
    let mut base = MouseAxisMap::default();
    base.rows[0][0] = 1e16;
    base.rows[1][0] = 1.0;
    base.rows[4][0] = -1e16;
    let f = NumericFactor::from_block(domain, vec![X, Z, Y], vec![X], vec![vec![1.0, 1.0, 1.0]])
        .unwrap();
    let result = program(domain, vec![MotionOperation::ComposeOutput(f)])
        .apply(&base)
        .unwrap();
    assert_eq!(result.rows[0][0], 0.0); // (1e16 + 1) - 1e16, not (1e16 - 1e16) + 1
    assert!(result.rows[0][0].is_sign_positive());
}

#[test]
fn linear_unused_storage_does_not_change_semantics_or_capture() {
    let domain = domain(MouseAxisMapMode::Linear);
    let identity = MouseAxisMap::default();
    let mut arbitrary = identity;
    arbitrary.negative_rows = [[f64::NAN; AXIS_COUNT]; AXIS_COUNT];
    arbitrary.rows[0][1] = -0.0;
    assert!(identity.semantic_eq(&arbitrary));
    assert!(identity.output_eq(&arbitrary, X));
    assert!(!arbitrary.maps_movement());
    let canonical = program(domain, vec![]).apply(&arbitrary).unwrap();
    assert_eq!(canonical, identity);
    assert!(canonical.rows[0][1].is_sign_positive());
    let mut directional = identity;
    directional.mode = MouseAxisMapMode::Directional;
    assert!(!identity.semantic_eq(&directional));
    assert!(!identity.output_eq(&directional, Misc));
    assert!(program(domain, vec![]).apply(&directional).is_err());
    arbitrary.rows[0][1] = f64::INFINITY;
    assert_eq!(
        program(domain, vec![])
            .apply(&arbitrary)
            .unwrap_err()
            .instruction_index,
        None
    );
}

#[test]
fn validated_types_reject_bad_cells_factors_and_domains() {
    let domain = domain(MouseAxisMapMode::Linear);
    let four = AxisDomain::with_axes(MouseAxisMapMode::Linear, &[X, Y, V, H]);
    for entries in [
        vec![CellValue {
            cell: linear_cell(X, X),
            value: Some(f64::NAN),
        }],
        vec![CellValue {
            cell: cell(X, X, InputBranch::Negative),
            value: Some(1.0),
        }],
        vec![CellValue {
            cell: linear_cell(X, Z),
            value: None,
        }],
        vec![
            CellValue {
                cell: linear_cell(X, X),
                value: None
            };
            2
        ],
    ] {
        assert!(CoefficientOperand::new(four, entries).is_err());
    }
    for (from, to, values) in [
        (vec![], vec![X], vec![vec![]]),
        (vec![X, X], vec![Y], vec![vec![1.0, 2.0]]),
        (vec![X], vec![Y, Y], vec![vec![1.0], vec![1.0]]),
        (vec![X], vec![Z], vec![vec![1.0]]),
        (vec![X], vec![Y], vec![vec![1.0, 2.0]]),
        (vec![X], vec![Y], vec![vec![f64::INFINITY]]),
    ] {
        assert!(NumericFactor::from_block(four, from, to, values).is_err());
    }
    let op = edit(four, EditOperation::Set, &[]);
    assert!(
        MotionProgram::new(
            domain,
            vec![MotionInstruction {
                operation: op,
                source: InstructionSource::default()
            }]
        )
        .is_err()
    );
}

#[test]
fn full_stack_rebuild_deduplicates_highest_holder_and_has_no_drift() {
    let domain = domain(MouseAxisMapMode::Linear);
    let low = program(
        domain,
        vec![edit(
            domain,
            EditOperation::MulElements,
            &[(linear_cell(X, Y), Some(2.0))],
        )],
    );
    let high = program(
        domain,
        vec![edit(
            domain,
            EditOperation::Add,
            &[(linear_cell(X, Y), Some(1.0))],
        )],
    );
    let base = map2([[1.0, 2.0], [3.0, 4.0]]);
    let resolve = |held: &[u16]| {
        canonical_layer_order(held.iter().copied(), 0)
            .into_iter()
            .fold(base, |map, layer| match layer {
                1 => low.apply(&map).unwrap(),
                2 => high.apply(&map).unwrap(),
                _ => map,
            })
    };
    for _ in 0..10 {
        assert_eq!(resolve(&[2, 1]).rows[0][1], 5.0);
    }
    assert_eq!(resolve(&[2]).rows[0][1], 3.0);
    assert_eq!(resolve(&[1, 2, 1]).rows[0][1], 6.0);
    assert_eq!(resolve(&[2, 1]).rows[0][1], 5.0);
    assert_eq!(resolve(&[1]).rows[0][1], 4.0);
    assert_eq!(resolve(&[]).rows[0][1], 2.0);
    assert_eq!(canonical_layer_order([1, 2, 1], 1), vec![2, 1]);
    assert_eq!(canonical_layer_order([2, 1], 3), vec![3, 1, 2]);
}

#[test]
fn runtime_program_and_diagnostics_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<MotionProgram>();
    assert_send_sync::<MotionInstruction>();
    assert_send_sync::<EvaluationError>();
}
