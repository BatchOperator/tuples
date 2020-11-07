use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::{ffi::OsString, fs, path::Path};
use syn::LitInt;

macro_rules! tif {
    { $c:expr => $t:expr ; $e:expr } => { if $c { $t } else { $e } };
}

pub fn code_gen(out_dir: OsString) {
    let t = format_ident!("T");
    let u = format_ident!("U");

    let ctx = init(33, &t, &u);

    #[cfg(feature = "tuple_meta")]
    gen_tuple_impl(&ctx, &out_dir);
    #[cfg(feature = "tuple_meta")]
    gen_tuple_n_impl(&ctx, &out_dir);
    #[cfg(feature = "shorthand")]
    gen_tuple_alias_macro(&ctx, &out_dir);
    #[cfg(feature = "tuple_as")]
    gen_tuple_as(&ctx, &out_dir);
    #[cfg(feature = "tuple_iter")]
    gen_tuple_iter(&ctx, &out_dir);
    #[cfg(feature = "tuple_map")]
    gen_tuple_map(&ctx, &out_dir);
    #[cfg(feature = "combin")]
    gen_combin(&ctx, &out_dir);
    #[cfg(feature = "transpose")]
    gen_transpose(&ctx, &out_dir);
}

struct Ctx<'a> {
    pub t: &'a Ident,
    pub u: &'a Ident,
    pub size_lits: Vec<LitInt>,
    pub ts: Vec<&'a Ident>,
    pub us: Vec<&'a Ident>,
    pub nts: Vec<Ident>,
    pub nvs: Vec<Ident>,
    pub ants: Vec<TokenStream>,
}

fn init<'a>(max: usize, t: &'a Ident, u: &'a Ident) -> Ctx<'a> {
    let size_lits = (0..max + 1)
        .into_iter()
        .map(|i| LitInt::new(i.to_string().as_str(), Span::call_site()))
        .collect::<Vec<_>>();
    let ts = (0..max + 1).into_iter().map(|_| t).collect::<Vec<_>>();
    let us = (0..max + 1).into_iter().map(|_| u).collect::<Vec<_>>();
    let nts = (0..max + 1)
        .into_iter()
        .map(|i| format_ident!("T{}", i))
        .collect::<Vec<_>>();
    let nvs = (0..max + 1)
        .into_iter()
        .map(|i| format_ident!("v{}", i))
        .collect::<Vec<_>>();
    let ants = nts[0..max + 1]
        .iter()
        .map(|i| quote! { #i: 'a })
        .collect::<Vec<_>>();
    let ctx = Ctx {
        t,
        u,
        size_lits,
        ts,
        us,
        nts,
        nvs,
        ants,
    };
    ctx
}

fn gen_tuple_impl(ctx: &Ctx, out_dir: &OsString) {
    let items = (2..33usize)
        .into_iter()
        .map(|i| gen_tuple_impl_size(ctx, i));
    let tks = quote! { #(#items)* };
    let code = tks.to_string();
    let dest_path = Path::new(out_dir).join("tuple_impl.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_impl_size(ctx: &Ctx, size: usize) -> TokenStream {
    let size_lit = &ctx.size_lits[size];

    let ts = &ctx.ts[0..size];

    let nts = &ctx.nts[0..size];

    let tks = quote! {
        impl<T> TupleSame<T> for (#(#ts),*) { }

        impl<#(#nts),*> Tuple for (#(#nts),*) {
            fn arity(&self) -> usize {
                #size_lit
            }
        }
    };
    tks
}

fn gen_tuple_n_impl(ctx: &Ctx, out_dir: &OsString) {
    let item_names = (0..34usize)
        .into_iter()
        .map(|i| format_ident!("Item{}", i))
        .collect::<Vec<_>>();
    let type_items = item_names
        .iter()
        .map(|i| quote! { type #i; })
        .collect::<Vec<_>>();
    let let_items = item_names
        .iter()
        .zip(ctx.nts.iter())
        .map(|(i, t)| quote! { type #i = #t; })
        .collect::<Vec<_>>();
    let items = (2..33usize)
        .into_iter()
        .map(|i| gen_tuple_n_impl_size(ctx, i, &type_items, &let_items));
    let tks = quote! { #(#items)* };
    let code = tks.to_string();
    let dest_path = Path::new(out_dir).join("tuple_n.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_n_impl_size(
    ctx: &Ctx,
    size: usize,
    type_items: &[TokenStream],
    let_items: &[TokenStream],
) -> TokenStream {
    let tuple_name = format_ident!("Tuple{}", size);

    let nts = &ctx.nts[0..size];

    let type_items = &type_items[0..size];
    let let_items = &let_items[0..size];

    let tks = quote! {
        pub trait #tuple_name: Tuple {
            #(#type_items)*
        }
        impl<#(#nts),*> #tuple_name for (#(#nts),*) {
            #(#let_items)*
        }
    };
    tks
}

fn gen_tuple_as(ctx: &Ctx, out_dir: &OsString) {
    let ref_nts = ctx
        .nts
        .iter()
        .map(|id| quote! { &'a #id })
        .collect::<Vec<_>>();
    let mut_nts = ctx
        .nts
        .iter()
        .map(|id| quote! { &'a mut #id })
        .collect::<Vec<_>>();
    let option_nts = ctx
        .nts
        .iter()
        .map(|id| quote! { Option<#id> })
        .collect::<Vec<_>>();
    let ok_nts = ctx
        .nts
        .iter()
        .map(|id| quote! { Result<#id, E> })
        .collect::<Vec<_>>();
    let err_nts = ctx
        .nts
        .iter()
        .map(|id| quote! { Result<O, #id> })
        .collect::<Vec<_>>();
    let ref_impl = ctx
        .size_lits
        .iter()
        .map(|l| {
            quote! { &self.#l }
        })
        .collect::<Vec<_>>();
    let mut_impl = ctx
        .size_lits
        .iter()
        .map(|l| {
            quote! { &mut self.#l }
        })
        .collect::<Vec<_>>();
    let some_impl = ctx
        .size_lits
        .iter()
        .map(|l| {
            quote! { Some(self.#l) }
        })
        .collect::<Vec<_>>();
    let ok_impl = ctx
        .size_lits
        .iter()
        .map(|l| {
            quote! { Ok(self.#l) }
        })
        .collect::<Vec<_>>();
    let err_impl = ctx
        .size_lits
        .iter()
        .map(|l| {
            quote! { Err(self.#l) }
        })
        .collect::<Vec<_>>();
    let items = (2..33usize).into_iter().map(|i| {
        gen_tuple_as_size(
            ctx,
            i,
            &ref_nts[0..i],
            &mut_nts[0..i],
            &option_nts[0..i],
            &ok_nts[0..i],
            &err_nts[0..i],
            &ref_impl[0..i],
            &mut_impl[0..i],
            &some_impl[0..i],
            &ok_impl[0..i],
            &err_impl[0..i],
        )
    });
    let tks = quote! { #(#items)* };
    let code = tks.to_string();
    let dest_path = Path::new(out_dir).join("tuple_as.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_as_size(
    ctx: &Ctx,
    size: usize,
    ref_nts: &[TokenStream],
    mut_nts: &[TokenStream],
    option_nts: &[TokenStream],
    ok_nts: &[TokenStream],
    err_nts: &[TokenStream],
    ref_impl: &[TokenStream],
    mut_impl: &[TokenStream],
    some_impl: &[TokenStream],
    ok_impl: &[TokenStream],
    err_impl: &[TokenStream],
) -> TokenStream {
    let nts = &ctx.nts[0..size];
    let ants = &ctx.ants[0..size];

    let ref_doc = format!("AsRef for Tuple{}", size);
    let mut_doc = format!("AsMut for Tuple{}", size);

    let tks = quote! {
        impl<'a, #(#ants),*> TupleAsRef<'a> for (#(#nts),*) {
            type OutTuple = (#(#ref_nts),*);

            #[doc = #ref_doc]
            fn as_ref(&'a self) -> Self::OutTuple {
                (#(#ref_impl),*)
            }
        }

        impl<'a, #(#ants),*> TupleAsMut<'a> for (#(#nts),*) {
            type OutTuple = (#(#mut_nts),*);

            #[doc = #mut_doc]
            fn as_mut(&'a mut self) -> Self::OutTuple {
                (#(#mut_impl),*)
            }
        }

        impl<#(#nts),*> TupleAsOption for (#(#nts),*) {
            type OutTuple = (#(#option_nts),*);

            fn as_some(self) -> Self::OutTuple {
                (#(#some_impl),*)
            }
        }

        impl<E, #(#nts),*> TupleAsResultOk<E> for (#(#nts),*) {
            type OutTuple = (#(#ok_nts),*);

            fn as_ok(self) -> Self::OutTuple {
                (#(#ok_impl),*)
            }
        }

        impl<O, #(#nts),*> TupleAsResultErr<O> for (#(#nts),*) {
            type OutTuple = (#(#err_nts),*);

            fn as_err(self) -> Self::OutTuple {
                (#(#err_impl),*)
            }
        }
    };
    tks
}

fn gen_tuple_alias_macro(ctx: &Ctx, out_dir: &OsString) {
    let items = (2..33usize)
        .into_iter()
        .map(|i| gen_tuple_alias_macro_size(ctx, i));
    let tks = quote! {
        #[doc(hidden)]
        #[macro_export(local_inner_macros)]
        macro_rules! tuple_ {
            { $t:ty ; 0 } => { () };
            { $t:expr ; 0 } => { () };
            { $t:ty ; 1 } => { ($t,) };
            { $t:expr ; 1 } => { ($t,) };
            #(#items)*
        }
    };
    let code = tks.to_string();
    let dest_path = Path::new(out_dir).join("tuple_alias.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_alias_macro_size(ctx: &Ctx, size: usize) -> TokenStream {
    let size_lit = &ctx.size_lits[size];

    let ty = quote! { $t };
    let tys = (0..size).into_iter().map(|_| &ty).collect::<Vec<_>>();

    let ntys = (0..size + 1)
        .into_iter()
        .map(|i| format_ident!("t{}", i))
        .map(|i| quote! { $#i })
        .collect::<Vec<_>>();

    let items = (0..size + 1)
        .into_iter()
        .map(|i| gen_tuple_alias_macro_size_n(ctx, size, i, &ntys));

    let tks = quote! {
        { $t:ty ; #size_lit } => { (#(#tys),*) };
        { $t:expr ; #size_lit } => { (#(#tys),*) };
        #(#items)*
    };
    tks
}

fn gen_tuple_alias_macro_size_n(
    ctx: &Ctx,
    size: usize,
    n: usize,
    ntys: &[TokenStream],
) -> TokenStream {
    let size_lit = &ctx.size_lits[size];

    let u = quote! { _ };
    let dtys = ntys[0..n]
        .iter()
        .map(|i| quote! { #i:ty })
        .collect::<Vec<_>>();
    let tys = ntys[0..size]
        .iter()
        .enumerate()
        .map(|(i, l)| if i < n { l } else { &u })
        .collect::<Vec<_>>();

    let tks = quote! {
        { #size_lit; #(#dtys),* } => { (#(#tys),*) };
    };
    tks
}

#[cfg(feature = "tuple_iter")]
fn gen_tuple_iter(ctx: &Ctx, out_dir: &OsString) {
    let items = (2..33usize)
        .into_iter()
        .map(|i| gen_tuple_iter_size(ctx, i));
    let tks = quote! { #(#items)* };
    let code = tks.to_string();
    let dest_path = Path::new(out_dir).join("tuple_iter.rs");
    fs::write(&dest_path, code).unwrap();
}

#[cfg(feature = "tuple_iter")]
fn gen_tuple_iter_size(ctx: &Ctx, size: usize) -> TokenStream {
    let size_lit = &ctx.size_lits[size];

    let iter_struct_name = format_ident!("Tuple{}Iter", size);
    let into_iter_struct_name = format_ident!("Tuple{}IntoIter", size);

    let ts = &ctx.ts[0..size];

    let from = quote! { iter.next().unwrap() };
    let froms = (0..size).into_iter().map(|_| &from);

    let iter_new = ctx.size_lits[0..size].iter().map(|i| quote! { &t.#i });
    let into_new = ctx.size_lits[0..size]
        .iter()
        .map(|i| quote! { MaybeUninit::new(t.#i) });

    let derive_iter = if size > 12 {
        quote! {}
    } else {
        quote! {#[derive(Debug, Clone)]}
    };
    let derive_into = if size > 12 {
        quote! {}
    } else {
        quote! {#[derive(Debug)]}
    };

    let iter_ = quote! {
        #derive_iter
        pub struct #iter_struct_name<'a, T>([&'a T; #size_lit], Range<usize>);
        impl<'a, T> #iter_struct_name<'a, T> {
            #[inline]
            pub fn new(t: &'a (#(#ts),*)) -> Self {
                Self([#(#iter_new),*], 0..#size_lit)
            }
        }

        impl<'a, T> Iterator for #iter_struct_name<'a, T> {
            type Item = &'a T;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.1.next().map(|idx| unsafe { *self.0.get_unchecked(idx) })
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.len();
                (len, Some(len))
            }

            #[inline]
            fn count(self) -> usize {
                self.len()
            }

            #[inline]
            fn last(mut self) -> Option<Self::Item> {
                self.next_back()
            }
        }
        impl<'a, T> DoubleEndedIterator for #iter_struct_name<'a, T> {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.1.next_back().map(|idx| unsafe { *self.0.get_unchecked(idx) })
            }
        }
        impl<'a, T> ExactSizeIterator for #iter_struct_name<'a, T> {
            #[inline]
            fn len(&self) -> usize { self.1.end - self.1.start }
        }
        impl<'a, T> FusedIterator for #iter_struct_name<'a, T> { }
        impl<'a, T: 'a> TupleIter<'a> for (#(#ts),*) {
            type Iter = #iter_struct_name<'a, T>;

            #[inline]
            fn iter(&'a self) -> Self::Iter {
                #iter_struct_name::new(self)
            }
        }
    };

    let into_ = quote! {
        #derive_into
        pub struct #into_iter_struct_name<T>([MaybeUninit<T>; #size_lit], Range<usize>);
        impl<T> #into_iter_struct_name<T> {
            #[inline]
            pub fn new(t: (#(#ts),*)) -> Self {
                Self([#(#into_new),*], 0..#size_lit)
            }
        }
        impl<T> Iterator for #into_iter_struct_name<T> {
            type Item = T;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.1.next().map(|idx| unsafe {
                    core::mem::replace(self.0.get_unchecked_mut(idx), MaybeUninit::uninit()).assume_init()
                })
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let len = self.len();
                (len, Some(len))
            }

            #[inline]
            fn count(self) -> usize {
                self.len()
            }

            #[inline]
            fn last(mut self) -> Option<Self::Item> {
                self.next_back()
            }
        }
        impl<T> DoubleEndedIterator for #into_iter_struct_name<T> {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.1.next_back().map(|idx| unsafe {
                    core::mem::replace(self.0.get_unchecked_mut(idx), MaybeUninit::uninit()).assume_init()
                })
            }
        }
        impl<T> ExactSizeIterator for #into_iter_struct_name<T> {
            #[inline]
            fn len(&self) -> usize { self.1.end - self.1.start }
        }
        impl<T> FusedIterator for #into_iter_struct_name<T> { }
        impl<T> TupleIntoIter for (#(#ts),*) {
            type Iter = #into_iter_struct_name<T>;

            #[inline]
            fn into_iter(self) -> Self::Iter {
                #into_iter_struct_name::new(self)
            }
        }
        impl<T> Drop for #into_iter_struct_name<T> {
            fn drop(&mut self) {
                let slice = unsafe { self.0.get_unchecked_mut(self.1.clone()) };
                let slice = unsafe { &mut *(slice as *mut [MaybeUninit<T>] as *mut [T]) };
                unsafe { core::ptr::drop_in_place(slice) }
            }
        }
    };

    let tks = quote! {
        #iter_
        #into_

        impl<T> TupleFromIter<T> for (#(#ts),*) {
            fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
                let mut iter = iter.into_iter();
                (#(#froms),*)
            }
        }
    };
    tks
}

#[cfg(feature = "tuple_map")]
fn gen_tuple_map(ctx: &Ctx, out_dir: &OsString) {
    let items = (2..33usize).into_iter().map(|i| gen_tuple_map_size(ctx, i));
    let tks = quote! { #(#items)* };
    let code = tks.to_string();
    let dest_path = Path::new(out_dir).join("tuple_map.rs");
    fs::write(&dest_path, code).unwrap();
}

#[cfg(feature = "tuple_map")]
fn gen_tuple_map_size(ctx: &Ctx, size: usize) -> TokenStream {
    let items = if size > 16 {
        vec![]
    } else {
        (0..size)
            .into_iter()
            .map(|n| gen_tuple_map_n_size(ctx, size, n))
            .collect()
    };

    let map_name = format_ident!("Tuple{}Map", size);

    let ts = &ctx.ts[0..size];
    let us = &ctx.us[0..size];

    let map_impl = ctx.size_lits[0..size].iter().map(|l| {
        quote! {
            f(self.#l)
        }
    });

    let map_doc = format!("Mapping for Tuple{}", size);

    let tks = quote! {
        #(#items)*

        #[doc = #map_doc]
        pub trait #map_name<T> {
            #[doc = #map_doc]
            fn map<U>(self, f: impl FnMut(T) -> U) -> (#(#us),*);
        }
        impl<T> #map_name<T> for (#(#ts),*) {
            fn map<U>(self, mut f: impl FnMut(T) -> U) -> (#(#us),*) {
                (#(#map_impl),*)
            }
        }
    };
    tks
}

#[cfg(feature = "tuple_map")]
fn gen_tuple_map_n_size(ctx: &Ctx, size: usize, n: usize) -> TokenStream {
    let t = &ctx.nts[n];
    let map_n_name = format_ident!("Tuple{}Map{}", size, n);
    let map_n = format_ident!("map{}", n);

    let rts = &ctx.nts[0..size];
    let ts = ctx.nts[0..size]
        .iter()
        .enumerate()
        .map(|(i, l)| if i == n { ctx.u } else { l })
        .collect::<Vec<_>>();

    let impls = ctx.size_lits[0..size].iter().enumerate().map(|(i, l)| {
        if i == n {
            quote! { f(self.#l) }
        } else {
            quote! { self.#l }
        }
    });

    let doc = format!("Mapping `.{}` for Tuple{}", n, size);

    let tks = quote! {
        #[doc=#doc]
        pub trait #map_n_name<#(#rts),*> {
            #[doc=#doc]
            fn #map_n<U>(self, f: impl FnOnce(#t) -> U) -> (#(#ts),*);
        }
        impl<#(#rts),*> #map_n_name<#(#rts),*> for (#(#rts),*) {
            fn #map_n<U>(self, f: impl FnOnce(#t) -> U) -> (#(#ts),*) {
                (#(#impls),*)
            }
        }
    };
    tks
}

#[cfg(feature = "combin")]
fn gen_combin(ctx: &Ctx, out_dir: &OsString) {
    let self_impl = ctx
        .size_lits
        .iter()
        .map(|i| quote! { self.#i })
        .collect::<Vec<_>>();
    let target_impl = ctx
        .size_lits
        .iter()
        .map(|i| quote! { target.#i })
        .collect::<Vec<_>>();
    let items = (2..33usize)
        .into_iter()
        .map(|i| gen_combin_size(ctx, i, &self_impl));
    let concats = (0..17usize).into_iter().flat_map(|a| {
        let self_impl = &self_impl;
        let target_impl = &target_impl;
        (0..17usize)
            .into_iter()
            .map(move |b| gen_combin_concat_size(ctx, a, b, &self_impl, &target_impl))
    });
    let tks = quote! {
        #(#items)*
        #(#concats)*
    };
    let code = tks.to_string();
    let dest_path = Path::new(out_dir).join("combin.rs");
    fs::write(&dest_path, code).unwrap();
}

#[cfg(feature = "combin")]
fn gen_combin_size(ctx: &Ctx, size: usize, self_impl: &[TokenStream]) -> TokenStream {
    let ts = &ctx.nts[0..size];
    let self_impl = &self_impl[0..size];

    let tks = quote! {
        impl<T, #(#ts),*> CombinLeft<T> for (#(#ts),*) {
            type Out = (T, #(#ts),*);

            fn left(self, target: T) -> Self::Out {
                (target, #(#self_impl),*)
            }
        }
        impl<T, #(#ts),*> CombinRight<T> for ( #(#ts),*) {
            type Out = ( #(#ts),*, T);

            fn push(self, target: T) -> Self::Out {
                (#(#self_impl),*, target)
            }
        }
    };
    tks
}

fn gen_combin_concat_size(
    ctx: &Ctx,
    sizea: usize,
    sizeb: usize,
    self_impl: &[TokenStream],
    target_impl: &[TokenStream],
) -> TokenStream {
    let ants = &ctx.nts[0..sizea];
    let bnts = &ctx.nts[sizea..sizea + sizeb];
    let gnts = &ctx.nts[0..sizea + sizeb];
    let atc = tif! { ants.len() == 1 => quote! { , } ; quote! { } };
    let btc = tif! { bnts.len() == 1 => quote! { , } ; quote! { } };
    let gtc = tif! { gnts.len() == 1 => quote! { , } ; quote! { } };
    let impls = self_impl[0..sizea]
        .iter()
        .chain(target_impl[0..sizeb].iter());
    let tks = quote! {
        impl<#(#gnts),*> CombinConcat<(#(#bnts),*#btc)> for (#(#ants),*#atc) {
            type Out = (#(#gnts),*#gtc);

            #[allow(unused_variables)]
            fn concat(self, target: (#(#bnts),*#btc)) -> Self::Out {
                (#(#impls),*#gtc)
            }
        }
    };
    tks
}

fn gen_transpose(ctx: &Ctx, out_dir: &OsString) {
    let none_impl = ctx
        .size_lits
        .iter()
        .map(|_| quote! { None })
        .collect::<Vec<_>>();
    let items_1 = (2..33usize)
        .into_iter()
        .map(|i| gen_transpose_size_option_1(ctx, i, &none_impl[0..i]));
    let items_2 = (2..33usize)
        .into_iter()
        .map(|i| gen_transpose_size_option_2(ctx, i));

    let tks = quote! { #(#items_1)* #(#items_2)* };
    let code = tks.to_string();
    let dest_path = Path::new(out_dir).join("transpose.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_transpose_size_option_1(ctx: &Ctx, size: usize, none_impl: &[TokenStream]) -> TokenStream {
    let nts = &ctx.nts[0..size];
    let i = &ctx.size_lits[0..size];
    let tks = quote! {
        impl<#(#nts,)*> TupleTranspose for Option<(#(#nts,)*)> {
            type OutTuple = (#(Option<#nts>),*);

            fn transpose(self) -> Self::OutTuple {
                match self {
                    Some(v) => (#(Some(v.#i)),*),
                    None => (#(#none_impl),*),
                }
            }
        }
    };
    tks
}

fn gen_transpose_size_option_2(ctx: &Ctx, size: usize) -> TokenStream {
    let nts = &ctx.nts[0..size];
    let nvs = &ctx.nvs[0..size];
    let tks = quote! {
        impl<#(#nts),*> TupleTranspose for (#(Option<#nts>),*) {
            type OutTuple = Option<(#(#nts),*)>;

            fn transpose(self) -> Self::OutTuple {
                match self {
                    (#(Some(#nvs)),*) => Some((#(#nvs),*)),
                    _ => None,
                }
            }
        }
    };
    tks
}
