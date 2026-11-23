/// URNs de tipo de problema usadas no campo `type` de [`OpenMTRError`](super::OpenMTRError).
///
/// Cada URN é montada uma vez em um `LazyLock` estático com o prefixo
/// `urn:openmtr:error:` e sufixo fixo; as funções devolvem um clone. Assim o
/// valor de cada tipo é estável entre erros da mesma categoria.
pub mod error_types {
    use std::sync::LazyLock;
    use url::Url;

    #[allow(
        clippy::expect_used,
        reason = "URN montada apenas com sufixos literais conhecidos; a sintaxe é infalível"
    )]
    fn make_urn(suffix: &str) -> Url {
        Url::parse(&format!("urn:openmtr:error:{suffix}")).expect("URN de erro do openMTR é válida")
    }

    macro_rules! lazy_urn {
        ($name:ident, $suffix:literal) => {
            static $name: LazyLock<Url> = LazyLock::new(|| make_urn($suffix));
        };
    }

    lazy_urn!(NETWORK, "network");
    lazy_urn!(TIMEOUT, "timeout");
    lazy_urn!(AUTH, "authentication");
    lazy_urn!(VALIDATION, "validation");
    lazy_urn!(CONFIG, "configuration");
    lazy_urn!(NOT_FOUND, "not-found");
    lazy_urn!(API, "api-error");
    lazy_urn!(NOT_SUPPORTED, "not-supported");
    lazy_urn!(INTERNAL, "internal");

    #[must_use]
    /// URN `urn:openmtr:error:network`.
    pub fn network() -> Url {
        NETWORK.clone()
    }
    #[must_use]
    /// URN `urn:openmtr:error:timeout`.
    pub fn timeout() -> Url {
        TIMEOUT.clone()
    }
    #[must_use]
    /// URN `urn:openmtr:error:authentication`.
    pub fn auth() -> Url {
        AUTH.clone()
    }
    #[must_use]
    /// URN `urn:openmtr:error:validation`.
    pub fn validation() -> Url {
        VALIDATION.clone()
    }
    #[must_use]
    /// URN `urn:openmtr:error:configuration`.
    pub fn config() -> Url {
        CONFIG.clone()
    }
    #[must_use]
    /// URN `urn:openmtr:error:not-found`.
    pub fn not_found() -> Url {
        NOT_FOUND.clone()
    }
    #[must_use]
    /// URN `urn:openmtr:error:api-error`.
    pub fn api() -> Url {
        API.clone()
    }
    #[must_use]
    /// URN `urn:openmtr:error:not-supported`.
    pub fn not_supported() -> Url {
        NOT_SUPPORTED.clone()
    }
    #[must_use]
    /// URN `urn:openmtr:error:internal`.
    pub fn internal() -> Url {
        INTERNAL.clone()
    }
}
