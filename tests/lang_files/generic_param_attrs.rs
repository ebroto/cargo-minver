#![allow(unused, deprecated)]

struct StructLt<#[deprecated] 'a>(&'a u32);
struct StructTy<#[deprecated] T>(T);
impl<#[deprecated] 'a> StructLt<'a> {}
impl<#[deprecated] T> StructTy<T> {}

enum EnumLt<#[deprecated] 'a> {
    Value(&'a u32),
}
enum EnumTy<#[deprecated] T> {
    Value(T),
}

trait TraitLt<#[deprecated] 'a> {
    fn foo(&self, _: &'a u32) -> &'a u32;
}
trait TraitTy<#[deprecated] T> {
    fn foo(&self, _: T);
}
impl<#[deprecated] 'a> TraitLt<'a> for StructLt<'a> {
    fn foo(&self, _: &'a u32) -> &'a u32 {
        loop {}
    }
}
impl<#[deprecated] T, #[cfg(any())] U> TraitTy<T> for StructTy<T> {
    fn foo(&self, _: T) {}
}

fn fun_lt<#[deprecated] 'a>(_: &'a u32) -> &'a u32 {
    loop {}
}
fn fun_ty<#[deprecated] T>(_: T) {}

impl<I> StructTy<I> {
    fn m_lt<#[deprecated] 'a>(_: &'a [u32]) -> &'a u32 {
        loop {}
    }

    fn m_ty<#[deprecated] T>(_: T) {}
}

type TypeLt<#[deprecated] 'a, #[cfg(any())] 'b> = &'a u32;
type TypeTy<#[deprecated] T> = T;

macro_rules! in_macro {
    () => {
        fn more_fun_ty<#[deprecated] T>(_: T) {}
    };
}

in_macro!();

fn main() {}
