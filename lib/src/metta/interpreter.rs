use crate::*;
use crate::common::plan::*;
use crate::atom::subexpr::*;
use crate::atom::matcher::*;
use crate::space::grounding::*;

static INTERPRET_OR_DEFAULT_OP: FunctionPlan<(GroundingSpace, Atom, Bindings), InterpreterResult> = FunctionPlan{ func: interpret_or_default_op, name: "interpret_or_default_op" };
static INTERPRET_OP: FunctionPlan<(GroundingSpace, Atom, Bindings), InterpreterResult> = FunctionPlan{ func: interpret_op, name: "interpret_op" };
static INTERPRET_REDUCTED_OP: FunctionPlan<(GroundingSpace, Atom, Bindings), InterpreterResult> = FunctionPlan{ func: interpret_reducted_op, name: "interpret_reducted_op" };
static REDUCT_ARGS_OP: FunctionPlan<(GroundingSpace, Atom, Bindings), InterpreterResult> = FunctionPlan{ func: reduct_args_op, name: "reduct_args_op" };
static MATCH_OP: FunctionPlan<(GroundingSpace, Atom, Bindings), InterpreterResult> = FunctionPlan{ func: match_op, name: "match_op" };
static REDUCT_NEXT_ARG_OP: FunctionPlan<((GroundingSpace, SubexprStream), InterpreterResult), InterpreterResult> = FunctionPlan{ func: reduct_next_arg_op, name: "reduct_next_arg_op" };

static EXECUTE_OP: FunctionPlan<(Atom, Bindings), InterpreterResult> = FunctionPlan{ func: execute_op, name: "execute_op" };
static INTERPRET_RESULTS_FURTHER_OP: FunctionPlan<(GroundingSpace, InterpreterResult), InterpreterResult> = FunctionPlan{ func: interpret_results_further_op, name: "interpret_results_further_op" };
static REDUCT_ARG_BY_ARG_OP: FunctionPlan<(GroundingSpace, SubexprStream, Bindings), InterpreterResult> = FunctionPlan{ func: reduct_arg_by_arg_op, name: "reduct_arg_by_arg_op" };
static INTERPRET_AFTER_ARG_REDUCTION_OP: FunctionPlan<((GroundingSpace, SubexprStream), InterpreterResult), InterpreterResult> = FunctionPlan{ func: interpret_after_arg_reduction_op, name: "interpret_after_arg_reduction_op" };

// TODO: error handing and logging can also be moved into Plan structures
// or implemented atop of them
pub type InterpreterResult = Result<Vec<(Atom, Bindings)>, String>;

fn merge_ok_results(plan_res: InterpreterResult, step_res: InterpreterResult) -> InterpreterResult {
    match (step_res, plan_res) {
        (Ok(mut step_res), Ok(mut plan_res)) => {
            plan_res.append(&mut step_res);
            Ok(plan_res)
        },
        _ => panic!("Error is not expected"),
    }
}

pub fn interpret_init(space: GroundingSpace, expr: &Atom) -> StepResult<InterpreterResult> {
    StepResult::execute(ApplyPlan::new(INTERPRET_OR_DEFAULT_OP, (space, expr.clone(), Bindings::new())))
}

pub fn interpret_step(step: StepResult<InterpreterResult>) -> StepResult<InterpreterResult> {
    match step {
        StepResult::Execute(plan) => plan.step(()),
        StepResult::Return(_) => panic!("Plan execution is finished already"),
        StepResult::Error(_) => panic!("Plan execution is finished with error"),
    }
}

pub fn interpret(space: GroundingSpace, expr: &Atom) -> Result<Vec<Atom>, String> {
    let mut step = interpret_init(space, expr);
    while step.has_next() {
        log::debug!("current plan:\n{:?}", step);
        step = interpret_step(step);
    }
    match step.get_result() {
        Ok(result) => result.map(|mut res| res.drain(0..).map(|(atom, _)| atom).collect()),
        Err(message) => Err(message),
    }
}

fn is_grounded(expr: &ExpressionAtom) -> bool {
    matches!(expr.children().get(0), Some(Atom::Grounded(_)))
}

fn interpret_or_default_op((space, atom, bindings): (GroundingSpace, Atom, Bindings)) -> StepResult<InterpreterResult> {
    log::debug!("interpret_or_default_op: {}, {}", atom, bindings);
    let atom = apply_bindings_to_atom(&atom, &bindings);
    let default = (atom.clone(), bindings.clone());
    StepResult::execute(OrPlan::new(
            ApplyPlan::new(INTERPRET_OP, (space, atom, bindings)),
            StepResult::ret(Ok(vec![default])),
    ))
}

fn interpret_op((space, atom, bindings): (GroundingSpace, Atom, Bindings)) -> StepResult<InterpreterResult> {
    log::debug!("interpret_op: {}, {}", atom, bindings);
    if let Atom::Expression(ref expr) = atom {
        if expr.is_plain() {
            StepResult::execute(ApplyPlan::new(INTERPRET_REDUCTED_OP, (space,  atom, bindings)))
        } else {
            if is_grounded(expr) {
                StepResult::execute(ApplyPlan::new(REDUCT_ARGS_OP, (space,  atom, bindings)))
            } else {
                StepResult::execute(SequencePlan::new(
                    OrPlan::new(
                        ApplyPlan::new(MATCH_OP, (space.clone(), atom.clone(), bindings.clone())),
                        reduct_arg_by_arg_plan((space.clone(), atom.clone(), bindings.clone()))),
                    PartialApplyPlan::new(INTERPRET_RESULTS_FURTHER_OP, space),
                ))
            }
        }
    } else {
        StepResult::ret(Ok(vec![(atom, bindings)]))
    }
}

fn interpret_reducted_op((space, atom, bindings): (GroundingSpace, Atom, Bindings)) -> StepResult<InterpreterResult> {
    let atom = apply_bindings_to_atom(&atom, &bindings);
    log::debug!("interpret_reducted_op: {}", atom);
    if let Atom::Expression(ref expr) = atom {
        if is_grounded(expr) {
            StepResult::execute(SequencePlan::new(
                ApplyPlan::new(EXECUTE_OP, (atom.clone(), bindings.clone())),
                PartialApplyPlan::new(INTERPRET_RESULTS_FURTHER_OP, space)
            ))
        } else {
            StepResult::execute(SequencePlan::new(
                ApplyPlan::new(MATCH_OP, (space.clone(), atom.clone(), bindings.clone())),
                PartialApplyPlan::new(INTERPRET_RESULTS_FURTHER_OP, space)
            ))
        }
    } else {
        StepResult::err("Expression is expected")
    }
}

fn interpret_results_further_op((space, result): (GroundingSpace, InterpreterResult)) -> StepResult<InterpreterResult> {
    match result {
        Err(_) => panic!("Error is not expected here"),
        Ok(mut vec) => StepResult::Execute(
            // Start from empty vector, because empty result is not an error for
            // this operation. It should just process what was passed.
            vec.drain(0..).into_parallel_plan(Ok(vec![]),
            |(result, bindings)| Box::new(
                ApplyPlan::new(INTERPRET_OR_DEFAULT_OP, (space.clone(), result, bindings))),
                merge_ok_results)),
    }
}

fn reduct_arg_by_arg_plan((space, expr, bindings): (GroundingSpace, Atom, Bindings)) -> StepResult<InterpreterResult> {
    log::debug!("reduct_arg_by_arg_plan: {}", expr);
    if let Atom::Expression(_) = expr {
        let iter = SubexprStream::from_expr(expr, BOTTOM_UP_DEPTH_WALK);
        reduct_arg_by_arg_op((space, iter, bindings))
    } else {
        panic!("Atom::Expression is expected as an argument, found: {}", expr)
    }
}

fn reduct_arg_by_arg_op((space, mut iter, bindings): (GroundingSpace, SubexprStream, Bindings)) -> StepResult<InterpreterResult> {
    if let Some(arg) = iter.next().cloned() {
        StepResult::execute(OrPlan::new(
                SequencePlan::new(
                    ApplyPlan::new(INTERPRET_REDUCTED_OP, (space.clone(), arg, bindings.clone())),
                    PartialApplyPlan::new(INTERPRET_AFTER_ARG_REDUCTION_OP, (space.clone(), iter.clone()))),
                ApplyPlan::new(REDUCT_ARG_BY_ARG_OP, (space, iter, bindings))
        ))
    } else {
        StepResult::err("No results for reducted found")
    }
}

fn interpret_after_arg_reduction_op(((space, iter), reduction_result): ((GroundingSpace, SubexprStream), InterpreterResult)) -> StepResult<InterpreterResult> {
    log::debug!("interpret_after_arg_reduction_op: reduction_result: {:?}", reduction_result);
    match reduction_result {
        Ok(vec) if vec.is_empty() => {
            //panic!("Unexpected empty result while reducting: {}, it should be either error or non-empty, full expression: {}", iter.get(), iter.as_atom());
            // TODO: Reducting next argument instead of panic allows creating
            // grounded atom NOP which is not reducted when met inside
            // expression but returns nothing when executed.
            StepResult::err("NOP special case")
        },
        Ok(mut vec) => {
            let plan = vec.drain(0..)
                .into_parallel_plan(Ok(vec![]),
                    |(result, bindings)| {
                        let mut iter = iter.clone();
                        *iter.get_mut() = result;
                        Box::new(ApplyPlan::new(INTERPRET_OR_DEFAULT_OP, (space.clone(), iter.into_atom(), bindings)))
                    },
                    merge_ok_results);
            StepResult::Execute(plan)
        },
        _ => panic!("Only successful results are expected here"),
    }
}

fn find_next_sibling_skip_last<'a>(levels: &mut Vec<usize>, expr: &'a ExpressionAtom, level: usize) -> Option<&'a Atom> {
    let mut idx = levels[level];
    while idx < expr.children().len() - 1 {
        let child = &expr.children()[idx];
        if let Atom::Expression(_) = child {
            levels[level] = idx + 1;
            log::trace!("find_next_sibling_expr: return: {}", child);
            return Some(child);
        }
        idx += 1;
    }
    levels.pop();
    log::trace!("find_next_sibling_expr: return None");
    return None;
}


fn reduct_args_op((space, expr, bindings): (GroundingSpace, Atom, Bindings)) -> StepResult<InterpreterResult> {
    log::debug!("reduct_args_op: {}", expr);
    if let Atom::Expression(ref e) = expr {
        // TODO: remove this hack when it is possible to use types in order
        // to prevent reducing of the last argument of the match
        let mut iter = if format!("{}", e.children()[0]) == "match" {
            log::trace!("skip reducing the last argument of the match");
            SubexprStream::from_expr(expr, find_next_sibling_skip_last)
        } else {
            SubexprStream::from_expr(expr, FIND_NEXT_SIBLING_WALK)
        };
        let sub = iter.next().expect("Non plain expression expected").clone();
        StepResult::execute(SequencePlan::new(
                ApplyPlan::new(INTERPRET_OR_DEFAULT_OP, (space.clone(), sub, bindings)),
                PartialApplyPlan::new(REDUCT_NEXT_ARG_OP, (space, iter))
        ))
    } else {
        StepResult::err(format!("Atom::Expression is expected as an argument, found: {}", expr))
    }
}

fn reduct_next_arg_op(((space, iter), prev_result): ((GroundingSpace, SubexprStream), InterpreterResult)) -> StepResult<InterpreterResult> {
    match prev_result {
        Err(_) => panic!("Error is not expected here, because INTERPRET_OR_DEFAULT_OP always returns result"),
        Ok(mut results) => {
            let plan = results.drain(0..)
                .map(|(reducted, bindings)| {
                    let mut iter = iter.clone();
                    log::debug!("reduct_next_arg_op: reducted: {}, bindings: {:?}", reducted, bindings);
                    *iter.get_mut() = reducted;
                    log::debug!("reduct_next_arg_op: expression: {}", iter.as_atom());

                    let next_sub = if let Some(next_sub) = iter.next().cloned() {
                        log::debug!("reduct_next_arg_op: next_sub after reduction: {}", next_sub);
                        Some(next_sub)
                    } else { None };

                    (next_sub, bindings, iter)
                })
                .into_parallel_plan(Ok(vec![]),
                    |(next_sub, bindings, iter)| {
                        if let Some(next_sub) = next_sub {
                            Box::new(SequencePlan::new(
                                    ApplyPlan::new(INTERPRET_OR_DEFAULT_OP, (space.clone(), next_sub, bindings)),
                                    PartialApplyPlan::new(REDUCT_NEXT_ARG_OP, (space.clone(), iter))
                            ))
                        } else {
                            let expr = iter.into_atom();
                            Box::new(ApplyPlan::new(INTERPRET_REDUCTED_OP,
                                            (space.clone(), expr, bindings)))
                        }
                    },
                    merge_ok_results);
            StepResult::Execute(plan)
        },
    }
}

fn execute_op((atom, bindings): (Atom, Bindings)) -> StepResult<InterpreterResult> {
    log::debug!("execute_op: {}", atom);
    if let Atom::Expression(mut expr) = atom {
        let op = expr.children().get(0).cloned();
        if let Some(Atom::Grounded(op)) = op {
            let mut args = expr.children_mut().drain(1..).collect();
            match op.execute(&mut args) {
                Ok(mut vec) => StepResult::ret(Ok(vec.drain(0..).map(|atom| (atom, bindings.clone())).collect())),
                Err(msg) => StepResult::err(msg),
            }
        } else {
            StepResult::err(format!("Trying to execute non grounded atom: {}", expr))
        }
    } else {
        StepResult::err(format!("Unexpected non expression argument: {}", atom))
    }
}

fn match_op((space, expr, prev_bindings): (GroundingSpace, Atom, Bindings)) -> StepResult<InterpreterResult> {
    log::debug!("match_op: {}", expr);
    let var_x = VariableAtom::from("X");
    // TODO: unique variable?
    let atom_x = Atom::Variable(var_x.clone());
    let mut local_bindings = space.query(&Atom::expr(&[Atom::sym("="), expr.clone(), atom_x]));
    let results: Vec<(Atom, Bindings)> = local_bindings
        .drain(0..)
        .map(|mut binding| {
            let result = binding.remove(&var_x).unwrap(); 
            let result = apply_bindings_to_atom(&result, &binding);
            let bindings = apply_bindings_to_bindings(&binding, &prev_bindings);
            let bindings = bindings.map(|mut bindings| {
                binding.drain().for_each(|(k, v)| { bindings.insert(k, v); });
                bindings
            });
            log::debug!("match_op: query: {}, binding: {:?}, result: {}", expr, bindings, result);
            (result, bindings)
        })
        .filter(|(_, bindings)| bindings.is_ok())
        .map(|(result, bindings)| (result, bindings.unwrap()))
        .collect();
    if results.is_empty() {
        StepResult::err("Match is not found")
    } else {
        StepResult::ret(Ok(results))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_match_all() {
        init_logger();
        let mut space = GroundingSpace::new();
        space.add(expr!("=", ("color"), "blue"));
        space.add(expr!("=", ("color"), "red"));
        space.add(expr!("=", ("color"), "green"));
        let expr = expr!(("color"));

        assert_eq!(interpret(space, &expr),
            Ok(vec![expr!("blue"), expr!("red"), expr!("green")]));
    }

    #[test]
    fn test_frog_reasoning() {
        init_logger();
        let mut space = GroundingSpace::new();
        space.add(expr!("=", ("and", "True", "True"), "True"));
        space.add(expr!("=", ("if", "True", then, else), then));
        space.add(expr!("=", ("if", "False", then, else), else));
        space.add(expr!("=", ("Fritz", "croaks"), "True"));
        space.add(expr!("=", ("Fritz", "eats-flies"), "True"));
        space.add(expr!("=", ("Tweety", "chirps"), "True"));
        space.add(expr!("=", ("Tweety", "yellow"), "True"));
        space.add(expr!("=", ("Tweety", "eats-flies"), "True"));
        let expr = expr!("if", ("and", (x, "croaks"), (x, "eats-flies")),
            ("=", (x, "frog"), "True"), "nop");

        assert_eq!(interpret(space, &expr),
            Ok(vec![expr!("=", ("Fritz", "frog"), "True")]));
    }

    #[test]
    fn test_variable_keeps_value_in_different_sub_expressions() {
        init_logger();
        let mut space = GroundingSpace::new();
        space.add(expr!("=", ("eq", x, x), "True"));
        space.add(expr!("=", ("plus", "Z", y), y));
        space.add(expr!("=", ("plus", ("S", k), y), ("S", ("plus", k, y))));
        let space = space;

        assert_eq!(interpret(space.clone(), &expr!("eq", ("plus", "Z", n), n)),
            Ok(vec![expr!("True")]));
        assert_eq!(interpret(space.clone(), &expr!("eq", ("plus", ("S", "Z"), n), n)),
            Ok(vec![expr!("eq", ("S", y), y)]));
    }
}

