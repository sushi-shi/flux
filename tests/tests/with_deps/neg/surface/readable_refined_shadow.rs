extern crate flux_alloc;
extern crate flux_core;

struct BTreeMap<K, V>(K, V);

// Name matching cannot grant a user type the standard library's model.
#[flux_core::refined] //~ ERROR incompatible refinement annotation
struct Buffer { pending: BTreeMap<u32, u32> }
