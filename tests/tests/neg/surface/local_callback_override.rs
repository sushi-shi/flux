use flux_attrs::*;

struct Callback<F>(F);

impl<F: FnMut(usize) -> usize> Callback<F> {
    // Local implementations must establish their callback requirements from
    // their bodies; the extern-model override mechanism grants no assumption.
    #[spec(fn(&mut Self) -> usize where F: FnMut(usize[0]) -> usize)]
    fn call(&mut self) -> usize { //~ ERROR cannot determine corresponding unrefined predicate
        (self.0)(1)
    }
}
