use super::*;

#[test]
fn diamond_is_not_released_early_and_old_rebuild_is_stale() -> Result<(), Refusal> {
    let mut dag = Rigging::new(1, [0, 1, 1, 6, 0])?;
    let root = dag.start(0)?;
    dag.finish(root, Outcome::Success)?;
    let left = dag.start(1)?;
    dag.finish(left, Outcome::Success)?;
    assert_eq!(dag.finish(left, Outcome::Success), Ok(Recorded::Repeated));
    assert_eq!(dag.start(3), Err(Refusal::NotReady));
    let right = dag.start(2)?;
    dag.finish(right, Outcome::Failed)?;
    assert_eq!(dag.ready(), 16);
    assert_eq!(dag.invalidate(2), Ok(12));
    let right = dag.start(2)?;
    dag.finish(right, Outcome::Success)?;
    let old_join = dag.start(3)?;
    assert_eq!(dag.invalidate(1), Ok(10));
    assert_eq!(dag.finish(old_join, Outcome::Success), Err(Refusal::Stale));
    assert_eq!(dag.ready(), 18);
    assert_eq!(Rigging::new(1, [2, 1]), Err(Refusal::InvalidGraph));
    Ok(())
}
