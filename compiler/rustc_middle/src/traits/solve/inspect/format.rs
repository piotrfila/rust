use super::*;

pub(super) struct ProofTreeFormatter<'a, 'b> {
    f: &'a mut (dyn Write + 'b),
}

/// A formatter which adds 4 spaces of indentation to its input before
/// passing it on to its nested formatter.
///
/// We can use this for arbitrary levels of indentation by nesting it.
struct Indentor<'a, 'b> {
    f: &'a mut (dyn Write + 'b),
    on_newline: bool,
}

impl Write for Indentor<'_, '_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        for line in s.split_inclusive('\n') {
            if self.on_newline {
                self.f.write_str("    ")?;
            }
            self.on_newline = line.ends_with('\n');
            self.f.write_str(line)?;
        }

        Ok(())
    }
}

impl<'a, 'b> ProofTreeFormatter<'a, 'b> {
    pub(super) fn new(f: &'a mut (dyn Write + 'b)) -> Self {
        ProofTreeFormatter { f }
    }

    fn nested<F, R>(&mut self, func: F) -> R
    where
        F: FnOnce(&mut ProofTreeFormatter<'_, '_>) -> R,
    {
        func(&mut ProofTreeFormatter { f: &mut Indentor { f: self.f, on_newline: true } })
    }

    pub(super) fn format_goal_evaluation(&mut self, eval: &GoalEvaluation<'_>) -> std::fmt::Result {
        let goal_text = match eval.is_normalizes_to_hack {
            IsNormalizesToHack::Yes => "NORMALIZES-TO HACK GOAL",
            IsNormalizesToHack::No => "GOAL",
        };
        writeln!(self.f, "{}: {:?}", goal_text, eval.uncanonicalized_goal)?;
        self.nested(|this| this.format_canonical_goal_evaluation(&eval.evaluation))?;
        if eval.returned_goals.len() > 0 {
            writeln!(self.f, "NESTED GOALS ADDED TO CALLER: [")?;
            self.nested(|this| {
                for goal in eval.returned_goals.iter() {
                    writeln!(this.f, "ADDED GOAL: {goal:?},")?;
                }
                Ok(())
            })?;

            writeln!(self.f, "]")
        } else {
            Ok(())
        }
    }

    pub(super) fn format_canonical_goal_evaluation(
        &mut self,
        eval: &CanonicalGoalEvaluation<'_>,
    ) -> std::fmt::Result {
        writeln!(self.f, "GOAL: {:?}", eval.goal)?;

        match &eval.kind {
            GoalEvaluationKind::Overflow => {
                writeln!(self.f, "OVERFLOW: {:?}", eval.result)
            }
            GoalEvaluationKind::CacheHit(CacheHit::Global) => {
                writeln!(self.f, "GLOBAL CACHE HIT: {:?}", eval.result)
            }
            GoalEvaluationKind::CacheHit(CacheHit::Provisional) => {
                writeln!(self.f, "PROVISIONAL CACHE HIT: {:?}", eval.result)
            }
            GoalEvaluationKind::Uncached { revisions } => {
                for (n, step) in revisions.iter().enumerate() {
                    writeln!(self.f, "REVISION {n}")?;
                    self.nested(|this| this.format_evaluation_step(step))?;
                }
                writeln!(self.f, "RESULT: {:?}", eval.result)
            }
        }
    }

    pub(super) fn format_evaluation_step(
        &mut self,
        evaluation_step: &GoalEvaluationStep<'_>,
    ) -> std::fmt::Result {
        writeln!(self.f, "INSTANTIATED: {:?}", evaluation_step.instantiated_goal)?;
        self.format_candidate(&evaluation_step.evaluation)
    }

    pub(super) fn format_candidate(&mut self, candidate: &GoalCandidate<'_>) -> std::fmt::Result {
        match &candidate.kind {
            ProbeKind::Root { result } => {
                writeln!(self.f, "ROOT RESULT: {result:?}")
            }
            ProbeKind::NormalizedSelfTyAssembly => {
                writeln!(self.f, "NORMALIZING SELF TY FOR ASSEMBLY:")
            }
            ProbeKind::UnsizeAssembly => {
                writeln!(self.f, "ASSEMBLING CANDIDATES FOR UNSIZING:")
            }
            ProbeKind::UpcastProjectionCompatibility => {
                writeln!(self.f, "PROBING FOR PROJECTION COMPATIBILITY FOR UPCASTING:")
            }
            ProbeKind::MiscCandidate { name, result } => {
                writeln!(self.f, "CANDIDATE {name}: {result:?}")
            }
            ProbeKind::TraitCandidate { source, result } => {
                writeln!(self.f, "CANDIDATE {source:?}: {result:?}")
            }
        }?;

        self.nested(|this| {
            for candidate in &candidate.candidates {
                this.format_candidate(candidate)?;
            }
            for nested in &candidate.added_goals_evaluations {
                this.format_added_goals_evaluation(nested)?;
            }
            Ok(())
        })
    }

    pub(super) fn format_added_goals_evaluation(
        &mut self,
        added_goals_evaluation: &AddedGoalsEvaluation<'_>,
    ) -> std::fmt::Result {
        writeln!(self.f, "TRY_EVALUATE_ADDED_GOALS: {:?}", added_goals_evaluation.result)?;

        for (n, iterations) in added_goals_evaluation.evaluations.iter().enumerate() {
            writeln!(self.f, "ITERATION {n}")?;
            self.nested(|this| {
                for goal_evaluation in iterations {
                    this.format_goal_evaluation(goal_evaluation)?;
                }
                Ok(())
            })?;
        }

        Ok(())
    }
}
