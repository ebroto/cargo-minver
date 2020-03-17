#![allow(unused, deprecated)]

struct ALt<#[deprecated] 'a>(&'a u32);
struct ATy<#[deprecated] T>(T);
impl<#[deprecated] 'a> ALt<'a> {}
impl<#[deprecated] T> ATy<T> {}

enum BLt<#[deprecated] 'a> {
    Value(&'a u32),
}
enum BTy<#[deprecated] T> {
    Value(T),
}

trait CLt<#[deprecated] 'a> {
    fn foo(&self, _: &'a u32) -> &'a u32;
}
trait CTy<#[deprecated] T> {
    fn foo(&self, _: T);
}
impl<#[deprecated] 'a> CLt<'a> for ALt<'a> {
    fn foo(&self, _: &'a u32) -> &'a u32 {
        loop {}
    }
}
impl<#[deprecated] T> CTy<T> for ATy<T> {
    fn foo(&self, _: T) {}
}

fn f_lt<#[deprecated] 'a>(_: &'a u32) -> &'a u32 {
    loop {}
}
fn f_ty<#[deprecated] T>(_: T) {}

impl<I> ATy<I> {
    fn m_lt<#[deprecated] 'a>(_: &'a [u32]) -> &'a u32 {
        loop {}
    }

    fn m_ty<#[deprecated] T>(_: T) {}
}

type DLt<#[deprecated] 'a> = &'a u32;
type DTy<#[deprecated] T> = T;

fn main() {}
