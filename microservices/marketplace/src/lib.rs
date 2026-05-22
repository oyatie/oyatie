#![forbid(unsafe_code)]

pub const MICROSERVICE: &str = "marketplace";
pub const PRIMARY_ADR: &str = "ADR-0314";
pub const OPENAPI_VERSION: &str = "3.2.0";
pub const ASYNCAPI_VERSION: &str = "3.1.0";
pub const PROTO_SYNTAX: &str = "proto3";
pub const BNF_V4_ACTION_PREFIX: &str = "marketplace";

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
    Graphql,
    Worker,
    Cli,
    Sdk,
    Api,
}

impl Layer {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Kernel => "kernel",
            Self::Domain => "domain",
            Self::Usecase => "usecase",
            Self::App => "app",
            Self::Adapter => "adapter",
            Self::Infrastructure => "infrastructure",
            Self::Rest => "rest",
            Self::Grpc => "grpc",
            Self::Graphql => "graphql",
            Self::Worker => "worker",
            Self::Cli => "cli",
            Self::Sdk => "sdk",
            Self::Api => "api",
        }
    }
}

pub mod domain {
    use super::Layer;

    pub const LAYERS: &[Layer] = &[
        Layer::Kernel,
        Layer::Domain,
        Layer::Usecase,
        Layer::App,
        Layer::Adapter,
        Layer::Infrastructure,
        Layer::Rest,
        Layer::Grpc,
        Layer::Graphql,
        Layer::Worker,
        Layer::Cli,
        Layer::Sdk,
        Layer::Api,
    ];
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DocSuiteScaffold {
    pub microservice: &'static str,
    pub primary_adr: &'static str,
    pub primitive: &'static str,
    pub layers: &'static [Layer],
}

pub fn scaffold() -> DocSuiteScaffold {
    DocSuiteScaffold {
        microservice: MICROSERVICE,
        primary_adr: PRIMARY_ADR,
        primitive: "DealSet",
        layers: domain::LAYERS,
    }
}

pub fn validate_scaffold() -> Result<(), &'static str> {
    let scaffold = scaffold();

    if scaffold.layers.len() != 13 {
        return Err("ADR-0105 layer enum must declare exactly 13 layers");
    }

    for (index, layer) in scaffold.layers.iter().enumerate() {
        let slug = layer.as_str();
        if matches!(slug, "application" | "iac" | "policy" | "observability") {
            return Err("obsolete pre-ADR-0106 layer value declared");
        }
        if scaffold.layers[..index].contains(layer) {
            return Err("duplicate ADR-0105 layer declared");
        }
    }

    if !scaffold.layers.contains(&Layer::Usecase) {
        return Err("ADR-0106 usecase layer missing");
    }
    if !scaffold.layers.contains(&Layer::Api) {
        return Err("ADR-0105 api layer missing");
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
    fn declares_13_canonical_layers() {
        assert_eq!(domain::LAYERS.len(), 13);
        assert_eq!(
            domain::LAYERS,
            &[
                Layer::Kernel,
                Layer::Domain,
                Layer::Usecase,
                Layer::App,
                Layer::Adapter,
                Layer::Infrastructure,
                Layer::Rest,
                Layer::Grpc,
                Layer::Graphql,
                Layer::Worker,
                Layer::Cli,
                Layer::Sdk,
                Layer::Api,
            ]
        );

        let slugs: Vec<_> = domain::LAYERS.iter().map(|layer| layer.as_str()).collect();
        assert!(!slugs.contains(&"application"));
        assert!(!slugs.contains(&"iac"));
        assert!(!slugs.contains(&"policy"));
        assert!(!slugs.contains(&"observability"));
    }

    #[test]
    fn validates_scaffold_against_canonical_layers() {
        assert_eq!(scaffold().layers, domain::LAYERS);
        assert_eq!(validate_scaffold(), Ok(()));
    }
}
