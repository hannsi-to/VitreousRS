pub fn align(value: &mut usize, alignment: usize) -> Result<(),String>{
    if (alignment & (alignment - 1)) != 0 {
        return Err("alignment must be a power of 2".to_string());
    }

    *value = (*value + (alignment - 1)) & !(alignment - 1);

    Ok(())
}