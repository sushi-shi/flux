use std::collections::BTreeMap;

#[test]
fn weighted_map_agrees_with_independent_slots_through_replacement_and_removal() {
    let mut map = BTreeMap::new();
    let mut slots = [None; 128];
    let mut total = 0usize;
    let mut seed = 173u64;
    for step in 0..20_000 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let key = (seed >> 32) as usize % slots.len();
        if step % 3 != 0 {
            let len = (seed >> 48) as usize % 257;
            let old = map.insert(key as u32, vec![0u8; len]);
            assert_eq!(old.as_ref().map(Vec::len), slots[key]);
            total = total - old.map_or(0, |v| v.len()) + len;
            slots[key] = Some(len);
        } else {
            let old = map.remove(&(key as u32));
            assert_eq!(old.as_ref().map(Vec::len), slots[key]);
            total -= old.map_or(0, |v| v.len());
            slots[key] = None;
        }
        assert_eq!(total, slots.iter().flatten().sum());
        assert_eq!(map.len(), slots.iter().flatten().count());
        for (key, expected) in slots.iter().enumerate() {
            assert_eq!(map.contains_key(&(key as u32)), expected.is_some());
        }
    }
}

#[test]
fn zst_length_can_exceed_the_byte_allocation_bound() {
    let values = vec![(); usize::MAX];
    assert_eq!(values.len(), usize::MAX);
    assert!(values.len() > isize::MAX as usize);
}

#[test]
fn full_zst_push_really_panics() {
    let mut values = vec![(); usize::MAX];
    assert!(std::panic::catch_unwind(move || values.push(())).is_err());
}
