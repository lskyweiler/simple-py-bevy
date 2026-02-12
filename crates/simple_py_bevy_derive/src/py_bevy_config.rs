extern crate proc_macro;
extern crate quote;
#[cfg(feature = "py-bevy")]
use darling::FromMeta;
use proc_macro::TokenStream;
#[cfg(feature = "py-bevy")]
use quote::ToTokens;

#[cfg(feature = "py-bevy")]
#[derive(Debug, FromMeta)]
#[darling(derive_syn_parse)]
struct ConfigStructArgs {
    #[darling(default)]
    name: Option<String>,
    yaml_env_var: syn::Path,
}

#[cfg(feature = "py-bevy")]
fn yaml_loader_impls(
    args: &ConfigStructArgs,
    new_name: &str,
    ast: &syn::ItemStruct,
) -> proc_macro2::TokenStream {
    let struct_name = &ast.ident;

    let yaml_env_var = &args.yaml_env_var;
    let mut yaml_env_ident = proc_macro2::TokenStream::new();
    yaml_env_var.to_tokens(&mut yaml_env_ident);

    let info_str = format!(r#"Loaded {} yaml from >> {{:?}}"#, new_name);

    quote::quote! {
        impl #struct_name {
            pub fn new_from_env_yaml_path() -> Result<Self, Box<dyn std::error::Error>> {
                let var = std::env::var(#yaml_env_ident)?;
                Ok(#struct_name::from_yaml(var.into()))
            }

            fn from_yaml(config_yaml_path: std::path::PathBuf) -> Self {
                match std::fs::read_to_string(config_yaml_path.clone()) {
                    Ok(yaml_str) => {
                        let mut cfg = match serde_yaml::from_str::<Self>(&yaml_str)
                        {
                            Ok(cfg) => cfg,
                            Err(what) => panic!("{}", what),
                        };
                        cfg.make_paths_absolute(&config_yaml_path.parent().unwrap().to_path_buf());

                        bevy::log::info!(
                            #info_str,
                            config_yaml_path.clone()
                        );
                        return cfg;
                    }
                    Err(what) => panic!(
                        "Failed to read file {:?}: {}",
                        config_yaml_path.clone(),
                        what
                    ),
                }
            }
        }

        impl simple_py_bevy::UnwrapOrFromYamlEnv<#struct_name> for Option<#struct_name> {
            fn unwrap_or_from_yaml_env(self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                if self.is_some() {
                    Ok(self.unwrap())
                } else {
                    #struct_name::new_from_env_yaml_path()
                }
            }
        }
    }
    .into()
}

#[allow(unused_variables)]
pub(crate) fn py_bevy_config_res_struct_impl(
    args: TokenStream,
    ast: syn::ItemStruct,
) -> TokenStream {
    #[cfg(feature = "py-bevy")]
    {
        let struct_name = &ast.ident;

        let args: ConfigStructArgs = match syn::parse(args) {
            Ok(v) => v,
            Err(e) => {
                return e.to_compile_error().into();
            }
        };

        let new_name = match &args.name {
            Some(n) => format!(r#"{}"#, n),
            None => format!(r#"{}"#, struct_name),
        };

        let yaml_impl_export = yaml_loader_impls(&args, &new_name, &ast);

        quote::quote!(
            #yaml_impl_export
        )
        .into()
    }

    #[cfg(not(feature = "py-bevy"))]
    {
        quote::quote!(
            #[derive(DummyPyO3)]
            #ast
        )
        .into()
    }
}
