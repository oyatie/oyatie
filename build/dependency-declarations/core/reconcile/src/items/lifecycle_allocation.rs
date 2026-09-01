fn lifecycle_allocation_bytes<T>(length: usize) -> Result<usize, LifecycleFailureV1> {
    std::mem::size_of::<T>()
        .checked_mul(length)
        .ok_or_else(lifecycle_bounds)
}

fn lifecycle_add_bytes(
    current: usize,
    additional: usize,
) -> Result<usize, LifecycleFailureV1> {
    current.checked_add(additional).ok_or_else(lifecycle_bounds)
}

fn lifecycle_try_vec<T>(capacity: usize) -> Result<Vec<T>, LifecycleFailureV1> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(capacity)
        .map_err(|_| lifecycle_bounds())?;
    Ok(values)
}

fn lifecycle_try_filled_vec<T: Clone>(
    length: usize,
    value: T,
) -> Result<Vec<T>, LifecycleFailureV1> {
    let mut values = lifecycle_try_vec(length)?;
    values.resize(length, value);
    Ok(values)
}

fn lifecycle_try_push<T>(values: &mut Vec<T>, value: T) -> Result<(), LifecycleFailureV1> {
    if values.len() == values.capacity() {
        values.try_reserve(1).map_err(|_| lifecycle_bounds())?;
    }
    values.push(value);
    Ok(())
}

fn lifecycle_u32_index(value: usize) -> Result<u32, LifecycleFailureV1> {
    u32::try_from(value).map_err(|_| lifecycle_bounds())
}
