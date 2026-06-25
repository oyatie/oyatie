#![forbid(unsafe_code)]

pub const MICROSERVICE: &str = "workplace-integration";
pub const PRIMARY_ADR: &str = "ADR-0320";
pub const OPENAPI_VERSION: &str = "3.2.0";
pub const ASYNCAPI_VERSION: &str = "3.1.0";
pub const PROTO_SYNTAX: &str = "proto3";
pub const BNF_V4_ACTION_PREFIX: &str = "workplace-integration";
pub const USECASE_RENAME_ADR: &str = "ADR-0106";

pub mod domain {
    pub const LAYERS: &[Layer] = &[
        Layer::Kernel,
        Layer::Domain,
        Layer::Usecase,
        Layer::App,
        Layer::Adapter,
        Layer::Infrastructure,
        Layer::Rest,
        Layer::Grpc,
        Layer::Worker,
        Layer::Cli,
        Layer::Sdk,
        Layer::Api,
    ];

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    pub enum Layer {
        Kernel,
        Domain,
        Usecase,
        App,
        Adapter,
        Infrastructure,
        Rest,
        Grpc,
        Worker,
        Cli,
        Sdk,
        Api,
    }

    impl Layer {
        pub const fn slug(&self) -> &'static str {
            match self {
                Self::Kernel => "kernel",
                Self::Domain => "domain",
                Self::Usecase => "usecase",
                Self::App => "app",
                Self::Adapter => "adapter",
                Self::Infrastructure => "infrastructure",
                Self::Rest => "rest",
                Self::Grpc => "grpc",
                Self::Worker => "worker",
                Self::Cli => "cli",
                Self::Sdk => "sdk",
                Self::Api => "api",
            }
        }
    }
}

pub use domain::{LAYERS, Layer};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocSetScaffold {
    pub microservice: &'static str,
    pub primary_adr: &'static str,
    pub primitive: &'static str,
    pub architecture_layers: Vec<&'static str>,
}

impl DocSetScaffold {
    pub fn layer_count(&self) -> usize {
        self.architecture_layers.len()
    }

    pub fn includes_layer(&self, layer: Layer) -> bool {
        self.architecture_layers.contains(&layer.slug())
    }
}

pub fn scaffold() -> DocSetScaffold {
    DocSetScaffold {
        microservice: MICROSERVICE,
        primary_adr: PRIMARY_ADR,
        primitive: "WorkplaceAgreement",
        architecture_layers: domain::LAYERS.iter().map(Layer::slug).collect(),
    }
}

pub fn validate_scaffold() -> Result<(), &'static str> {
    let scaffold = scaffold();
    if scaffold.layer_count() != 12 {
        return Err("adr_0105_layer_count");
    }

    for layer in domain::LAYERS {
        match layer.slug() {
            "application" | "iac" | "policy" | "observability" => {
                return Err("adr_0105_non_layer_value");
            }
            _ => {}
        }
    }

    if !scaffold.includes_layer(Layer::Usecase) {
        return Err("adr_0106_usecase_layer_missing");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn declares_contract_versions() {
        assert_eq!(OPENAPI_VERSION, "3.2.0");
        assert_eq!(ASYNCAPI_VERSION, "3.1.0");
        assert_eq!(PROTO_SYNTAX, "proto3");
    }

    #[test]
    fn declares_12_canonical_layers() {
        let layers: Vec<_> = LAYERS.iter().map(Layer::slug).collect();
        assert_eq!(
            layers,
            vec![
                "kernel",
                "domain",
                "usecase",
                "app",
                "adapter",
                "infrastructure",
                "rest",
                "grpc",
                "worker",
                "cli",
                "sdk",
                "api"
            ]
        );
        assert!(!layers.contains(&"application"));
        assert!(!layers.contains(&"iac"));
        assert!(!layers.contains(&"policy"));
        assert!(!layers.contains(&"observability"));
    }

    #[test]
    fn scaffold_validation_walks_12_layer_enum() {
        let descriptor = scaffold();
        assert_eq!(descriptor.layer_count(), 12);
        assert!(descriptor.includes_layer(Layer::Kernel));
        assert!(descriptor.includes_layer(Layer::Usecase));
        assert!(descriptor.includes_layer(Layer::Api));
        assert!(validate_scaffold().is_ok());
    }
}
