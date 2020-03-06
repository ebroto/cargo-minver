struct A {}

impl A {
    async fn method(&self) {}
}

async fn fun() {
    let _block = async {};

    let a = A {};
    a.method().await;
}

fn main() {
    let _ = fun();
}
