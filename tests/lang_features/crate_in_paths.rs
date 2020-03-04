struct A {}
trait B {}

fn main() {
    crate::module::fun();
}

mod module {
    use crate::A;

    impl crate::B for A {}

    pub fn fun() {
        let _a: A;
    }
}
