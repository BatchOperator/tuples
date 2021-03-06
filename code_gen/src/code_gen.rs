use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use std::{fs, path::Path};
use syn::LitInt;

macro_rules! tif {
    { $c:expr => $t:expr ; $e:expr } => { if $c { $t } else { $e } };
}

pub fn code_gen(out_dir: &Path) {
    let t = format_ident!("T");
    let u = format_ident!("U");

    let ctx = init(33, &t, &u);

    gen_tuple_impl(&ctx, &out_dir);
    gen_tuple_n_impl(&ctx, &out_dir);
    gen_tuple_alias_macro(&ctx, &out_dir);
    gen_tuple_as(&ctx, &out_dir);
    gen_tuple_iter(&ctx, &out_dir);
    gen_tuple_map(&ctx, &out_dir);
    gen_combin(&ctx, &out_dir);
    gen_transpose(&ctx, &out_dir);
    gen_flatten(&ctx, &out_dir);
    gen_cloned(&ctx, &out_dir);
    gen_call(&ctx, &out_dir);
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
    let size_lits = (0..max + 1).into_iter().map(|i| LitInt::new(i.to_string().as_str(), Span::call_site())).collect::<Vec<_>>();
    let ts = (0..max + 1).into_iter().map(|_| t).collect::<Vec<_>>();
    let us = (0..max + 1).into_iter().map(|_| u).collect::<Vec<_>>();
    let nts = (0..max + 1).into_iter().map(|i| format_ident!("T{}", i)).collect::<Vec<_>>();
    let nvs = (0..max + 1).into_iter().map(|i| format_ident!("v{}", i)).collect::<Vec<_>>();
    let ants = nts[0..max + 1].iter().map(|i| quote! { #i: 'a }).collect::<Vec<_>>();
    let ctx = Ctx { t, u, size_lits, ts, us, nts, nvs, ants };
    ctx
}

fn gen_tuple_impl(ctx: &Ctx, out_dir: &Path) {
    let items = (2..33usize).map(|i| gen_tuple_impl_size(ctx, i));
    let tks = quote! { #(#items)* };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
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

fn gen_tuple_n_impl(ctx: &Ctx, out_dir: &Path) {
    let item_names = (0..34usize).into_iter().map(|i| format_ident!("Item{}", i)).collect::<Vec<_>>();
    let items = (2..33usize).into_iter().map(|i| gen_tuple_n_impl_size(ctx, i, &item_names[0..i]));
    let tks = quote! { #(#items)* };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("tuple_n.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_n_impl_size(ctx: &Ctx, size: usize, item_names: &[Ident]) -> TokenStream {
    let tuple_name = format_ident!("Tuple{}", size);

    let nts = &ctx.nts[0..size];

    let tks = quote! {
        pub trait #tuple_name: Tuple {
            #(type #item_names;)*
        }
        impl<#(#nts),*> #tuple_name for (#(#nts),*) {
            #(type #item_names = #nts;)*
        }
    };
    tks
}

fn gen_tuple_as(ctx: &Ctx, out_dir: &Path) {
    let items = (2..33usize).map(|i| gen_tuple_as_size(ctx, i));
    let tks = quote! { #(#items)* };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("tuple_as.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_as_size(ctx: &Ctx, size: usize) -> TokenStream {
    let size_lits = &ctx.size_lits[0..size];
    let nts = &ctx.nts[0..size];

    let ref_doc = format!("AsRef for Tuple{}", size);
    let mut_doc = format!("AsMut for Tuple{}", size);

    let tks = quote! {
        impl<'a, #(#nts: 'a),*> TupleAsRef<'a> for (#(#nts),*) {
            type OutTuple = (#(&'a #nts),*);

            #[doc = #ref_doc]
            fn as_ref(&'a self) -> Self::OutTuple {
                (#(&self.#size_lits),*)
            }
        }

        impl<'a, #(#nts: 'a),*> TupleAsMut<'a> for (#(#nts),*) {
            type OutTuple = (#(&'a mut #nts),*);

            #[doc = #mut_doc]
            fn as_mut(&'a mut self) -> Self::OutTuple {
                (#(&mut self.#size_lits),*)
            }
        }

        impl<#(#nts),*> TupleAsOption for (#(#nts),*) {
            type OutTuple = (#(Option<#nts>),*);

            fn as_some(self) -> Self::OutTuple {
                (#(Some(self.#size_lits)),*)
            }
        }

        impl<E, #(#nts),*> TupleAsResultOk<E> for (#(#nts),*) {
            type OutTuple = (#(Result<#nts, E>),*);

            fn as_ok(self) -> Self::OutTuple {
                (#(Ok(self.#size_lits)),*)
            }
        }

        impl<O, #(#nts),*> TupleAsResultErr<O> for (#(#nts),*) {
            type OutTuple = (#(Result<O, #nts>),*);

            fn as_err(self) -> Self::OutTuple {
                (#(Err(self.#size_lits)),*)
            }
        }

        impl<'a, #(#nts: 'a + Deref),*> TupleAsDeref<'a> for (#(#nts),*) {
            type OutTuple = (#(&'a <#nts as Deref>::Target),*);

            fn as_deref(&'a self) -> Self::OutTuple {
                (#(self.#size_lits.deref()),*)
            }
        }

        impl<'a, #(#nts: 'a + DerefMut),*> TupleAsDerefMut<'a> for (#(#nts),*) {
            type OutTuple = (#(&'a mut <#nts as Deref>::Target),*);

            fn as_deref_mut(&'a mut self) -> Self::OutTuple {
                (#(self.#size_lits.deref_mut()),*)
            }
        }
    };
    tks
}

fn gen_tuple_alias_macro(ctx: &Ctx, out_dir: &Path) {
    let items = (2..33usize).map(|i| gen_tuple_alias_macro_size(ctx, i));
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
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("tuple_alias.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_alias_macro_size(ctx: &Ctx, size: usize) -> TokenStream {
    let size_lit = &ctx.size_lits[size];

    let ty = quote! { $t };
    let tys = (0..size).into_iter().map(|_| &ty).collect::<Vec<_>>();

    let ntys = (0..size + 1).map(|i| format_ident!("t{}", i)).map(|i| quote! { $#i }).collect::<Vec<_>>();

    let items = (0..size + 1).map(|i| gen_tuple_alias_macro_size_n(ctx, size, i, &ntys));

    let tks = quote! {
        { $t:ty ; #size_lit } => { (#(#tys),*) };
        { $t:expr ; #size_lit } => { (#(#tys),*) };
        #(#items)*
    };
    tks
}

fn gen_tuple_alias_macro_size_n(ctx: &Ctx, size: usize, n: usize, ntys: &[TokenStream]) -> TokenStream {
    let size_lit = &ctx.size_lits[size];

    let u = quote! { _ };
    let nntys = &ntys[0..n];
    let tys = ntys[0..size].iter().enumerate().map(|(i, l)| if i < n { l } else { &u });

    let tks = quote! {
        { #size_lit; #(#nntys:ty),* } => { (#(#tys),*) };
    };
    tks
}

fn gen_tuple_iter(ctx: &Ctx, out_dir: &Path) {
    let items = (2..33usize).map(|i| gen_tuple_iter_size(ctx, i));
    let tks = quote! { #(#items)* };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("tuple_iter.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_iter_size(ctx: &Ctx, size: usize) -> TokenStream {
    let size_lit = &ctx.size_lits[size];
    let size_lits = &ctx.size_lits[0..size];

    let iter_struct_name = format_ident!("Tuple{}Iter", size);
    let into_iter_struct_name = format_ident!("Tuple{}IntoIter", size);

    let ts = &ctx.ts[0..size];

    let from = quote! { iter.next() };
    let froms = (0..size).into_iter().map(|_| &from).collect::<Vec<_>>();

    let derive_iter = tif! {size > 12 =>  quote! {} ; quote! {#[derive(Debug, Clone)]} };
    let derive_into = tif! {size > 12 =>  quote! {} ; quote! {#[derive(Debug)]} };

    let iter_ = quote! {
        #derive_iter
        pub struct #iter_struct_name<'a, T>([&'a T; #size_lit], Range<usize>);
        impl<'a, T> #iter_struct_name<'a, T> {
            #[inline]
            pub fn new(t: &'a (#(#ts),*)) -> Self {
                Self([#(&t.#size_lits),*], 0..#size_lit)
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
                Self([#(MaybeUninit::new(t.#size_lits)),*], 0..#size_lit)
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
                (#(#froms.unwrap()),*)
            }
        }

        impl<T> TupleTryFromIter<T> for (#(#ts),*) {
            fn try_from_iter<I: IntoIterator<Item = T>>(iter: I) -> Option<Self> {
                let mut iter = iter.into_iter();
                Some((#(#froms?),*))
            }
        }

        impl<T> TupleFromIterTry<T> for (#(#ts),*) {
            type OutTuple = (#(Option<#ts>),*);

            fn from_iter_try<I: IntoIterator<Item = T>>(iter: I) -> Self::OutTuple {
                let mut iter = iter.into_iter();
                (#(#froms),*)
            }
        }
    };
    tks
}

fn gen_tuple_map(ctx: &Ctx, out_dir: &Path) {
    let items = (2..33usize).map(|i| gen_tuple_map_size(ctx, i));
    let tks = quote! { #(#items)* };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("tuple_map.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_tuple_map_size(ctx: &Ctx, size: usize) -> TokenStream {
    let items = if size > 16 { vec![] } else { (0..size).map(|n| gen_tuple_map_n_size(ctx, size, n)).collect() };

    let map_name = format_ident!("Tuple{}Map", size);

    let ts = &ctx.ts[0..size];
    let us = &ctx.us[0..size];
    let size_lits = &ctx.size_lits[0..size];

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
                (#(f(self.#size_lits)),*)
            }
        }
    };
    tks
}

fn gen_tuple_map_n_size(ctx: &Ctx, size: usize, n: usize) -> TokenStream {
    let t = &ctx.nts[n];
    let map_n_name = format_ident!("Tuple{}Map{}", size, n);
    let map_n_option_name = format_ident!("Tuple{}Map{}Option", size, n);
    let map_n_option_result = format_ident!("Tuple{}Map{}Result", size, n);
    let map_n = format_ident!("map{}", n);

    let rts = &ctx.nts[0..size];
    let ts = ctx.nts[0..size].iter().enumerate().map(|(i, l)| if i == n { ctx.u } else { l }).collect::<Vec<_>>();

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

        #[doc=#doc]
        pub trait #map_n_option_name<#(#rts),*> {
            #[doc=#doc]
            fn #map_n<U>(self, f: impl FnOnce(#t) -> U) -> Option<(#(#ts),*)>;
        }
        impl<#(#rts),*> #map_n_option_name<#(#rts),*> for Option<(#(#rts),*)> {
            fn #map_n<U>(self, f: impl FnOnce(#t) -> U) -> Option<(#(#ts),*)> {
                match self {
                    Some(v) => Some(v.#map_n(f)),
                    None => None
                }
            }
        }

        #[doc=#doc]
        pub trait #map_n_option_result<E, #(#rts),*> {
            #[doc=#doc]
            fn #map_n<U>(self, f: impl FnOnce(#t) -> U) -> Result<(#(#ts),*), E>;
        }
        impl<E, #(#rts),*> #map_n_option_result<E, #(#rts),*> for Result<(#(#rts),*), E> {
            fn #map_n<U>(self, f: impl FnOnce(#t) -> U) -> Result<(#(#ts),*), E> {
                match self {
                    Ok(v) => Ok(v.#map_n(f)),
                    Err(e) => Err(e)
                }
            }
        }
    };
    tks
}

fn gen_combin(ctx: &Ctx, out_dir: &Path) {
    let self_impl = ctx.size_lits.iter().map(|i| quote! { self.#i }).collect::<Vec<_>>();
    let target_impl = ctx.size_lits.iter().map(|i| quote! { target.#i }).collect::<Vec<_>>();
    let items = (2..33usize).map(|i| gen_combin_size(ctx, i));
    let concats = (0..17usize).flat_map(|a| {
        let self_impl = &self_impl;
        let target_impl = &target_impl;
        (0..17usize).map(move |b| gen_combin_concat_size(ctx, a, b, &self_impl, &target_impl))
    });
    let tks = quote! {
        #(#items)*
        #(#concats)*
    };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("combin.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_combin_size(ctx: &Ctx, size: usize) -> TokenStream {
    let ts = &ctx.nts[0..size];
    let size_lits = &ctx.size_lits[0..size];

    let tks = quote! {
        impl<T, #(#ts),*> CombinLeft<T> for (#(#ts),*) {
            type Out = (T, #(#ts),*);

            fn left(self, target: T) -> Self::Out {
                (target, #(self.#size_lits),*)
            }
        }
        impl<T, #(#ts),*> CombinRight<T> for ( #(#ts),*) {
            type Out = ( #(#ts),*, T);

            fn push(self, target: T) -> Self::Out {
                (#(self.#size_lits),*, target)
            }
        }
    };
    tks
}

fn gen_combin_concat_size(ctx: &Ctx, sizea: usize, sizeb: usize, self_impl: &[TokenStream], target_impl: &[TokenStream]) -> TokenStream {
    let ants = &ctx.nts[0..sizea];
    let bnts = &ctx.nts[sizea..sizea + sizeb];
    let gnts = &ctx.nts[0..sizea + sizeb];
    let atc = tif! { ants.len() == 1 => quote! { , } ; quote! { } };
    let btc = tif! { bnts.len() == 1 => quote! { , } ; quote! { } };
    let gtc = tif! { gnts.len() == 1 => quote! { , } ; quote! { } };
    let impls = self_impl[0..sizea].iter().chain(target_impl[0..sizeb].iter());
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

fn gen_transpose(ctx: &Ctx, out_dir: &Path) {
    let none_impl = ctx.size_lits.iter().map(|_| quote! { None }).collect::<Vec<_>>();
    let items_1 = (2..33usize).map(|i| gen_transpose_size_option_1(ctx, i, &none_impl[0..i]));
    let items_2 = (2..33usize).map(|i| gen_transpose_size_option_2(ctx, i));
    let items_3 = (2..33usize).map(|i| gen_transpose_size_result(ctx, i));

    let tks = quote! { #(#items_1)* #(#items_2)* #(#items_3)* };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
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

fn gen_transpose_size_result(ctx: &Ctx, size: usize) -> TokenStream {
    let nts = &ctx.nts[0..size];
    let ents = (0..size).map(|i| format_ident!("E{}", i)).collect::<Vec<_>>();
    let nvs = &ctx.nvs[0..size];
    let tks = quote! {
        impl<Eo: #(From<#ents>)+*, #(#ents, #nts),*> TupleTransposeResult<Eo> for (#(Result<#nts, #ents>),*) {
            type OutTuple = Result<(#(#nts),*), Eo>;

            fn transpose(self) -> Self::OutTuple {
                let (#(#nvs),*) = self;
                Ok((#(#nvs?),*))
            }
        }
    };
    tks
}

fn gen_flatten(ctx: &Ctx, out_dir: &Path) {
    let et = quote! { () };
    let ets = ctx.nts.iter().map(|_| &et).collect::<Vec<_>>();

    let item_0s = (2..33usize).map(|i| gen_flatten_size_0(&ets[0..i]));

    let items = (2..33).map(|i| gen_flatten_size(ctx, i));

    let tks = quote! {
        #(#item_0s)*
        #(#items)*
    };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("flatten.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_flatten_size_0(ets: &[&TokenStream]) -> TokenStream {
    let tks = quote! {
        impl TupleFlatten for (#(#ets),*) {
            type OutTuple = ();

            fn flatten(self) -> Self::OutTuple {
                ()
            }
        }
    };
    tks
}

fn gen_flatten_size(ctx: &Ctx, size: usize) -> TokenStream {
    let max = (32 / size) + 1;
    let items = (1..33.min(max)).map(|i| gen_flatten_size_n(ctx, size, i));
    let tks = quote! {
        #(#items)*
    };
    tks
}

fn gen_flatten_size_n(ctx: &Ctx, size: usize, n: usize) -> TokenStream {
    let nct = tif! { n == 1 => quote! { , } ; quote! {} };
    let nts = &ctx.nts[0..size * n];
    let nnts = (0..size * n).step_by(n).map(|i| &nts[i..i + n]).map(|nts| quote! { (#(#nts),*#nct) });
    let nnimpl = ctx.size_lits[0..size].iter().flat_map(|i| ctx.size_lits[0..n].iter().map(move |n| quote! { (self.#i).#n }));
    let tks = quote! {
        impl<#(#nts),*> TupleFlatten for (#(#nnts),*) {
            type OutTuple = (#(#nts),*);

            fn flatten(self) -> Self::OutTuple {
                (#(#nnimpl),*)
            }
        }
    };
    tks
}

fn gen_cloned(ctx: &Ctx, out_dir: &Path) {
    let items = (2..33usize).map(|i| gen_cloned_size(ctx, i));
    let tks = quote! { #(#items)* };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("cloned.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_cloned_size(ctx: &Ctx, size: usize) -> TokenStream {
    let nts = &ctx.nts[0..size];
    let size_lits = &ctx.size_lits[0..size];

    let tks = quote! {
        impl<'a, #(#nts: Clone),*> TupleCloned for (#(&'a #nts),*) {
            type TupleOut = (#(#nts),*);

            fn cloned(self) -> Self::TupleOut {
                (#(self.#size_lits.clone()),*)
            }
        }

        impl<'a, #(#nts: Clone),*> TupleCloned for (#(&'a mut #nts),*) {
            type TupleOut = (#(#nts),*);

            fn cloned(self) -> Self::TupleOut {
                (#(self.#size_lits.clone()),*)
            }
        }

        impl<'a, #(#nts: Copy),*> TupleCopied for (#(&'a #nts),*) {
            type TupleOut = (#(#nts),*);

            fn copied(self) -> Self::TupleOut {
                (#(*self.#size_lits),*)
            }
        }

        impl<'a, #(#nts: Copy),*> TupleCopied for (#(&'a mut #nts),*) {
            type TupleOut = (#(#nts),*);

            fn copied(self) -> Self::TupleOut {
                (#(*self.#size_lits),*)
            }
        }
    };
    tks
}

fn gen_call(ctx: &Ctx, out_dir: &Path) {
    let items = (2..33usize).map(|i| gen_call_size(ctx, i));
    let tks = quote! { #(#items)* };
    let mut code = tks.to_string();
    code.insert_str(0, "// This file is by code gen, do not modify\n\n");
    let dest_path = Path::new(out_dir).join("tuple_call.rs");
    fs::write(&dest_path, code).unwrap();
}

fn gen_call_size(ctx: &Ctx, size: usize) -> TokenStream {
    let name = format_ident!("Tuple{}Call", size);
    let nts = &ctx.nts[0..size];
    let size_lits = &ctx.size_lits[0..size];

    let tks = quote! {
        pub trait #name<#(#nts),*> {
            fn call<F: FnOnce(#(#nts),*) -> O, O>(self, f: F) -> O;
        }
        impl<#(#nts),*> #name<#(#nts),*> for (#(#nts),*) {
            fn call<F: FnOnce(#(#nts),*) -> O, O>(self, f: F) -> O {
                f(#(self.#size_lits),*)
            }
        }
    };
    tks
}
