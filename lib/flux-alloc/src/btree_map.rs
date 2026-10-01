//! Weighted finite-map model. Contributions are clamped to nonnegative values.
//! Key semantics require an audited integer key and identity Borrow model.

use std::{
    alloc::{Allocator, Global},
    borrow::Borrow,
};

use flux_attrs::*;

#[allow(unused_imports)] // Used by refinement expressions.
use crate::weight::ValueWeight;

defs! { use flux_core::num::max; }

// Global delegates to GlobalAlloc, whose safety contract forbids unwinding.
// Its Clone is trivial; allocation failure aborts. This does not promise that
// arbitrary Allocator implementations or allocation-layout calculations cannot panic.
#[extern_spec(std::alloc)]
#[assoc(fn collection_ops_no_panic() -> bool { true })]
impl Allocator for Global {}

#[extern_spec(std::collections)]
#[refined_by(keys: Set<int>, weights: Map<int, int>, total: int, len: int)]
#[invariant(total >= 0)]
#[invariant(len >= 0 && len <= usize::MAX)]
#[invariant((len == 0) <=> (keys == set_empty(0)))]
#[invariant(len == 0 => total == 0)]
struct BTreeMap<K, V, A: Allocator + Clone = Global>;

#[extern_spec(core::borrow)]
#[assoc(fn preserves_btree_key() -> bool { false })]
trait Borrow<Borrowed: ?Sized> {}

#[extern_spec(core::borrow)]
#[assoc(fn preserves_btree_key() -> bool { true })]
impl<T: ?Sized> Borrow<T> for T {}

#[extern_spec(std::collections)]
impl<K, V> BTreeMap<K, V> {
    #[no_panic]
    #[spec(fn() -> Self[set_empty(0), map_default(0), 0, 0])]
    const fn new() -> Self;
}

#[extern_spec(std::collections)]
impl<K, V> Default for BTreeMap<K, V> {
    #[no_panic]
    #[spec(fn() -> Self[set_empty(0), map_default(0), 0, 0])]
    fn default() -> Self;
}

#[extern_spec(std::collections)]
impl<K, V, A: Allocator + Clone> BTreeMap<K, V, A> {
    #[no_panic_if(<K as Ord>::has_btree_key_model() && <Q as Ord>::has_btree_key_model()
        && <K as Borrow<Q>>::preserves_btree_key())]
    #[spec(fn(&Self[@keys, @weights, @total, @len], &Q[@query])
        -> bool[set_is_in(<Q as Ord>::btree_key(query), keys)]
        requires <K as Ord>::has_btree_key_model() && <Q as Ord>::has_btree_key_model()
            && <K as Borrow<Q>>::preserves_btree_key())]
    fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord;

    // Audited keys are nonzero-sized integers. Live nodes need more than one
    // byte per entry (including node metadata), so the entry count cannot
    // overflow usize before Global allocation fails and aborts. Node layouts
    // are statically sized; unlike Vec growth, there is no doubling calculation.
    #[no_panic_if(<K as Ord>::has_btree_key_model()
        && <A as Allocator>::collection_ops_no_panic())]
    #[spec(fn(self: &strg Self[@keys, @weights, @total, @len], K[@key], V[@value])
        -> Option<V{old: max(0, <V as ValueWeight>::weight(old)) == map_select(weights, <K as Ord>::btree_key(key)) &&
                         max(0, <V as ValueWeight>::weight(old)) <= total}>
             [set_is_in(<K as Ord>::btree_key(key), keys)]
        requires <K as Ord>::has_btree_key_model()
        ensures self: Self[set_union(keys, set_singleton(<K as Ord>::btree_key(key))),
                           map_store(weights, <K as Ord>::btree_key(key), max(0, <V as ValueWeight>::weight(value))),
                           total - (if set_is_in(<K as Ord>::btree_key(key), keys) {
                               map_select(weights, <K as Ord>::btree_key(key))
                           } else { 0 }) + max(0, <V as ValueWeight>::weight(value)),
                           if set_is_in(<K as Ord>::btree_key(key), keys) { len } else { len + 1 }])]
    fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord;

    #[no_panic_if(<K as Ord>::has_btree_key_model() && <Q as Ord>::has_btree_key_model()
        && <K as Borrow<Q>>::preserves_btree_key()
        && <A as Allocator>::collection_ops_no_panic())]
    #[spec(fn(self: &strg Self[@keys, @weights, @total, @len], &Q[@query])
        -> Option<V{removed: max(0, <V as ValueWeight>::weight(removed)) == map_select(weights, <Q as Ord>::btree_key(query)) &&
                             max(0, <V as ValueWeight>::weight(removed)) <= total}>
             [set_is_in(<Q as Ord>::btree_key(query), keys)]
        requires <K as Ord>::has_btree_key_model() && <Q as Ord>::has_btree_key_model()
            && <K as Borrow<Q>>::preserves_btree_key()
        ensures self: Self[set_difference(keys, set_singleton(<Q as Ord>::btree_key(query))),
                           map_store(weights, <Q as Ord>::btree_key(query), 0),
                           total - (if set_is_in(<Q as Ord>::btree_key(query), keys) {
                               map_select(weights, <Q as Ord>::btree_key(query))
                           } else { 0 }),
                           if set_is_in(<Q as Ord>::btree_key(query), keys) { len - 1 } else { len }])]
    fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord;
}

#[extern_spec(std::collections)]
impl<K, V, A: Allocator + Clone> BTreeMap<K, V, A> {
    #[no_panic]
    #[spec(fn(&Self[@m]) -> usize[m.len])]
    fn len(&self) -> usize;
}
