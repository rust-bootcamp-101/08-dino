use darling::{
    ast::{Data, Style},
    FromDeriveInput, FromField,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, DeriveInput, GenericParam, Generics, Ident};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct StructData {
    ident: Ident,
    generics: Generics,
    data: Data<(), StructFields>,
}

#[derive(Debug, FromField)]
struct StructFields {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

pub(crate) fn process_from_js(input: DeriveInput) -> TokenStream {
    let (ident, generics, fields) = parse_struct(input);

    let code = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field must have a name");
        let ty = &field.ty;
        quote! {
            let #name: #ty = obj.get(stringify!(#name))?;
        }
    });

    let idents = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field must have a name");
        quote! { #name }
    });

    let generics_clone = generics.clone();
    let (_, ty_generics, _) = generics_clone.split_for_impl();

    let generics = add_from_js_trait_bounds(generics);
    let (impl_generics, _, where_clause) = generics.split_for_impl();
    quote! {
        impl #impl_generics rquickjs::FromJs<'js> for #ident #ty_generics #where_clause
        {
            fn from_js(_ctx: &rquickjs::Ctx<'js>, value: rquickjs::Value<'js>) -> rquickjs::Result<Self> {
                let obj = value.into_object().unwrap();

                #(#code)*

                Ok(#ident {
                    #(#idents),*
                })
            }
        }
    }
}

pub(crate) fn process_into_js(input: DeriveInput) -> TokenStream {
    let (ident, generics, fields) = parse_struct(input);

    let code = fields.iter().map(|field| {
        let name = field.ident.as_ref().expect("Field must have a name");
        quote! {
            obj.set(stringify!(#name), self.#name)?;
        }
    });

    let generics_clone = generics.clone();
    let (_, ty_generics, _) = generics_clone.split_for_impl();

    let generics = add_into_js_trait_bounds(generics);
    let (impl_generics, _, where_clause) = generics.split_for_impl();

    quote! {
        impl #impl_generics rquickjs::IntoJs<'js> for #ident #ty_generics #where_clause {
            fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
                let obj = Object::new(ctx.clone())?;

                #(#code)*

                Ok(obj.into())
            }
        }
    }
}

fn parse_struct(input: DeriveInput) -> (syn::Ident, syn::Generics, Vec<StructFields>) {
    let StructData {
        ident,
        generics,
        data: Data::Struct(fields),
    } = StructData::from_derive_input(&input).expect("Can not parse input")
    else {
        panic!("Only struct is supported");
    };

    let fields = match fields.style {
        Style::Struct => fields.fields,
        _ => panic!("Only named fields are supported"),
    };

    (ident, generics, fields)
}

// TODO: 合并 add_from_js_trait_bounds 和 add_into_js_trait_bounds 这两个函数
// Add a bound `T: Bounds` to every type parameter T.
// copy from: https://github.com/dtolnay/syn/blob/b5a5a8c17737ac7a7b3553ec202626035bfa779c/examples/heapsize/heapsize_derive/src/lib.rs#L37
fn add_from_js_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(rquickjs::FromJs<'js>));
        }
    }
    // add lifetime
    generics.params.push(syn::parse_quote!('js));
    generics
}

fn add_into_js_trait_bounds(mut generics: Generics) -> Generics {
    for param in &mut generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            type_param.bounds.push(parse_quote!(rquickjs::IntoJs<'js>));
        }
    }
    // add lifetime
    generics.params.push(syn::parse_quote!('js));
    generics
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_from_js_with_generics_should_work() {
        let input = r#"
            #[derive(FromJs)]
            pub struct Request<T> {
                pub query: HashMap<String, String>,
                pub params: HashMap<String, String>,
                pub headers: HashMap<String, String>,
                pub method: String,
                pub url: String,
                pub body: Option<T>,
            }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = StructData::from_derive_input(&parsed).unwrap();
        assert_eq!(info.ident.to_string(), "Request");
        let code = process_from_js(parsed);
        println!("{}", code);

        // pub struct Request<T> {
        //     pub query: HashMap<String, String>,
        //     pub params: HashMap<String, String>,
        //     pub headers: HashMap<String, String>,
        //     pub method: String,
        //     pub url: String,
        //     pub body: Option<T>,
        // }

        // impl<'js, T: rquickjs::FromJs<'js>> rquickjs::FromJs<'js> for Request<T> {
        //     fn from_js(
        //         _ctx: &rquickjs::Ctx<'js>,
        //         value: rquickjs::Value<'js>,
        //     ) -> rquickjs::Result<Self> {
        //         let obj = value.into_object().unwrap();
        //         let query: HashMap<String, String> = obj.get(stringify!(query))?;
        //         let params: HashMap<String, String> = obj.get(stringify!(params))?;
        //         let headers: HashMap<String, String> = obj.get(stringify!(headers))?;
        //         let method: String = obj.get(stringify!(method))?;
        //         let url: String = obj.get(stringify!(url))?;
        //         let body: Option<T> = obj.get(stringify!(body))?;
        //         Ok(Request {
        //             query,
        //             params,
        //             headers,
        //             method,
        //             url,
        //             body,
        //         })
        //     }
        // }
    }

    #[test]
    fn process_from_js_should_work() {
        let input = r#"
            #[derive(FromJs)]
            pub struct Request {
                pub query: HashMap<String, String>,
                pub params: HashMap<String, String>,
                pub headers: HashMap<String, String>,
                pub method: String,
                pub url: String,
                pub body: Option<String>,
            }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = StructData::from_derive_input(&parsed).unwrap();
        assert_eq!(info.ident.to_string(), "Request");
        let code = process_from_js(parsed);
        println!("{}", code);

        // pub struct Request {
        //     pub query: HashMap<String, String>,
        //     pub params: HashMap<String, String>,
        //     pub headers: HashMap<String, String>,
        //     pub method: String,
        //     pub url: String,
        //     pub body: Option<String>,
        // }

        // impl<'js> rquickjs::FromJs<'js> for Request {
        //     fn from_js(
        //         _ctx: &rquickjs::Ctx<'js>,
        //         value: rquickjs::Value<'js>,
        //     ) -> rquickjs::Result<Self> {
        //         let obj = value.into_object().unwrap();
        //         let query: HashMap<String, String> = obj.get(stringify!(query))?;
        //         let params: HashMap<String, String> = obj.get(stringify!(params))?;
        //         let headers: HashMap<String, String> = obj.get(stringify!(headers))?;
        //         let method: String = obj.get(stringify!(method))?;
        //         let url: String = obj.get(stringify!(url))?;
        //         let body: Option<String> = obj.get(stringify!(body))?;
        //         Ok(Request {
        //             query,
        //             params,
        //             headers,
        //             method,
        //             url,
        //             body,
        //         })
        //     }
        // }
    }

    #[test]
    fn process_into_js_should_work() {
        let input = r#"
            #[derive(IntoJs)]
            pub struct Response {
                pub status: u16,
                pub headers: HashMap<String, String>,
                pub body: Option<String>,
            }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = StructData::from_derive_input(&parsed).unwrap();
        assert_eq!(info.ident.to_string(), "Response");
        let code = process_into_js(parsed);
        println!("{}", code);

        // pub struct Response {
        //     pub status: u16,
        //     pub headers: HashMap<String, String>,
        //     pub body: Option<String>,
        // }
        // impl<'js> rquickjs::IntoJs<'js> for Response {
        //     fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        //         let obj = Object::new(ctx.clone())?;
        //         obj.set(stringify!(status), self.status)?;
        //         obj.set(stringify!(headers), self.headers)?;
        //         obj.set(stringify!(body), self.body)?;
        //         Ok(obj.into())
        //     }
        // }
    }

    #[test]
    fn process_into_js_with_generics_should_work() {
        let input = r#"
            #[derive(IntoJs)]
            pub struct Response<T> {
                pub status: u16,
                pub headers: HashMap<String, String>,
                pub body: Option<T>,
            }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = StructData::from_derive_input(&parsed).unwrap();
        assert_eq!(info.ident.to_string(), "Response");
        let code = process_into_js(parsed);
        println!("{}", code);

        // pub struct Response<T> {
        //     pub status: u16,
        //     pub headers: HashMap<String, String>,
        //     pub body: Option<T>,
        // }

        // impl<'js, T: rquickjs::IntoJs<'js>> rquickjs::IntoJs<'js> for Response<T> {
        //     fn into_js(self, ctx: &rquickjs::Ctx<'js>) -> rquickjs::Result<rquickjs::Value<'js>> {
        //         let obj = Object::new(ctx.clone())?;
        //         obj.set(stringify!(status), self.status)?;
        //         obj.set(stringify!(headers), self.headers)?;
        //         obj.set(stringify!(body), self.body)?;
        //         Ok(obj.into())
        //     }
        // }
    }
}
