fn in_tuple() {
    let tuple = (1, 2, 3, 4, 5);

    let (_a, _b, ..) = tuple;
    let (_a, .., _e) = tuple;
    let (.., _d, _e) = tuple;
}

fn in_tuple_struct() {
    struct Test(i32, i32, i32, i32, i32);
    let tuple_struct = Test(1, 2, 3, 4, 5);

    let Test(_a, _b, ..) = tuple_struct;
    let Test(_a, .., _e) = tuple_struct;
    let Test(.., _d, _e) = tuple_struct;
}

fn main() {
    in_tuple();
    in_tuple_struct();
}
