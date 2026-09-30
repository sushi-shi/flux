// (1) fieldless enum: #[reflect] works directly
#[flux::reflect]
#[derive(Clone, Copy)]
pub enum Kind { NotFound, Timeout, Denied }

#[flux::sig(fn(k: Kind{k: k == Kind::NotFound}) -> i32)]
fn handle_nf(k: Kind) -> i32 {
    match k {
        Kind::NotFound => 1,
        _ => unreachable!(),          // must be PROVEN unreachable
    }
}

fn caller(k: Kind) -> i32 {
    match k {
        Kind::NotFound => handle_nf(k),   // fact comes from the match
        _ => 0,
    }
}

fn bad(k: Kind) -> i32 { handle_nf(k) }   // should FAIL

// (2) enum with payloads: have to hand-roll the refinement
#[flux::refined_by(tag: int)]
pub enum Error {
    #[flux::variant((u32) -> Error[0])]
    NotFound(u32),
    #[flux::variant((u64) -> Error[1])]
    Timeout(u64),
    #[flux::variant(Error[2])]
    Denied,
}

#[flux::sig(fn(e: Error{t: t == 0}) -> u32)]
fn handle_nf2(e: Error) -> u32 {
    match e {
        Error::NotFound(id) => id,
        _ => unreachable!(),
    }
}

// "this function can only fail with NotFound or Denied"
#[flux::sig(fn(x: u32) -> Result<u32, Error{t: t == 0 || t == 2}>)]
fn lookup(x: u32) -> Result<u32, Error> {
    if x == 0 { Err(Error::Denied) } else if x > 100 { Err(Error::NotFound(x)) } else { Ok(x) }
}

#[flux::sig(fn(x: u32) -> Result<u32, Error{t: t == 0 || t == 2}>)]
fn lookup_bad(x: u32) -> Result<u32, Error> {
    if x == 0 { Err(Error::Timeout(5)) } else { Ok(x) }    // should FAIL
}

fn caller2(x: u32) -> u32 {
    match lookup(x) {
        Ok(v) => v,
        Err(e @ Error::NotFound(_)) => handle_nf2(e),
        Err(_) => 0,
    }
}
fn main() {}
