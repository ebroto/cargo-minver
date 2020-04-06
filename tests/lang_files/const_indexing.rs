const INDEXES: [usize; 2] = [1, 2];
const _ARR: [usize; INDEXES[0]] = [42];

fn main() {
    let _ = [42; INDEXES[1]];
}
