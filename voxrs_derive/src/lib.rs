mod asset;
use proc_macro::TokenStream;

/// #[derive(Asset)] XxxAsset { ... }
/// generates
///
/// impl Asset for XxxAsset {
///     fn asset_type() -> AssetType
///     where
///         Self: Sized,
///     {
///         AssetType::Xxx
///     }
///     fn get_asset_type(&self) -> AssetType {
///         Self::asset_type()
///     }
/// }
#[proc_macro_derive(Asset)]
pub fn derive_asset(input: TokenStream) -> TokenStream {
    asset::derive_asset(input)
}
