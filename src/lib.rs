//! A fixed DAG of up to 64 nodes. Scheduling and execution are external.
#![no_std]

use periscope_machines::Issuer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    InvalidGraph,
    UnknownNode,
    NotReady,
    Stale,
    Exhausted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Success,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recorded {
    New,
    Repeated,
}

/// One started attempt, tied to the node's current input generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ticket {
    node: usize,
    generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rigging<const N: usize> {
    prerequisites: [u64; N],
    tickets: [Option<u64>; N],
    issuer: Issuer,
    started: u64,
    success: u64,
    failed: u64,
}

impl<const N: usize> Rigging<N> {
    /// Seal the graph. Each mask names the node's direct prerequisites.
    /// Use a fresh scope for each graph that may receive another's tickets.
    ///
    /// # Errors
    /// Rejects oversized graphs, unknown nodes, and cycles.
    pub fn new(scope: u16, prerequisites: [u64; N]) -> Result<Self, Refusal> {
        if N == 0 || N > 64 || prerequisites.iter().any(|mask| N < 64 && *mask >> N != 0) {
            return Err(Refusal::InvalidGraph);
        }
        let mut resolved = 0_u64;
        for _ in 0..N {
            for (node, &parents) in prerequisites.iter().enumerate() {
                if parents & !resolved == 0 {
                    resolved |= 1 << node;
                }
            }
        }
        if resolved.count_ones() as usize != N {
            return Err(Refusal::InvalidGraph);
        }
        Ok(Self {
            prerequisites,
            tickets: [None; N],
            issuer: Issuer::new(scope),
            started: 0,
            success: 0,
            failed: 0,
        })
    }

    /// Nodes eligible to start; this does not reserve capacity.
    #[must_use]
    pub fn ready(&self) -> u64 {
        self.prerequisites
            .iter()
            .enumerate()
            .fold(0, |mask, (node, parents)| {
                let bit = 1 << node;
                if self.started & bit == 0 && parents & !self.success == 0 {
                    mask | bit
                } else {
                    mask
                }
            })
    }

    /// Reserve a ready node for this generation.
    ///
    /// # Errors
    /// Refuses an unknown or ineligible node, or an exhausted issuer.
    pub fn start(&mut self, node: usize) -> Result<Ticket, Refusal> {
        let ticket = self.tickets.get_mut(node).ok_or(Refusal::UnknownNode)?;
        let bit = 1 << node;
        if self.started & bit != 0 || self.prerequisites[node] & !self.success != 0 {
            return Err(Refusal::NotReady);
        }
        let generation = self.issuer.issue().map_err(|reason| match reason {
            periscope_machines::Refusal::Exhausted => Refusal::Exhausted,
        })?;
        *ticket = Some(generation);
        self.started |= bit;
        Ok(Ticket { node, generation })
    }

    /// Record a matching result; an exact repeat leaves readiness unchanged.
    ///
    /// # Errors
    /// Refuses an obsolete ticket or conflicting repeat.
    pub fn finish(&mut self, ticket: Ticket, outcome: Outcome) -> Result<Recorded, Refusal> {
        if self.tickets.get(ticket.node) != Some(&Some(ticket.generation)) {
            return Err(Refusal::Stale);
        }
        let bit = 1 << ticket.node;
        let (target, other) = match outcome {
            Outcome::Success => (&mut self.success, self.failed),
            Outcome::Failed => (&mut self.failed, self.success),
        };
        if other & bit != 0 {
            return Err(Refusal::Stale);
        }
        if *target & bit != 0 {
            return Ok(Recorded::Repeated);
        }
        *target |= bit;
        Ok(Recorded::New)
    }

    /// Wake: invalidate this node and all descendants, not unrelated work.
    ///
    /// # Errors
    /// Refuses an unknown node without changing state.
    pub fn invalidate(&mut self, node: usize) -> Result<u64, Refusal> {
        if node >= N {
            return Err(Refusal::UnknownNode);
        }
        let mut affected = 1 << node;
        loop {
            let before = affected;
            for (child, parents) in self.prerequisites.iter().enumerate() {
                if parents & affected != 0 {
                    affected |= 1 << child;
                }
            }
            if affected == before {
                break;
            }
        }
        for (index, ticket) in self.tickets.iter_mut().enumerate() {
            if affected & (1 << index) != 0 {
                *ticket = None;
            }
        }
        self.started &= !affected;
        self.success &= !affected;
        self.failed &= !affected;
        Ok(affected)
    }
}

#[cfg(test)]
mod tests;
