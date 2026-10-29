//! Pure ordered edits of one motion map. Nullable coefficient operands and
//! complete numeric composition factors are deliberately different types.
//!
//! No parser expressions, native devices, quantization or mutable publication
//! state belong here. A program always starts from its caller's inherited map;
//! layer changes must rebuild from the immutable baseline, not the last result.

use std::fmt;
use std::sync::Arc;

use super::{AXIS_COUNT, AxisRow, MouseAxis, MouseAxisMap, MouseAxisMapMode};

/// Supported selection domain, retaining the canonical ten-axis storage/order.
/// Unsupported axes are never selected by a scalar's omitted selectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisDomain {
    mode: MouseAxisMapMode,
    supported: [bool; AXIS_COUNT],
}

impl AxisDomain {
    pub fn native(mode: MouseAxisMapMode) -> Self {
        Self {
            mode,
            supported: MouseAxis::ALL.map(MouseAxis::supported),
        }
    }

    /// Explicit domain for capability checks and platform-independent tests.
    /// Input order does not change the canonical arithmetic order.
    pub fn with_axes(mode: MouseAxisMapMode, axes: &[MouseAxis]) -> Self {
        let mut supported = [false; AXIS_COUNT];
        for &axis in axes {
            supported[axis as usize] = true;
        }
        Self { mode, supported }
    }

    pub fn mode(self) -> MouseAxisMapMode {
        self.mode
    }

    pub fn supports(self, axis: MouseAxis) -> bool {
        self.supported[axis as usize]
    }

    pub fn outputs(self) -> impl Iterator<Item = MouseAxis> {
        MouseAxis::ALL
            .into_iter()
            .filter(move |&a| self.supports(a))
    }

    /// Canonical input order: signed axes in linear mode; positive then negative
    /// magnitude branch for each axis in directional mode.
    pub fn inputs(self) -> impl Iterator<Item = AxisInput> {
        self.outputs().flat_map(move |axis| {
            let branches = match self.mode {
                MouseAxisMapMode::Linear => [Some(InputBranch::Signed), None],
                MouseAxisMapMode::Directional => {
                    [Some(InputBranch::Positive), Some(InputBranch::Negative)]
                }
            };
            branches
                .into_iter()
                .flatten()
                .map(move |branch| AxisInput { axis, branch })
        })
    }

    fn validate_axis(self, axis: MouseAxis) -> Result<(), ProgramError> {
        if !self.supports(axis) {
            return Err(ProgramError::Invalid(format!(
                "mouse axis {} is unsupported in this domain",
                axis.name()
            )));
        }
        Ok(())
    }

    fn validate_cell(self, cell: Cell) -> Result<(), ProgramError> {
        self.validate_axis(cell.output)?;
        self.validate_axis(cell.input.axis)?;
        if !matches!(
            (self.mode, cell.input.branch),
            (MouseAxisMapMode::Linear, InputBranch::Signed)
                | (
                    MouseAxisMapMode::Directional,
                    InputBranch::Positive | InputBranch::Negative
                )
        ) {
            return Err(ProgramError::Invalid(format!(
                "input {} does not match {:?} mouse-axis mode",
                cell.input, self.mode
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputBranch {
    Signed,
    Positive,
    Negative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisInput {
    pub axis: MouseAxis,
    pub branch: InputBranch,
}

impl fmt::Display for AxisInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = match self.branch {
            InputBranch::Signed => "",
            InputBranch::Positive => "+",
            InputBranch::Negative => "-",
        };
        write!(f, "{}{suffix}", self.axis.name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Cell {
    pub output: MouseAxis,
    pub input: AxisInput,
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} <- {}", self.output.name(), self.input)
    }
}

/// `None` is an explicit null, not zero or a restore of an earlier value.
/// Omitted cells are absent from this list and have the same no-edit behavior.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellValue {
    pub cell: Cell,
    pub value: Option<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CoefficientOperand {
    domain: AxisDomain,
    cells: Vec<CellValue>,
}

impl CoefficientOperand {
    pub fn new(domain: AxisDomain, mut cells: Vec<CellValue>) -> Result<Self, ProgramError> {
        for entry in &mut cells {
            domain.validate_cell(entry.cell)?;
            if let Some(value) = &mut entry.value {
                *value = finite(*value, entry.cell, "operand", None)?;
            }
        }
        for (index, entry) in cells.iter().enumerate() {
            if cells[..index].iter().any(|other| other.cell == entry.cell) {
                return Err(ProgramError::Invalid(format!(
                    "duplicate cell {}",
                    entry.cell
                )));
            }
        }
        Ok(Self { domain, cells })
    }

    pub fn cells(&self) -> &[CellValue] {
        &self.cells
    }
}

/// Full square numeric factor on the canonical basis. Named rectangular blocks
/// clear listed rows before filling; unlisted rows (including hidden axes) stay
/// identity. The original axis order is retained for explanations.
#[derive(Clone, Debug, PartialEq)]
pub struct NumericFactor {
    domain: AxisDomain,
    from: Vec<MouseAxis>,
    to: Vec<MouseAxis>,
    rows: [AxisRow; AXIS_COUNT],
}

impl NumericFactor {
    pub fn from_block(
        domain: AxisDomain,
        from: Vec<MouseAxis>,
        to: Vec<MouseAxis>,
        values: Vec<Vec<f64>>,
    ) -> Result<Self, ProgramError> {
        for (label, axes) in [("from", &from), ("to", &to)] {
            if axes.is_empty() {
                return Err(ProgramError::Invalid(format!(
                    "factor {label} must be nonempty"
                )));
            }
            for (index, &axis) in axes.iter().enumerate() {
                domain.validate_axis(axis)?;
                if axes[..index].contains(&axis) {
                    return Err(ProgramError::Invalid(format!(
                        "duplicate factor {label} axis {}",
                        axis.name()
                    )));
                }
            }
        }
        if values.len() != to.len() || values.iter().any(|row| row.len() != from.len()) {
            return Err(ProgramError::Invalid(format!(
                "factor values must have {} rows and {} columns",
                to.len(),
                from.len()
            )));
        }
        let mut rows = MouseAxisMap::default().rows;
        for (&output, values) in to.iter().zip(values) {
            rows[output as usize] = [0.0; AXIS_COUNT];
            for (&input, value) in from.iter().zip(values) {
                rows[output as usize][input as usize] = finite(
                    value,
                    Cell {
                        output,
                        input: AxisInput {
                            axis: input,
                            branch: InputBranch::Signed,
                        },
                    },
                    "factor",
                    None,
                )?;
            }
        }
        Ok(Self {
            domain,
            from,
            to,
            rows,
        })
    }

    pub fn from_axes(&self) -> &[MouseAxis] {
        &self.from
    }
    pub fn to_axes(&self) -> &[MouseAxis] {
        &self.to
    }
    pub fn rows(&self) -> &[AxisRow; AXIS_COUNT] {
        &self.rows
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditOperation {
    Set,
    Add,
    MulElements,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MotionOperation {
    Edit(EditOperation, CoefficientOperand),
    ComposeOutput(NumericFactor),
    ComposeInput(NumericFactor),
}

impl MotionOperation {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Edit(EditOperation::Set, _) => "set",
            Self::Edit(EditOperation::Add, _) => "add",
            Self::Edit(EditOperation::MulElements, _) => "mul-elements",
            Self::ComposeOutput(_) => "compose-output",
            Self::ComposeInput(_) => "compose-input",
        }
    }

    fn domain(&self) -> AxisDomain {
        match self {
            Self::Edit(_, operand) => operand.domain,
            Self::ComposeOutput(factor) | Self::ComposeInput(factor) => factor.domain,
        }
    }
}

/// Zero-based byte/line/byte-column coordinates, independent of parser `Span`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourcePosition {
    pub byte: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SourceLocation {
    pub file_name: Arc<str>,
    pub start: SourcePosition,
    pub end: SourcePosition,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}",
            self.file_name,
            self.start.line + 1,
            self.start.column + 1
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InstructionSource {
    /// Program use in the layer header, including when the whole program is a variable.
    pub program: SourceLocation,
    /// Instruction expression (possibly a variable reference inside that program).
    pub use_site: SourceLocation,
    /// Resolved instruction definition when different from its use site.
    pub definition: Option<SourceLocation>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MotionInstruction {
    pub operation: MotionOperation,
    pub source: InstructionSource,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MotionProgram {
    domain: AxisDomain,
    instructions: Vec<MotionInstruction>,
}

impl MotionProgram {
    pub fn new(
        domain: AxisDomain,
        instructions: Vec<MotionInstruction>,
    ) -> Result<Self, ProgramError> {
        for instruction in &instructions {
            if instruction.operation.domain() != domain {
                return Err(ProgramError::Invalid(
                    "motion operand domain mismatch".into(),
                ));
            }
            if domain.mode == MouseAxisMapMode::Directional
                && matches!(instruction.operation, MotionOperation::ComposeInput(_))
            {
                return Err(ProgramError::Invalid(
                    "compose-input is not supported in directional mouse-axis mode".into(),
                ));
            }
        }
        Ok(Self {
            domain,
            instructions,
        })
    }

    pub fn domain(&self) -> AxisDomain {
        self.domain
    }
    pub fn instructions(&self) -> &[MotionInstruction] {
        &self.instructions
    }

    /// Deterministic finite-checked fold. Failure leaves the caller's map intact.
    /// Products read a complete old map, checking every product and partial sum
    /// in canonical axis order, without FMA, fusion, rounding or saturation.
    pub fn apply(&self, inherited: &MouseAxisMap) -> Result<MouseAxisMap, EvaluationError> {
        let mut map = *inherited;
        validate_map(&mut map, self.domain.mode).map_err(|cause| EvaluationError {
            instruction_index: None,
            source: None,
            operation: None,
            cause,
        })?;
        for (index, instruction) in self.instructions.iter().enumerate() {
            apply_instruction(&mut map, &instruction.operation).map_err(|cause| {
                EvaluationError {
                    instruction_index: Some(index),
                    source: Some(Box::new(instruction.source.clone())),
                    operation: Some(instruction.operation.name()),
                    cause,
                }
            })?;
        }
        Ok(map)
    }
}

#[derive(Clone, Debug, PartialEq, thiserror::Error)]
pub enum ProgramError {
    #[error("{0}")]
    Invalid(String),
    #[error("nonfinite {stage} at {cell} (product term {via:?})")]
    NonFinite {
        cell: Cell,
        stage: &'static str,
        via: Option<MouseAxis>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluationError {
    /// Zero-based; absent for an invalid inherited map.
    pub instruction_index: Option<usize>,
    pub source: Option<Box<InstructionSource>>,
    pub operation: Option<&'static str>,
    pub cause: ProgramError,
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(index) = self.instruction_index {
            write!(
                f,
                "mouse-axis instruction {} ({})",
                index + 1,
                self.operation.unwrap_or("unknown")
            )?;
            if let Some(source) = &self.source {
                write!(f, " at {} (program {})", source.use_site, source.program)?;
                if let Some(definition) = &source.definition {
                    write!(f, ", defined at {definition}")?;
                }
            }
            write!(f, ": ")?;
        } else {
            write!(f, "inherited mouse-axis map: ")?;
        }
        self.cause.fmt(f)
    }
}

impl std::error::Error for EvaluationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.cause)
    }
}

fn finite(
    value: f64,
    cell: Cell,
    stage: &'static str,
    via: Option<MouseAxis>,
) -> Result<f64, ProgramError> {
    if !value.is_finite() {
        return Err(ProgramError::NonFinite { cell, stage, via });
    }
    // Normalize signed zero without changing other coefficients.
    Ok(if value == 0.0 { 0.0 } else { value })
}

fn validate_map(map: &mut MouseAxisMap, mode: MouseAxisMapMode) -> Result<(), ProgramError> {
    if map.mode != mode {
        return Err(ProgramError::Invalid(
            "inherited mouse-axis map mode mismatch".into(),
        ));
    }
    for output in MouseAxis::ALL {
        for axis in MouseAxis::ALL {
            let cell = Cell {
                output,
                input: AxisInput {
                    axis,
                    branch: match mode {
                        MouseAxisMapMode::Linear => InputBranch::Signed,
                        MouseAxisMapMode::Directional => InputBranch::Positive,
                    },
                },
            };
            let value = &mut map.rows[output as usize][axis as usize];
            *value = finite(*value, cell, "inherited coefficient", None)?;
            if mode == MouseAxisMapMode::Directional {
                let cell = Cell {
                    input: AxisInput {
                        branch: InputBranch::Negative,
                        ..cell.input
                    },
                    ..cell
                };
                let value = &mut map.negative_rows[output as usize][axis as usize];
                *value = finite(*value, cell, "inherited coefficient", None)?;
            }
        }
    }
    if mode == MouseAxisMapMode::Linear {
        // Never expose arbitrary unused negative storage in resolved linear maps.
        map.negative_rows = MouseAxisMap::default().negative_rows;
    }
    Ok(())
}

fn apply_instruction(
    map: &mut MouseAxisMap,
    operation: &MotionOperation,
) -> Result<(), ProgramError> {
    match operation {
        MotionOperation::Edit(operation, operand) => {
            for entry in operand.cells() {
                let Some(value) = entry.value else { continue };
                let rows = match entry.cell.input.branch {
                    InputBranch::Signed | InputBranch::Positive => &mut map.rows,
                    InputBranch::Negative => &mut map.negative_rows,
                };
                let coefficient =
                    &mut rows[entry.cell.output as usize][entry.cell.input.axis as usize];
                let next = match operation {
                    EditOperation::Set => value,
                    EditOperation::Add => *coefficient + value,
                    EditOperation::MulElements => *coefficient * value,
                };
                *coefficient = finite(next, entry.cell, "coefficient edit", None)?;
            }
        }
        MotionOperation::ComposeOutput(factor) | MotionOperation::ComposeInput(factor) => {
            let old = *map;
            let output_side = matches!(operation, MotionOperation::ComposeOutput(_));
            let branch = if map.mode == MouseAxisMapMode::Linear {
                InputBranch::Signed
            } else {
                InputBranch::Positive
            };
            map.rows = product(&old.rows, &factor.rows, output_side, branch)?;
            if map.mode == MouseAxisMapMode::Directional {
                map.negative_rows = product(
                    &old.negative_rows,
                    &factor.rows,
                    output_side,
                    InputBranch::Negative,
                )?;
            }
        }
    }
    Ok(())
}

fn product(
    old: &[AxisRow; AXIS_COUNT],
    factor: &[AxisRow; AXIS_COUNT],
    output_side: bool,
    branch: InputBranch,
) -> Result<[AxisRow; AXIS_COUNT], ProgramError> {
    let mut result = [[0.0; AXIS_COUNT]; AXIS_COUNT];
    for output in MouseAxis::ALL {
        for axis in MouseAxis::ALL {
            let cell = Cell {
                output,
                input: AxisInput { axis, branch },
            };
            let o = output as usize;
            let i = axis as usize;
            let mut sum = 0.0;
            for via in MouseAxis::ALL {
                let k = via as usize;
                let term = if output_side {
                    factor[o][k] * old[k][i]
                } else {
                    old[o][k] * factor[k][i]
                };
                let term = finite(term, cell, "product", Some(via))?;
                sum = finite(sum + term, cell, "product partial sum", Some(via))?;
            }
            result[o][i] = sum;
        }
    }
    Ok(result)
}

/// Full held/default order, lowest priority first. Deduplication keeps the
/// highest surviving occurrence; no keyboard transparent-resolution fallback.
pub fn canonical_layer_order(
    held_newest_first: impl IntoIterator<Item = u16>,
    default: u16,
) -> Vec<u16> {
    let mut order = Vec::new();
    for layer in held_newest_first
        .into_iter()
        .chain(std::iter::once(default))
    {
        if !order.contains(&layer) {
            order.push(layer);
        }
    }
    order.reverse();
    order
}

#[cfg(test)]
mod tests;
