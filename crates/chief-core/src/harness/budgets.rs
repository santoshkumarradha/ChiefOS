//! Turn, cost, and wall-clock budget enforcement.

use super::runtime::TerminationReason;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy)]
pub struct BudgetLimits {
    pub max_turns: u32,
    pub max_cost_usd: f64,
    pub max_wall_secs: u32,
}

impl BudgetLimits {
    pub fn new(max_turns: u32, max_cost_usd: f64, max_wall_secs: u32) -> Self {
        Self {
            max_turns,
            max_cost_usd,
            max_wall_secs,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BudgetState {
    limits: BudgetLimits,
    started_at: Instant,
    turns: u32,
    cost_usd: f64,
}

impl BudgetState {
    pub fn new(limits: BudgetLimits) -> Self {
        Self {
            limits,
            started_at: Instant::now(),
            turns: 0,
            cost_usd: 0.0,
        }
    }

    pub fn record_turn(&mut self, cost_usd: f64) -> Option<TerminationReason> {
        self.turns = self.turns.saturating_add(1);
        self.cost_usd += cost_usd;
        self.check()
    }

    pub fn absorb_child(&mut self, turns: u32, cost_usd: f64) -> Option<TerminationReason> {
        self.turns = self.turns.saturating_add(turns);
        self.cost_usd += cost_usd;
        self.check()
    }

    pub fn check(&self) -> Option<TerminationReason> {
        if self.turns >= self.limits.max_turns {
            return Some(TerminationReason::MaxTurns);
        }
        if self.cost_usd > self.limits.max_cost_usd {
            return Some(TerminationReason::MaxCost);
        }
        if self.elapsed() >= Duration::from_secs(u64::from(self.limits.max_wall_secs)) {
            return Some(TerminationReason::MaxWall);
        }
        None
    }

    pub fn turns(&self) -> u32 {
        self.turns
    }

    pub fn cost_usd(&self) -> f64 {
        self.cost_usd
    }

    pub fn wall_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }

    fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }
}
