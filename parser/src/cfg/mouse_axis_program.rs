//! Variable-aware compiler for ordered motion programs. Separate from the global
//! map parser: a named operand row is sparse, not a replacement identity row.

use super::{HashMap, ParseError, Result, SExpr, Span, TrimAtomQuotes};
use crate::mouse_axis::program::*;
use crate::mouse_axis::{MouseAxis, MouseAxisMapMode};
use crate::{anyhow_expr, bail_expr};

pub(super) fn parse_motion_program(
    expr: &SExpr,
    vars: &HashMap<String, SExpr>,
    domain: AxisDomain,
) -> Result<MotionProgram> {
    ProgramParser { vars, domain }.program(expr)
}

struct ProgramParser<'a> {
    vars: &'a HashMap<String, SExpr>,
    domain: AxisDomain,
}

impl ProgramParser<'_> {
    /// Guard alias chains before invoking the usual SExpr accessors (which also
    /// record LSP references). Do not let an unknown/cyclic variable turn into a
    /// misleading shape error or recursive accessor stack overflow.
    fn resolved<'a>(&'a self, mut expr: &'a SExpr) -> Result<&'a SExpr> {
        let mut seen = Vec::new();
        while let SExpr::Atom(atom) = expr {
            let Some(name) = atom.t.strip_prefix('$') else {
                break;
            };
            if seen.contains(&name) {
                bail_expr!(expr, "cyclic mouse-axis variable reference ${name}");
            }
            seen.push(name);
            expr = self
                .vars
                .get(name)
                .ok_or_else(|| anyhow_expr!(expr, "unknown mouse-axis variable ${name}"))?;
        }
        Ok(expr)
    }

    fn atom<'a>(&'a self, expr: &'a SExpr, what: &str) -> Result<&'a str> {
        self.resolved(expr)?;
        expr.atom(Some(self.vars))
            .map(|value| value.trim_atom_quotes())
            .ok_or_else(|| anyhow_expr!(expr, "expected {what}"))
    }

    fn list<'a>(&'a self, expr: &'a SExpr, what: &str) -> Result<&'a [SExpr]> {
        self.resolved(expr)?;
        expr.list(Some(self.vars))
            .ok_or_else(|| anyhow_expr!(expr, "expected {what}"))
    }

    fn keyed<'a>(&'a self, expr: &'a SExpr) -> Result<(&'a str, &'a [SExpr])> {
        let (head, rest) = self
            .list(expr, "a nonempty mouse-axis form")?
            .split_first()
            .ok_or_else(|| anyhow_expr!(expr, "expected a nonempty mouse-axis form"))?;
        Ok((self.atom(head, "a mouse-axis form name")?, rest))
    }

    fn program(&self, expr: &SExpr) -> Result<MotionProgram> {
        let entries = match self.resolved(expr)? {
            SExpr::Atom(_) => {
                if self.atom(expr, "null or a mouse-axis instruction list")? != "null" {
                    bail_expr!(expr, "expected null or a mouse-axis instruction list");
                }
                &[][..]
            }
            SExpr::List(_) => self.list(expr, "a mouse-axis instruction list")?,
        };
        let mut instructions = Vec::with_capacity(entries.len());
        for (index, entry) in entries.iter().enumerate() {
            instructions.push(self.instruction(entry, expr).map_err(|mut error| {
                error.msg = format!("mouse-axis instruction {}: {}", index + 1, error.msg);
                error
            })?);
        }
        MotionProgram::new(self.domain, instructions).map_err(|error| anyhow_expr!(expr, "{error}"))
    }

    fn instruction(&self, expr: &SExpr, program: &SExpr) -> Result<MotionInstruction> {
        let (name, args) = self.keyed(expr)?;
        let [operand] = args else {
            bail_expr!(expr, "{name} takes exactly one operand");
        };
        let operation = match name {
            "set" | "add" | "mul-elements" => MotionOperation::Edit(
                match name {
                    "set" => EditOperation::Set,
                    "add" => EditOperation::Add,
                    _ => EditOperation::MulElements,
                },
                self.operand(operand)?,
            ),
            "compose-output" => MotionOperation::ComposeOutput(self.factor(operand)?),
            "compose-input" => {
                if self.domain.mode() == MouseAxisMapMode::Directional {
                    bail_expr!(
                        expr,
                        "compose-input is not supported in directional mouse-axis mode"
                    );
                }
                MotionOperation::ComposeInput(self.factor(operand)?)
            }
            _ => bail_expr!(
                expr,
                "unknown mouse-axis operation {name}; expected set, add, mul-elements, compose-output or compose-input"
            ),
        };
        let use_site = location(&expr.span());
        let definition = location(&self.resolved(expr)?.span());
        Ok(MotionInstruction {
            operation,
            source: InstructionSource {
                program: location(&program.span()),
                definition: (definition != use_site).then_some(definition),
                use_site,
            },
        })
    }

    fn axis_text(&self, expr: &SExpr, name: &str) -> Result<MouseAxis> {
        let axis = MouseAxis::parse(name).ok_or_else(|| {
            anyhow_expr!(
                expr,
                "unknown mouse axis {name}; expected x, y, v, h, z, rx, ry, rz, dial or misc"
            )
        })?;
        if !self.domain.supports(axis) {
            bail_expr!(
                expr,
                "mouse axis {name} is unsupported in this domain (extended axes require the Linux/Android backend)"
            );
        }
        Ok(axis)
    }

    fn output(&self, expr: &SExpr) -> Result<MouseAxis> {
        self.axis_text(expr, self.atom(expr, "an unsuffixed mouse axis")?)
    }

    fn input(&self, expr: &SExpr) -> Result<AxisInput> {
        let text = self.atom(expr, "a mouse-axis input")?;
        let (name, branch) = match self.domain.mode() {
            MouseAxisMapMode::Linear => (text, InputBranch::Signed),
            MouseAxisMapMode::Directional => {
                if let Some(name) = text.strip_suffix('+') {
                    (name, InputBranch::Positive)
                } else if let Some(name) = text.strip_suffix('-') {
                    (name, InputBranch::Negative)
                } else {
                    bail_expr!(
                        expr,
                        "directional mouse-axis inputs require an explicit + or - branch, got {text}"
                    );
                }
            }
        };
        Ok(AxisInput {
            axis: self.axis_text(expr, name)?,
            branch,
        })
    }

    fn number(&self, expr: &SExpr) -> Result<f64> {
        let text = self.atom(expr, "a finite number")?;
        if text == "null" {
            bail_expr!(
                expr,
                "null is not allowed in numeric factors; expected a finite number"
            );
        }
        text.parse::<f64>()
            .ok()
            .filter(|number| number.is_finite())
            .ok_or_else(|| anyhow_expr!(expr, "expected a finite number, got {text}"))
    }

    fn nullable(&self, expr: &SExpr) -> Result<Option<f64>> {
        if self.atom(expr, "a finite number or null")? == "null" {
            Ok(None)
        } else {
            self.number(expr).map(Some)
        }
    }

    fn operand(&self, expr: &SExpr) -> Result<CoefficientOperand> {
        let (name, args) = self.keyed(expr)?;
        let cells = match name {
            "cell" => {
                let [output, input, value] = args else {
                    bail_expr!(expr, "cell takes OUTPUT INPUT VALUE");
                };
                vec![CellValue {
                    cell: Cell {
                        output: self.output(output)?,
                        input: self.input(input)?,
                    },
                    value: self.nullable(value)?,
                }]
            }
            "matrix" => self.matrix(args)?,
            "scalar" => self.scalar(expr, args)?,
            _ => bail_expr!(
                expr,
                "expected a cell, matrix or scalar coefficient operand; got {name}"
            ),
        };
        CoefficientOperand::new(self.domain, cells).map_err(|error| anyhow_expr!(expr, "{error}"))
    }

    fn matrix(&self, args: &[SExpr]) -> Result<Vec<CellValue>> {
        let mut outputs = Vec::new();
        let mut cells = Vec::new();
        for row in args {
            let (output, entries) = self
                .list(row, "a named matrix row")?
                .split_first()
                .ok_or_else(|| anyhow_expr!(row, "matrix row needs an output axis"))?;
            let output_axis = self.output(output)?;
            if outputs.contains(&output_axis) {
                bail_expr!(output, "duplicate matrix output {}", output_axis.name());
            }
            outputs.push(output_axis);
            if let [value] = entries {
                if matches!(self.resolved(value)?, SExpr::Atom(_))
                    && self.atom(value, "null")? == "null"
                {
                    cells.extend(self.domain.inputs().map(|input| CellValue {
                        cell: Cell {
                            output: output_axis,
                            input,
                        },
                        value: None,
                    }));
                    continue;
                }
            }
            let mut inputs = Vec::new();
            for entry in entries {
                let [input, value] = self.list(entry, "an (INPUT VALUE) matrix entry")? else {
                    bail_expr!(entry, "matrix entry takes INPUT VALUE");
                };
                let input_axis = self.input(input)?;
                if inputs.contains(&input_axis) {
                    bail_expr!(input, "duplicate matrix input {input_axis}");
                }
                inputs.push(input_axis);
                cells.push(CellValue {
                    cell: Cell {
                        output: output_axis,
                        input: input_axis,
                    },
                    value: self.nullable(value)?,
                });
            }
        }
        Ok(cells)
    }

    fn output_axes(&self, expr: &SExpr, args: &[SExpr], label: &str) -> Result<Vec<MouseAxis>> {
        if args.is_empty() {
            bail_expr!(expr, "{label} axis list must be nonempty");
        }
        let mut axes = Vec::with_capacity(args.len());
        for arg in args {
            let axis = self.output(arg)?;
            if axes.contains(&axis) {
                bail_expr!(arg, "duplicate {label} axis {}", axis.name());
            }
            axes.push(axis);
        }
        Ok(axes)
    }

    fn input_axes(&self, expr: &SExpr, args: &[SExpr]) -> Result<Vec<AxisInput>> {
        if args.is_empty() {
            bail_expr!(expr, "inputs axis list must be nonempty");
        }
        let mut axes = Vec::with_capacity(args.len());
        for arg in args {
            let axis = self.input(arg)?;
            if axes.contains(&axis) {
                bail_expr!(arg, "duplicate inputs axis {axis}");
            }
            axes.push(axis);
        }
        Ok(axes)
    }

    fn scalar(&self, expr: &SExpr, args: &[SExpr]) -> Result<Vec<CellValue>> {
        let (value, selectors) = args
            .split_first()
            .ok_or_else(|| anyhow_expr!(expr, "scalar requires a value"))?;
        let value = self.nullable(value)?;
        let mut outputs = None;
        let mut inputs = None;
        let mut exclusions = Vec::new();
        let mut seen = Vec::new();
        for selector in selectors {
            let (name, args) = self.keyed(selector)?;
            if seen.contains(&name) {
                bail_expr!(selector, "duplicate scalar selector {name}");
            }
            seen.push(name);
            match name {
                "outputs" => outputs = Some(self.output_axes(selector, args, name)?),
                "inputs" => inputs = Some(self.input_axes(selector, args)?),
                "except" => {
                    for entry in args {
                        let [output, input] = self.list(entry, "an (OUTPUT INPUT) exclusion")?
                        else {
                            bail_expr!(entry, "except cell takes OUTPUT INPUT");
                        };
                        let cell = Cell {
                            output: self.output(output)?,
                            input: self.input(input)?,
                        };
                        if exclusions.iter().any(|&(other, _)| other == cell) {
                            bail_expr!(entry, "duplicate exclusion {cell}");
                        }
                        exclusions.push((cell, entry));
                    }
                }
                _ => bail_expr!(
                    selector,
                    "unknown scalar selector {name}; expected outputs, inputs or except"
                ),
            }
        }
        let outputs = outputs.unwrap_or_else(|| self.domain.outputs().collect());
        let inputs = inputs.unwrap_or_else(|| self.domain.inputs().collect());
        for &(cell, entry) in &exclusions {
            if !outputs.contains(&cell.output) || !inputs.contains(&cell.input) {
                bail_expr!(entry, "exclusion {cell} lies outside the scalar selection");
            }
        }
        let mut cells = Vec::new();
        for output in outputs {
            for &input in &inputs {
                let cell = Cell { output, input };
                let excluded = exclusions.iter().any(|&(other, _)| other == cell);
                cells.push(CellValue {
                    cell,
                    value: if excluded { None } else { value },
                });
            }
        }
        Ok(cells)
    }

    fn factor(&self, expr: &SExpr) -> Result<NumericFactor> {
        let (name, fields) = self.keyed(expr)?;
        if name != "factor" {
            bail_expr!(expr, "composition requires a numeric factor operand");
        }
        let mut from = None;
        let mut to = None;
        let mut values = None;
        let mut seen = Vec::new();
        for field in fields {
            let (name, args) = self.keyed(field)?;
            if seen.contains(&name) {
                bail_expr!(field, "duplicate factor field {name}");
            }
            seen.push(name);
            match name {
                "from" => from = Some(self.output_axes(field, args, name)?),
                "to" => to = Some(self.output_axes(field, args, name)?),
                "values" => values = Some((field, args)),
                _ => bail_expr!(
                    field,
                    "unknown factor field {name}; expected from, to or values"
                ),
            }
        }
        let from = from.ok_or_else(|| anyhow_expr!(expr, "factor requires from axes"))?;
        let to = to.ok_or_else(|| anyhow_expr!(expr, "factor requires to axes"))?;
        let (values_expr, values) =
            values.ok_or_else(|| anyhow_expr!(expr, "factor requires values"))?;
        if values.len() != to.len() {
            bail_expr!(
                values_expr,
                "factor values need {} rows (one per to axis), got {}",
                to.len(),
                values.len()
            );
        }
        let mut matrix = Vec::with_capacity(values.len());
        for row in values {
            let entries = self.list(row, "a numeric factor row")?;
            if entries.len() != from.len() {
                bail_expr!(
                    row,
                    "factor row needs {} columns (one per from axis), got {}",
                    from.len(),
                    entries.len()
                );
            }
            matrix.push(
                entries
                    .iter()
                    .map(|entry| self.number(entry))
                    .collect::<Result<Vec<_>>>()?,
            );
        }
        NumericFactor::from_block(self.domain, from, to, matrix)
            .map_err(|error| anyhow_expr!(expr, "{error}"))
    }
}

fn location(span: &Span) -> SourceLocation {
    let position = |pos: super::sexpr::Position| SourcePosition {
        byte: pos.absolute,
        line: pos.line,
        column: pos.absolute - pos.line_beginning,
    };
    SourceLocation {
        file_name: span.file_name.as_ref().into(),
        start: position(span.start),
        end: position(span.end),
    }
}
