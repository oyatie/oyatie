use super::*;

pub fn check_durability_across_reopen<F: ProjectionFixture>(fixture: &mut F) -> Result<(), String> {
    let entry = applied(
        "ten_a",
        1,
        vec![object("ten_a", "ent_a1", "ety_reading", vec![])],
    );
    fixture
        .store()
        .apply(entry, &KeyDesignations::default())
        .map_err(|error| fail("seed apply", format!("{error:?}")))?;
    if !fixture.reopen() {
        return Ok(());
    }
    let head = fixture
        .store()
        .applied_head("ten_a")
        .map_err(|error| fail("head survives reopen", format!("{error:?}")))?;
    if head != 1 {
        return Err(fail("the head survives reopen", format!("{head}")));
    }
    let stored = fixture
        .store()
        .get("ten_a", "ent_a1")
        .map_err(|error| fail("get survives reopen", format!("{error:?}")))?;
    if stored.is_none() {
        return Err(fail("the object survives reopen", "None".to_owned()));
    }
    Ok(())
}
