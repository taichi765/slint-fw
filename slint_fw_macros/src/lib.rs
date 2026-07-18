use crate::parse::InnerGlobalComponent;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse_macro_input;

mod parse;

/// Attribute used on generated `InnerXxxAdopter` type.
#[proc_macro_attribute]
pub fn slint_fw(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let item = parse_macro_input!(item as InnerGlobalComponent);
    slint_fw_inner(attr.into(), item).into()
}

fn slint_fw_inner(_attr: TokenStream, item: InnerGlobalComponent) -> TokenStream {
    let state_struct_name = format_ident!("{}ScreenStates", &item.name);

    /*let set_callback_handlers = item.callbacks.into_iter().map(|f| {
        let callback_field_name = f.ident;
        let vm_callback_name = format_ident!("on_{}", &callback_field_name);

        quote! {
            let _self = self.0.as_ref();
            #[allow(unused)]
            { *&#inner_adopter_name::FIELD_OFFSETS.#callback_field_name() }
                .apply_pin(_self)
                .set_handler(move |args| #vm_callback_name());
            { *&InnerConnectionFailedAdopter::FIELD_OFFSETS.callback_tracker_retry_connection() }
                .apply_pin(_self)
                .mark_dirty();
        }
    });*/
    let state_struct = {
        let fields = item.properties.into_iter().map(|f| {
            let field_name = f.ident;
            let field_ty = f.ty;
            quote! {
                #field_name: slint_fw::PropertyHandle<#field_ty>,
            }
        });
        quote! {
            pub struct #state_struct_name {
                #(#fields)*
            }
        }
    };
    let viewmodel_trait = {
        let callbacks = item.callbacks.into_iter().map(|f| {
            let fn_name = format_ident!("on_{}", &f.ident);
            let args = f.arg_type.elems.iter().enumerate().map(|(idx, typ)| {
                let arg_name = format_ident!("args_{}", idx);
                quote! {
                    #arg_name: #typ,
                }
            });
            let ret_typ = f.ret_type;
            quote! {
                fn #fn_name(#(#args)*) -> #ret_typ;
            }
        });

        let vm_trait_name = format_ident!("{}ViewModelTrait", &item.name);
        quote! {
            pub trait #vm_trait_name {
                #(#callbacks)*
            }
        }
    };
    /*quote! {
        impl<'a> #inner_adopter_name<'a> {
            pub fn into_state_and_viewmodel(
                self,
                vm: impl #vm_name,
            ) -> {
                // loop over callback fields
                #(#set_callback_handlers)*
            }
        }
    }*/
    let original = item.original;
    quote! {
        #original

        #state_struct

        #viewmodel_trait
    }
}
