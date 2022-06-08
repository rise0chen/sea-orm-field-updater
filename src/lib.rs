use heck::*;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::{Attribute, DeriveInput, Fields, Meta, NestedMeta, Type, Visibility};

#[proc_macro_derive(FieldUpdater, attributes(field_updater))]
pub fn field_updater(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let DeriveInput {
        ident: ty,
        generics,
        data,
        ..
    } = syn::parse(input).unwrap();
    let field_enum_ident = Ident::new(&(ty.to_string() + "Field"), Span::call_site());

    let fields = filter_fields(match data {
        syn::Data::Struct(ref s) => &s.fields,
        _ => panic!("Field can only be derived for structs"),
    });

    let str2col_branch = fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = convert_ident::to_normal_ident(&ident.to_string());
        let ident_ty = Ident::new(&ident_name.to_upper_camel_case(), Span::call_site());
        quote! {
            #ident_name => Some(Column::#ident_ty)
        }
    });
    let field2cv_branch = fields.iter().map(|(_vis, ident, _ty)| {
        let ident_name = convert_ident::to_normal_ident(&ident.to_string());
        let ident_ty = Ident::new(&ident_name.to_upper_camel_case(), Span::call_site());
        quote! {
            #field_enum_ident::#ident(v) => (Column::#ident_ty, Expr::value(v))
        }
    });
    let fields2active_branch = fields.iter().map(|(_vis, ident, _ty)| {
        quote! {
            #field_enum_ident::#ident(v) => model.#ident = Set(v)
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let tokens = quote! {
        impl #impl_generics #ty #ty_generics
            #where_clause
        {
            pub fn str2col(s: &str) -> Option<Column> {
                match s {
                    #(#str2col_branch,)*
                    _x => None,
                }
            }
            pub fn field2cv(field: #field_enum_ident) -> (Column, SimpleExpr) {
                match field {
                    #(#field2cv_branch,)*
                }
            }
            pub fn fields2active(fields: Vec<#field_enum_ident>) -> ActiveModel {
                let mut model = ActiveModel::new();
                for field in fields {
                    match field {
                        #(#fields2active_branch),*
                    }
                }
                model
            }
        }
    };
    tokens.into()
}

fn filter_fields(fields: &Fields) -> Vec<(Visibility, Ident, Type)> {
    fields
        .iter()
        .filter_map(|field| {
            if field
                .attrs
                .iter()
                .find(|attr| has_skip_attr(attr, "struct_field"))
                .is_none()
                && field.ident.is_some()
            {
                let field_vis = field.vis.clone();
                let field_ident = field.ident.as_ref().unwrap().clone();
                let field_ty = field.ty.clone();
                Some((field_vis, field_ident, field_ty))
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
}

const ATTR_META_SKIP: &'static str = "skip";

fn has_skip_attr(attr: &Attribute, path: &'static str) -> bool {
    if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
        if meta_list.path.is_ident(path) {
            for nested_item in meta_list.nested.iter() {
                if let NestedMeta::Meta(Meta::Path(path)) = nested_item {
                    if path.is_ident(ATTR_META_SKIP) {
                        return true;
                    }
                }
            }
        }
    }
    false
}
