fn main() {
    let array: Box<[u8; 4]> = Box::new([1, 2, 3, 4]);

    match array {
        nums if nums.iter().sum::<u8>() == 10 => {
            drop(nums);
        },
        _ => unreachable!(),
    }
}
