use std::iter::empty;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DeriveInput, Expr, Lit, LitStr, parse::ParseStream, parse_macro_input};

fn has_attr(attrs: &[Attribute], target_str: &str) -> bool {
    attrs.iter().any(|attr| attr.path().is_ident(target_str))
}

fn parse_label(input: ParseStream) -> syn::Result<(LitStr, Vec<Expr>)> {
    let fmt: LitStr = input.parse()?;
    let mut args = Vec::new();

    while input.parse::<syn::Token![,]>().is_ok() {
        let lit: Expr = input.parse()?;
        args.push(lit);
    }

    Ok((fmt, args))
}

#[proc_macro_derive(Ariadnenum, attributes(message, here, label, config, report))]
pub fn derive_ariadnenum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = input.ident.clone();
    let generics = input.generics.clone();
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let enum_data = match input.data {
        Data::Enum(enum_data) => enum_data,
        _ => {
            return syn::Error::new_spanned(input, "Ariadnenum can only be derived for enums!")
                .to_compile_error()
                .into();
        }
    };

    let match_error_message = {
        let arms = enum_data.variants.iter().filter_map(|variant| {
            let variant_ident = variant.ident.clone();
            match &variant.fields {
                syn::Fields::Named(fields) => {
                    let mut args = Vec::new();
                    for field in &fields.named {
                        let ident = field.ident.clone().unwrap();
                        args.push(quote! { #ident });
                    }

                    for attr in &variant.attrs {
                        if !attr.path().is_ident("message") {
                            continue;
                        }

                        if let Ok((label, exprs)) = attr.parse_args_with(parse_label) {
                            return Some(quote! {
                                #enum_name #ty_generics :: #variant_ident { #(#args,)* } => Some(format!(#label, #(#exprs,)*))
                            });
                        }
                    }
                    None
                }
                syn::Fields::Unnamed(fields) => {
                    let mut args = Vec::new();
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        let ident = format_ident!("arg{}", i);
                        args.push(quote! { #ident });
                    }

                    for attr in &variant.attrs {
                        if !attr.path().is_ident("message") {
                            continue;
                        }

                        if let Ok((label, exprs)) = attr.parse_args_with(parse_label) {
                            return Some(quote! {
                                #enum_name #ty_generics :: #variant_ident (#(#args,)*) => Some(format!(#label, #(#exprs,)*))
                            });
                        }
                    }
                    None
                }
                syn::Fields::Unit => None,
            }
        });

        quote! {
            match self {
                #(#arms,)*
                _ => None
            }
        }
    };

    let match_config = {
        let arms = enum_data.variants.iter().filter_map(|variant| {
            let variant_ident = variant.ident.clone();
            for attr in &variant.attrs {
                if !attr.path().is_ident("config") {
                    continue;
                }
                
                let expr: Result<Expr, syn::Error> = attr.parse_args();
                if let Ok(expr) = expr {
                    return match &variant.fields {
                        syn::Fields::Named(_) => Some(quote! {
                            #enum_name #ty_generics :: #variant_ident { .. } => #expr
                        }),
                        syn::Fields::Unnamed(_) => Some(quote! {
                            #enum_name #ty_generics :: #variant_ident ( .. ) => #expr
                        }),
                        syn::Fields::Unit => None
                    }
                }
            }
            None
        });

        quote! {
            match self {
                #(#arms,)*
                _ => ariadne::Config::new().with_index_type(ariadne::IndexType::Byte)
            }
        }
    };

    let match_report_kind = {
        let arms = enum_data.variants.iter().filter_map(|variant| {
            let variant_ident = variant.ident.clone();
            for attr in &variant.attrs {
                if !attr.path().is_ident("report") {
                    continue;
                }
                
                let expr: Result<Expr, syn::Error> = attr.parse_args();
                if let Ok(expr) = expr {
                    return match &variant.fields {
                        syn::Fields::Named(_) => Some(quote! {
                            #enum_name #ty_generics :: #variant_ident { .. } => #expr
                        }),
                        syn::Fields::Unnamed(_) => Some(quote! {
                            #enum_name #ty_generics :: #variant_ident ( .. ) => #expr
                        }),
                        syn::Fields::Unit => None
                    }
                }
            }
            None
        });

        quote! {
            match self {
                #(#arms,)*
                _ => ariadne::ReportKind::Error
            }
        }
    };

    let match_error_location = {
        let arms = enum_data.variants.iter().filter_map(|variant| {
            let variant_ident = variant.ident.clone();
            match &variant.fields {
                syn::Fields::Named(fields) => {
                    for field in &fields.named {
                        if has_attr(&field.attrs, "here") {
                            let arg = field.ident.clone().unwrap();
                            return Some(quote! {
                                #enum_name #ty_generics :: #variant_ident { #arg, .. } => Some(#arg.clone()),
                            });
                        }
                    }
                    return None;
                },
                syn::Fields::Unnamed(fields) => {
                    let mut patterns = Vec::new();
                    let mut found = false;
                    for field in fields.unnamed.iter() {
                        if !found && has_attr(&field.attrs, "here") {
                            patterns.push(quote! { span, });
                            found = true;
                        } else {
                            patterns.push(quote! { _, });
                        };
                    };
                    if found {
                        Some(quote! {
                            #enum_name #ty_generics :: #variant_ident ( #(#patterns)* ) => Some(span.clone()),
                        })
                    } else {
                        None
                    }
                },
                syn::Fields::Unit => None,
            }
        });

        quote! {
            match self {
                #(#arms)*
                _ => None
            }
        }
    };

    let match_labels = {
        let arms = enum_data.variants.iter().filter_map(|variant| {
            let variant_ident = variant.ident.clone();
            match &variant.fields {
                syn::Fields::Named(fields) => {
                    let mut args = Vec::new();
                    let mut labels = Vec::new();
                    for field in &fields.named {
                        let ident = field.ident.clone().unwrap();
                        args.push(quote! { #ident });
                        for attr in &field.attrs {
                            if !attr.path().is_ident("label") {
                                continue;
                            }

                            if let Ok((label, args)) = attr.parse_args_with(parse_label) {
                                labels.push(quote! {
                                    (format!(#label, #(#args,)*), #ident.clone()),
                                });
                            }
                        }
                    }
                    Some(
                        quote! {
                            #enum_name #ty_generics :: #variant_ident { #(#args,)* } => vec![#(#labels)*]
                        }
                    )
                }
                syn::Fields::Unnamed(fields) => {
                    let mut args = Vec::new();
                    let mut labels = Vec::new();
                    for (i, field) in fields.unnamed.iter().enumerate() {
                        let ident = format_ident!("arg{}", i);
                        args.push(quote! { #ident });
                        for attr in &field.attrs {
                            if !attr.path().is_ident("label") {
                                continue;
                            }

                            if let Ok((label, args)) = attr.parse_args_with(parse_label) {
                                labels.push(quote! {
                                    (format!(#label, #(#args,)*), #ident.clone()),
                                });
                            }
                        }
                    }
                    Some(
                        quote! {
                            #enum_name #ty_generics :: #variant_ident ( #(#args,)* ) => vec![#(#labels)*]
                        }
                    )
                }
                syn::Fields::Unit => None,
            }
        });

        quote! {
            match self {
                #(#arms,)*
                _ => Vec::new()
            }
        }
    };

    quote! {
        impl #impl_generics #enum_name #ty_generics #where_clause {
            pub fn report_kind(&self) -> ariadne::ReportKind {
                #match_report_kind
            }

            pub fn error_location(&self) -> Option<Range<usize>> {
                #match_error_location
            }

            pub fn config(&self) -> ariadne::Config {
                #match_config
            }

            pub fn message(&self) -> Option<String> {
                #match_error_message
            }

            pub fn labels(&self) -> Vec<(String, Range<usize>)> {
                #match_labels
            }

            pub fn report(&self) -> Option<ariadne::Report> {
                if self.error_location().is_none() || self.message().is_none() {
                    return None
                }

                let mut builder = ariadne::Report::build(
                    self.report_kind(),
                    self.error_location().unwrap()
                )
                .with_config(self.config())
                .with_message(self.message().unwrap());

                for (label, span) in self.labels() {
                    builder = builder.with_label(
                        ariadne::Label::new(span)
                        .with_message(label)
                    );
                }

                Some(builder.finish())
            }
        }
    }
    .into()
}
