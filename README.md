# Rigging Machines

> Don't hoist on a line that hasn't been made fast.

Rigging tracks readiness for a fixed dependency graph of 1–64 nodes. A node
can start only after all its direct prerequisites succeed. A failed branch
doesn't stop unrelated work. Rigging tells you what is eligible, not what to
run next or whether you have capacity to run it.

## The Flight Plan

1. **Lay out the dependencies.** Each index names a node; its mask names its
   direct prerequisites. `Rigging::new` seals the graph and rejects cycles and
   out-of-range dependencies. Give each graph a distinct scope if tickets
   might cross instances, and use a fresh scope if old tickets can outlive a
   restart.
2. **Dispatch only ready work.** Use `ready()` to mask candidates for Priority
   or Pipeline. Once the executor has capacity, `start(node)` reserves that
   node in Rigging and returns a scoped Periscope ticket. Keep it with the job.
3. **Report the outcome.** `finish(ticket, Outcome::Success)` can make
   dependants ready. `Outcome::Failed` leaves them blocked. An identical
   completion for a still-current ticket is `Recorded::Repeated`, not another
   release of downstream work.
4. **When an input changes, use Wake.** `invalidate(node)` clears the node and
   all descendants and returns their mask. If a downstream join was already
   running, its old ticket now gets `Refusal::Stale`; the join can't start
   again until its prerequisites succeed again. Unaffected branches keep their
   state.

## What's Yours

Wake is part of Rigging, not a second graph or a cancellation service. It does
not stop obsolete jobs or undo their effects. Serialize publication against
invalidation (or enforce an equivalent fence where the result is written),
and count obsolete jobs against execution capacity until they drain.
