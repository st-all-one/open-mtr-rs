#![allow(clippy::module_name_repetitions)]

use std::fmt;

use crate::openmtr::error::OpenMTRError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum System {
    Sinir,
    Sigor,
    MtrPe,
    MtrCe,
    MtrSp,
    MtrRj,
    MtrMg,
    MtrBa,
}

impl System {
    pub fn default_base_url(&self) -> &'static str {
        match self {
            Self::Sinir => "https://admin.sinir.gov.br/api",
            Self::Sigor => "https://sigor.mtr.ce.gov.br/api",
            Self::MtrPe => "https://mtr.pe.gov.br/api",
            Self::MtrCe => "https://mtr.ce.gov.br/api",
            Self::MtrSp => "https://mtr.sp.gov.br/api",
            Self::MtrRj => "https://mtr.rj.gov.br/api",
            Self::MtrMg => "https://mtr.mg.gov.br/api",
            Self::MtrBa => "https://mtr.ba.gov.br/api",
        }
    }

    pub(crate) fn adapter(self) -> Result<Box<dyn crate::openmtr::adapters::MtrAdapter>, OpenMTRError> {
        match self {
            Self::Sinir => Ok(Box::new(crate::openmtr::adapters::sinir::SinirAdapter)),
            other => Err(OpenMTRError::not_supported(
                &format!("adapter para {} ainda não implementado", other.nome()),
            )),
        }
    }

    pub fn nome(&self) -> &'static str {
        match self {
            Self::Sinir => "SINIR Nacional",
            Self::Sigor => "SIGOR",
            Self::MtrPe => "MTR Pernambuco",
            Self::MtrCe => "MTR Ceará",
            Self::MtrSp => "MTR São Paulo",
            Self::MtrRj => "MTR Rio de Janeiro",
            Self::MtrMg => "MTR Minas Gerais",
            Self::MtrBa => "MTR Bahia",
        }
    }
}

impl fmt::Display for System {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.nome())
    }
}

impl std::str::FromStr for System {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sinir" => Ok(Self::Sinir),
            "sigor" => Ok(Self::Sigor),
            "mtrpe" | "mtr_pe" | "mtr-pe" => Ok(Self::MtrPe),
            "mtrce" | "mtr_ce" | "mtr-ce" => Ok(Self::MtrCe),
            "mtrsp" | "mtr_sp" | "mtr-sp" => Ok(Self::MtrSp),
            "mtrrj" | "mtr_rj" | "mtr-rj" => Ok(Self::MtrRj),
            "mtrmg" | "mtr_mg" | "mtr-mg" => Ok(Self::MtrMg),
            "mtrba" | "mtr_ba" | "mtr-ba" => Ok(Self::MtrBa),
            _ => Err(format!("Sistema desconhecido: {s}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_base_urls() {
        assert!(System::Sinir.default_base_url().contains("sinir"));
        assert!(System::Sigor.default_base_url().contains("sigor"));
        assert!(System::MtrPe.default_base_url().contains("mtr.pe"));
        assert!(System::MtrCe.default_base_url().contains("mtr.ce"));
        assert!(System::MtrSp.default_base_url().contains("mtr.sp"));
        assert!(System::MtrRj.default_base_url().contains("mtr.rj"));
        assert!(System::MtrMg.default_base_url().contains("mtr.mg"));
        assert!(System::MtrBa.default_base_url().contains("mtr.ba"));
    }

    #[test]
    fn test_nome() {
        assert_eq!(System::Sinir.nome(), "SINIR Nacional");
        assert_eq!(System::Sigor.nome(), "SIGOR");
    }

    #[test]
    fn test_display() {
        assert_eq!(System::Sinir.to_string(), "SINIR Nacional");
    }

    #[test]
    fn test_from_str() {
        assert_eq!("sinir".parse::<System>(), Ok(System::Sinir));
        assert_eq!("SIGOR".parse::<System>(), Ok(System::Sigor));
        assert_eq!("mtr_pe".parse::<System>(), Ok(System::MtrPe));
        assert_eq!("mtr-ce".parse::<System>(), Ok(System::MtrCe));
        assert_eq!("mtrsp".parse::<System>(), Ok(System::MtrSp));
        assert_eq!("mtr-rj".parse::<System>(), Ok(System::MtrRj));
        assert_eq!("mtr_mg".parse::<System>(), Ok(System::MtrMg));
        assert_eq!("mtrba".parse::<System>(), Ok(System::MtrBa));
    }

    #[test]
    fn test_from_str_invalid() {
        assert!("unknown".parse::<System>().is_err());
    }

    #[test]
    fn test_serde_roundtrip() {
        let s = System::Sinir;
        let json = serde_json::to_string(&s).expect("serialize");
        let des: System = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(s, des);
    }

    #[test]
    fn test_clone_copy_eq() {
        let a = System::Sinir;
        let b = a;
        assert_eq!(a, b);
    }
}
