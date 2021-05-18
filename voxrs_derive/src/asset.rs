use proc_macro::TokenStream;

use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

pub fn derive_asset(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let name_str = name.to_string();
    let asset_name = Ident::new(name_str.strip_suffix("Asset").unwrap(), Span::call_site());

    let expanded = quote! {
        #[automatically_derived]
        #[async_trait::async_trait]
        impl Asset for #name {
            fn asset_type() -> AssetType
            where
                Self: Sized,
            {
                AssetType::#asset_name
            }

            fn get_asset_type(&self) -> AssetType {
                Self::asset_type()
            }

            async fn load<F: voxrs_types::io::FileSystem>(
                path: &crate::AssetPath,
                manager: &mut crate::AssetManager<F>,
                device: Option<&wgpu::Device>,
                queue: Option<&wgpu::Queue>,
            ) -> Result<Self, crate::handle::AssetLoadError>
            where Self: Sized {
                #name::load_asset(path, manager, device, queue).await
            }
        }
    };
    TokenStream::from(expanded)
}
