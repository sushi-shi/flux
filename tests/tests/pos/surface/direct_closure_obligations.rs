#[flux::sig(fn(bool[true]))]
fn assert(_: bool) {}

pub fn valid(value: i32) {
    let clamp = |n: i32| {
        let result = if n > 0 { n } else { 0 };
        assert(result >= 0);
        result
    };
    assert(clamp(value) >= 0);
}

pub fn valid_result() {
    let identity = |n: i32| n;
    assert(identity(0) == 0);
}

pub fn captured_bound(floor: i32) {
    let clamp = |n: i32| if n > floor { n } else { floor };
    assert(clamp(floor) >= floor);
    assert(clamp(0) >= floor);
}

pub fn mutable_closure() {
    let mut called = false;
    let mut check = |n: i32| {
        called = true;
        assert(n > 0);
        n
    };
    assert(check(1) == 1);
}

pub fn consumed_closure(token: String) {
    let check = move |n: i32| {
        drop(token);
        assert(n > 0);
        n
    };
    assert(check(1) == 1);
}

pub fn tupled_arguments() {
    let choose = |first: i32, second: i32| if first > second { first } else { second };
    assert(choose(1, 2) == 2);
    let zero = || 0;
    assert(zero() == 0);
}

pub fn mutable_argument() {
    let write = |value: &mut i32| { *value = 1; };
    let mut value = 0;
    write(&mut value);
}
