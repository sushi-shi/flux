#!/bin/sh
# gen.sh K MODE  -> K loops in ONE function (MODE=infer) or K explicit helpers (MODE=manual)
K=$1; MODE=$2
echo "#[path = \"../../tests/tests/lib/rvec.rs\"] mod rvec; use rvec::RVec;"
if [ $MODE = manual ]; then
  for k in $(seq 1 $K); do cat <<R
#[flux::sig(fn(xs: &RVec<i32>[@n], i: usize{i <= n}, acc: usize{acc <= i}) -> usize{v: v <= n})]
fn step$k(xs: &RVec<i32>, i: usize, acc: usize) -> usize {
    if i < xs.len() { step$k(xs, i + 1, if xs[i] > $k { acc + 1 } else { acc }) } else { acc }
}
R
  done
fi
echo '#[flux::sig(fn(xs: &RVec<i32>[@n]) -> usize{v: v <= n})]'
echo 'fn f(xs: &RVec<i32>) -> usize {'
echo '    let mut best = 0;'
for k in $(seq 1 $K); do
  if [ $MODE = manual ]; then
    echo "    let c$k = step$k(xs, 0, 0);"
  else cat <<L
    let mut i = 0; let mut c$k = 0;
    while i < xs.len() { if xs[i] > $k { c$k += 1; } i += 1; }
L
  fi
  echo "    if c$k > best { best = c$k; }"
done
echo '    best'
echo '}'
echo 'fn main() {}'
