//! Procedural macro implementing `#[derive(EnumLike)]`

// Ideas for configurable features:
// * Force NUM_VARIANTS to be a power of 2
// * Implement from_u8 as INSTANCE[x] where INSTANCE is
// static INSTANCE: [Self; NUM_VARIANTS]
// to avoid modulus and division

#![recursion_limit = "128"]

extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;

use std::iter;

use proc_macro::TokenStream;
use quote::ToTokens;
use quote::Tokens;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Field, Fields,
          GenericParam, Generics, Ident, Variant};

#[derive(Debug)]
struct MatchArm {
    name: Ident,
    offset: usize,
    // enum { A, B(usize), C(usize, usize) }
    // A has 0 data fields
    // B has 1 data field
    // C 2
    data_fields: usize,
    field_names: Vec<Tokens>,
    type_names: Vec<Tokens>,
    range_high: Tokens,
}

impl MatchArm {
    fn new(
        name: Ident,
        offset: usize,
        data_fields: usize,
        field_names: Vec<Tokens>,
        type_names: Vec<Tokens>,
        range_high: Tokens,
    ) -> Self {
        Self {
            name,
            offset,
            data_fields,
            field_names,
            type_names,
            range_high,
        }
    }
    // enum ABC { A, B, C }
    // name: A, offset: 0
    fn unit(name: Ident, offset: usize) -> Self {
        Self::new(name, offset, 0, vec![], vec![], quote!(0usize))
    }
    // Creates a MatchArm from the list of fields, and updates range_high
    // for the next variant
    // enum { Some(X) }
    // name: Some, type_name: X
    fn from_fields(
        name: Ident,
        fu_named: &Punctuated<Field, Comma>,
        offset: usize,
        range_high: &mut Tokens,
    ) -> Self {
        // Well, we need one match arm for each variant...
        // Is there any way to know all the variants? Probably not.
        // We could make it just one arm and add some constant
        // Which constant? How do we get it?
        // We get the constant from the type_name:
        // T::NUM_VARIANTS

        // 2 elements is a product type, so we must add
        // A::NUM_VARIANTS * B::NUM_VARIANTS items
        // And now from_discr() requires a division and a modulo
        // Is this worth it?
        let type_names = fu_named
            .iter()
            .map(|u| u.ty.clone().into_tokens())
            .collect();

        let named = fu_named[0].ident.is_some();

        let field_names = if named {
            fu_named
                .iter()
                .map(|u| u.ident.unwrap().into_tokens())
                .collect()
        } else {
            // On an unnamed variant the field names can be set to
            // anything later in the code
            // But must be "()" instead of "{}"
            let mut field_names = vec![];
            let elements = fu_named.len();
            for x in 0..elements {
                let i = syn::Index::from(x);
                field_names.push(quote!(#i));
            }
            field_names
        };

        // The clone is not needed but who am I to fight the borrow checker
        let m = Self::new(
            name,
            offset,
            field_names.len(),
            field_names,
            type_names,
            range_high.clone(),
        );
        *range_high = m.print_next_range_high();

        m
    }
    fn print_to_discr(&self, parent: &Ident) -> Tokens {
        let offset = self.offset;
        let name = &self.name;
        if self.data_fields == 0 {
            quote!(
                #parent::#name => #offset,
            )
        } else {
            let field_names = &self.field_names;
            let type_names = &self.type_names;
            let range_high = &self.range_high;
            let offset = &quote!(#offset + #range_high);
            let xfield_names: Vec<_> = field_names
                .iter()
                .map(|f| Ident::from(format!("x{}", f)))
                .collect();
            let to_discr_0 = to_discr_body(
                field_names,
                type_names,
                offset,
                Some(&xfield_names),
            );

            quote!(
                #parent::#name {
                    #(
                        #field_names: #xfield_names,
                    )*
                } => #to_discr_0,
            )
        }
    }
    fn print_from_discr(&self, parent: &Ident) -> Tokens {
        let offset = self.offset;
        let name = &self.name;
        if self.data_fields == 0 {
            quote!(
                #offset => #parent::#name,
            )
        } else {
            let range_high = &self.range_high;
            let field_names = &self.field_names;
            let type_names = &self.type_names;
            let range_high_next = self.print_next_range_high();
            let value = &quote!(x - #offset - (#range_high));
            let from_discr_0 = from_discr_body(field_names, type_names, value);
            quote!(
                x if x >= (#offset + #range_high)
                    && x < (#offset + #range_high_next) =>
                #parent::#name {
                    #from_discr_0
                },
            )
        }
    }
    // Total NUM_VARIANTS of this arm
    fn print_num_variants(&self) -> Tokens {
        if self.data_fields == 0 {
            // TODO: enum N { A(!) } has 0 elements?
            quote!(1)
        } else {
            total_num_variants_product(&self.type_names)
        }
    }
    // Returns an expression that evaluates to the first integer x > self.to_discr()
    fn print_next_range_high(&self) -> Tokens {
        if self.data_fields == 0 {
            panic!("You don't need to call this");
        } else {
            let range_high = &self.range_high;
            let n = self.print_num_variants();
            quote! {
                #range_high + #n
            }
        }
    }
}

fn all_variants_of(variants: &Punctuated<Variant, Comma>) -> Vec<MatchArm> {
    let mut x = vec![];

    // First push the data-less variants
    // enum { A, B, C }
    // This is important because we need the offset, otherwise everything breaks
    for v in variants {
        match v.fields {
            Fields::Unit => {
                let name = v.ident;
                let idx = x.len() as usize;
                x.push(MatchArm::unit(name, idx));
            }
            _ => {}
        }
    }

    // All the unit-like variants go from 0 to offset
    let offset = x.len() as usize;
    // range_high is the generic offset, for each type T
    // we add T::NUM_VARIANTS
    let mut range_high = quote!(0usize);

    // Next the named and unnamed variants, in definition order
    // enum { Ok(T), Err(E) }
    // Parses to:
    // const NUM_VARIANTS: usize = T::NUM_VARIANTS + E::NUM_VARIANTS;
    // fn to_discr(self) -> usize {
    //  match self {
    //      Ok(x) => x.to_discr(),
    //      Err(x) => x.to_discr() + T::NUM_VARIANTS,
    //  }
    // }
    for v in variants {
        match v.fields {
            Fields::Unnamed(ref fu) => {
                // Ok(x): 1 element
                // A(0, 1, 2): 3 elements
                assert!(
                    fu.unnamed.len() > 0,
                    "This is a unit field, wtf"
                );
                let m = MatchArm::from_fields(
                    v.ident,
                    &fu.unnamed,
                    offset,
                    &mut range_high,
                );
                x.push(m);
            }
            Fields::Named(ref fu) => {
                assert!(
                    fu.named.len() > 0,
                    "This is a named unit field, wtf"
                );
                let m = MatchArm::from_fields(
                    v.ident,
                    &fu.named,
                    offset,
                    &mut range_high,
                );
                x.push(m);
            }
            Fields::Unit => {} // We already handled these
        }
    }

    x
}

fn generate_rusty_enum_code(
    name: &Ident,
    generics: Generics,
    variants: &Punctuated<Variant, Comma>,
) -> Tokens {
    // We cannot do this:
    // First check if any of the variants implements EnumLike,
    // then we can recursively implement it
    //
    // Instead we assume that every variant implements EnumLike
    let enum_variants = all_variants_of(variants);
    let match_arm_from = enum_variants
        .iter()
        .map(|e| e.print_from_discr(name));
    let match_arm_to = enum_variants
        .iter()
        .map(|e| e.print_to_discr(name));
    let enum_count_s = enum_variants
        .iter()
        .map(|e| e.print_num_variants());
    let enum_count = quote!(
        #(
            #enum_count_s +
        )* 0usize
    );

    impl_enum_like(
        name,
        generics,
        quote! {
            const NUM_VARIANTS: usize = #enum_count;
            fn from_discr(value: usize) -> Self {
                match value {
                    #(
                        #match_arm_from
                    )*
                    _ => unreachable!()
                }
            }
            fn to_discr(self) -> usize {
                match self {
                    #(
                        #match_arm_to
                    )*
                }
            }
        },
    )
}

fn generate_c_enum_code(
    name: &Ident,
    generics: Generics,
    variants: &Punctuated<Variant, Comma>,
) -> Tokens {
    let variant_a = variants.iter().map(|variant| &variant.ident);
    let variant_b = variants.iter().map(|variant| &variant.ident);
    let repeat_name_a = iter::repeat(name);
    let repeat_name_b = iter::repeat(name);
    let counter_a = 0..variants.len() as usize;
    let counter_b = 0..variants.len() as usize;

    // We ignore explicit discriminants
    // (enum { A = 100 } becomes enum { A = 0 })

    let enum_count = variants.len();

    impl_enum_like(
        name,
        generics,
        quote! {
            const NUM_VARIANTS: usize = #enum_count;
            fn from_discr(value: usize) -> Self {
                match value {
                    #(
                        #counter_a => #repeat_name_a::#variant_a,
                    )*
                    _ => unreachable!()
                }
            }
            fn to_discr(self) -> usize {
                match self {
                    #(
                        #repeat_name_b::#variant_b => #counter_b,
                    )*
                }
            }
        },
    )
}

fn generate_enum_code(
    name: &Ident,
    generics: Generics,
    variants: &Punctuated<Variant, Comma>,
) -> Tokens {
    // We special-case c-like enums because the generated code is much cleaner
    let c_like = variants
        .iter()
        .all(|v| v.fields == Fields::Unit);

    // An empty enum {} is c_like
    if c_like {
        generate_c_enum_code(name, generics, variants)
    } else {
        generate_rusty_enum_code(name, generics, variants)
    }
}

// struct S;    (true)
// struct S{};  (false)
// struct S();  (false)
fn generate_unit_struct_impl(
    name: &Ident,
    generics: Generics,
    unit: bool,
) -> Tokens {
    // Luckly Self {} is valid syntax for Self = S();
    let hack = if unit { quote!() } else { quote!({}) };

    impl_enum_like(
        name,
        generics,
        quote! {
            const NUM_VARIANTS: usize = 1usize;
            fn from_discr(_value: usize) -> Self {
                #name #hack
            }
            fn to_discr(self) -> usize {
                0usize
            }
        },
    )
}

fn generate_struct_many_elem(
    name: &Ident,
    generics: Generics,
    field_names: &[Tokens],
    type_names: &[Tokens],
) -> Tokens {
    let value = &quote!(value);
    let from_discr_0 = from_discr_body(field_names, type_names, value);
    let offset = &quote!(0usize);
    let to_discr_0 = to_discr_body(field_names, type_names, offset, None);
    let total_num_variants = total_num_variants_product(type_names);

    impl_enum_like(
        name,
        generics,
        quote! {
            const NUM_VARIANTS: usize = #total_num_variants;
            fn from_discr(value: usize) -> Self {
                Self {
                    #from_discr_0
                }
            }
            fn to_discr(self) -> usize {
                #to_discr_0
            }
        },
    )
}

fn total_num_variants_product(type_names: &[Tokens]) -> Tokens {
    let mut type_names_plus_one = type_names.to_vec();
    type_names_plus_one.push(quote!{});
    // hack:
    // num_variants_product(A, B, C).last() returns A*B, we need
    // A*B*C so we add an empty type at the end
    let product = num_variants_product(&type_names_plus_one);

    product.last().unwrap().clone()
}

// For a product type (A, B, C, D), returns:
// [1, A::NUM_VARIANTS, A::NUM_VARIANTS * B::NUM_VARIANTS, A * B * C]
// TODO: if we add .next_power_of_two() here, will it work as expected?
fn num_variants_product(type_names: &[Tokens]) -> Vec<Tokens> {
    let n = type_names.len();
    // 1usize
    let mut last_p = quote!(1usize);
    let mut product = vec![last_p.clone()];
    // A::NUM_VARIANTS
    if n >= 2 {
        let type_n1 = &type_names[0];
        last_p = quote!( <#type_n1 as ::enum_like::EnumLike>::NUM_VARIANTS );
        product.push(last_p.clone());
    }
    // last_p * B::NUM_VARIANTS
    for i in 2..type_names.len() {
        let type_n1 = &type_names[i - 1];
        let old_last_p = last_p.clone();
        last_p = quote! {
            #old_last_p * <#type_n1 as ::enum_like::EnumLike>::NUM_VARIANTS
        };
        product.push(last_p.clone());
    }

    product
}

// Returns the expression inside Self{...} in
// Self { 0: A::from_discr((value - offset) / product) % rem), }
fn from_discr_body(
    field_names: &[Tokens],
    type_names: &[Tokens],
    value: &Tokens,
) -> Tokens {
    let value_r = std::iter::repeat(value);
    let product = num_variants_product(type_names);

    // The last element doesn't need a % operation, we gotta save that cpu cycles
    // Limiting the repeating part of the macro to (n - 1) elements is not that easy
    let n = type_names.len();
    let rem = type_names
        .iter()
        .take(n - 1)
        .map(|tn| {
            quote! {
                .wrapping_rem(<#tn as ::enum_like::EnumLike>::NUM_VARIANTS)
            }
        })
        .chain(std::iter::once(quote!{}));

    // The first element doesn't need a division, but any self-respecting compiler
    // will optimize (x / 1) to x

    // sanity checks
    debug_assert_eq!(field_names.len(), n);
    debug_assert_eq!(type_names.len(), n);
    debug_assert_eq!(product.len(), n);
    debug_assert_eq!(rem.size_hint().0, n);

    // Self { 0: A::from_discr((value - offset) / product) % rem), }
    quote! {
    #(
            #field_names: <#type_names as ::enum_like::EnumLike>::from_discr(
                ( #value_r ).wrapping_div( #product ) #rem
            ),
    )*
        }
}

// Returns self.0.to_discr() + A::NUM_VARIANTS * self.1.to_discr() + ...
fn to_discr_body(
    field_names: &[Tokens],
    type_names: &[Tokens],
    offset: &Tokens,
    xfield_names: Option<&[Ident]>,
) -> Tokens {
    let product = num_variants_product(type_names);

    // to_discr(self): offset + product * value
    if let Some(xfield_names) = xfield_names {
        // Named variant, we must use self.x0, self.x1, etc
        quote! {
            ( #offset )
        #(
                + #product *
            <#type_names as ::enum_like::EnumLike>::to_discr(#xfield_names)
        )*
            }
    } else {
        // Unnamed variant, we must use self.0, self.1 syntax
        quote! {
            ( #offset )
        #(
                + #product *
            <#type_names as ::enum_like::EnumLike>::to_discr(self.#field_names)
        )*
            }
    }
}

fn generate_struct_with_fields(
    name: &Ident,
    generics: Generics,
    fields: &Punctuated<Field, Comma>,
) -> Tokens {
    let elements = fields.len();
    match elements {
        0 => generate_unit_struct_impl(name, generics, false),
        _ => {
            let type_names: Vec<Tokens> = fields
                .iter()
                .map(|f| f.ty.clone().into_tokens())
                .collect();
            let mut field_names: Vec<Tokens> = vec![];
            for (i, n) in fields.iter().enumerate() {
                if let Some(x) = n.ident {
                    // Named struct
                    field_names.push(x.clone().into_tokens());
                } else {
                    // Unnamed struct, the names are 0, 1, 2, ...
                    let i = syn::Index::from(i);
                    field_names.push(quote!(#i));
                }
            }

            generate_struct_many_elem(name, generics, &field_names, &type_names)
        }
    }
}

fn generate_struct_code(
    name: &Ident,
    generics: Generics,
    fields: &Fields,
) -> Tokens {
    match *fields {
        // Unit struct, just one variant
        // struct S; (it's not the same as struct S {} or struct S())
        Fields::Unit => generate_unit_struct_impl(name, generics, true),
        Fields::Named(ref f) => {
            generate_struct_with_fields(name, generics, &f.named)
        }
        Fields::Unnamed(ref f) => {
            generate_struct_with_fields(name, generics, &f.unnamed)
        }
    }
}

// Add a bound `T: EnumLike` to every type parameter T.
fn add_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param
                .bounds
                .push(parse_quote!(::enum_like::EnumLike));
        }
    }
    generics
}

// This is a separate function because I'm not sure if ::enum_like::EnumLike
// is always valid syntax
fn impl_enum_like(name: &Ident, generics: Generics, body: Tokens) -> Tokens {
    let generics = add_trait_bounds(generics);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    quote! {
        unsafe impl #impl_generics ::enum_like::EnumLike for #name #ty_generics
        #where_clause {
            #body
        }
    }
}

/// Function that implements the `#[derive(EnumLike)]` proc macro
#[proc_macro_derive(EnumLike)]
pub fn derive_enum_like(input: TokenStream) -> TokenStream {
    let input: DeriveInput = syn::parse(input).unwrap();
    match input.data {
        Data::Enum(DataEnum {
            ref variants, ..
        }) => generate_enum_code(&input.ident, input.generics, variants),
        Data::Struct(DataStruct { ref fields, .. }) => {
            generate_struct_code(&input.ident, input.generics, fields)
        }
        Data::Union(..) => {
            panic!("#[derive(EnumLike)] is only defined for enums and structs")
        }
    }.into()
}

// The tests are in the ../example crate
