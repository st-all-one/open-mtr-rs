use crate::core::errors::MtrError;
use crate::schemas::cnpj::Cnpj;
use crate::schemas::common::DataMtr;
use crate::schemas::cpf::Cpf;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Clone, Serialize)]
pub struct ApiwsListaManifestoDTO {
    #[serde(rename = "dataInicial")]
    pub data_inicial: DataMtr,
    #[serde(rename = "dataFinal")]
    pub data_final: DataMtr,
    #[serde(rename = "tipoParceiro")]
    pub tipo_parceiro: TipoParceiro,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagina: Option<i32>,
    #[serde(rename = "tamanhoPagina", skip_serializing_if = "Option::is_none")]
    pub tamanho_pagina: Option<i32>,
}

#[derive(Debug, Clone, Copy)]
pub enum TipoParceiro {
    Transportador = 5,
    Gerador = 8,
    Destinador = 9,
    Armazenador = 10,
}

impl TipoParceiro {
    #[must_use]
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

impl TryFrom<i32> for TipoParceiro {
    type Error = MtrError;

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            5 => Ok(Self::Transportador),
            8 => Ok(Self::Gerador),
            9 => Ok(Self::Destinador),
            10 => Ok(Self::Armazenador),
            _ => Err(MtrError::validation(format!(
                "tipoParceiro inválido: {v}"
            ))),
        }
    }
}

impl Default for TipoParceiro {
    fn default() -> Self {
        Self::Gerador
    }
}

impl Serialize for TipoParceiro {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_i32(self.as_i32())
    }
}

impl<'de> Deserialize<'de> for TipoParceiro {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let v = i32::deserialize(d)?;
        Self::try_from(v).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}

// --- Sub-structs da listaManifesto ---

#[derive(Debug, Clone, Deserialize)]
pub struct Estado {
    #[serde(rename = "estAbreviacao")]
    pub abreviacao: String,
    #[serde(rename = "estCodigo")]
    pub codigo: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pais {
    #[serde(rename = "paiCodigo")]
    pub codigo: Option<i32>,
    #[serde(rename = "paiDescricao")]
    pub descricao: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParceiroAcesso {
    #[serde(rename = "estCodigo")]
    pub est_codigo: i32,
    #[serde(rename = "paaCodigo")]
    pub paa_codigo: i64,
    #[serde(rename = "paaCpf")]
    pub paa_cpf: Cpf,
    #[serde(rename = "paaNome")]
    pub paa_nome: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SituacaoManifesto {
    #[serde(rename = "simCodigo")]
    pub codigo: i32,
    #[serde(rename = "simDataRecebimento")]
    pub data_recebimento: Option<i64>,
    #[serde(rename = "simDescricao")]
    pub descricao: String,
    #[serde(rename = "simOrdem")]
    pub ordem: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParceiroMTR {
    #[serde(rename = "parCidade")]
    pub cidade: Option<String>,
    #[serde(rename = "parCnpj")]
    pub cnpj: Option<Cnpj>,
    #[serde(rename = "parCodigo")]
    pub codigo: Option<i32>,
    #[serde(rename = "parDescricao")]
    pub descricao: String,
    #[serde(rename = "parEndereco")]
    pub endereco: Option<String>,
    #[serde(rename = "parLicenca")]
    pub licenca: Option<String>,
    #[serde(rename = "parNumeroEndereco")]
    pub numero_endereco: Option<String>,
    #[serde(rename = "parOrgaoEmissor")]
    pub orgao_emissor: Option<String>,
    #[serde(rename = "parUf")]
    pub uf: Option<String>,
    #[serde(rename = "possuiPerfil")]
    pub possui_perfil: Option<bool>,
    #[serde(rename = "possuiPerfilANP")]
    pub possui_perfil_anp: Option<bool>,
    #[serde(rename = "spaCodigo")]
    pub spa_codigo: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestoResiduo {
    #[serde(rename = "resCodigoIbama")]
    pub res_codigo_ibama: String,
    #[serde(rename = "marSequencial")]
    pub sequencial: Option<i32>,
    #[serde(rename = "marQuantidade")]
    pub quantidade: f64,
    #[serde(rename = "uniCodigo")]
    pub uni_codigo: i32,
    #[serde(rename = "tieCodigo")]
    pub tie_codigo: i32,
    #[serde(rename = "claCodigo")]
    pub cla_codigo: i32,
    #[serde(rename = "tiaCodigo")]
    pub tia_codigo: i32,
    #[serde(rename = "traCodigo")]
    pub tra_codigo: i32,
    #[serde(rename = "marDescricaoInterna")]
    pub descricao_interna: Option<String>,
    #[serde(rename = "marCodigoInterno")]
    pub codigo_interno: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestoResiduoVariacao {
    #[serde(rename = "mrvSequencial")]
    pub sequencial: Option<i32>,
    #[serde(rename = "mrvResCodigoIbama")]
    pub res_codigo_ibama: Option<String>,
    #[serde(rename = "mrvQuantidade")]
    pub quantidade: Option<f64>,
    #[serde(rename = "mrvMotivo")]
    pub motivo: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ManifestoResumo {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    #[serde(rename = "manStatus")]
    pub man_status: Option<String>,
    #[serde(rename = "cdfEmitidoNumero")]
    pub cdf_emitido_numero: Option<i64>,
    #[serde(rename = "manCodigo")]
    pub man_codigo: Option<i64>,

    // Nomes textuais (strings simples)
    pub gerador: Option<String>,
    pub transportador: Option<String>,
    pub destinador: Option<String>,

    // Datas
    #[serde(rename = "manData")]
    pub man_data: Option<i64>,
    #[serde(rename = "manDataExpedicao")]
    pub man_data_expedicao: Option<i64>,

    // Textuais
    #[serde(rename = "manHashCode")]
    pub man_hash_code: Option<String>,
    #[serde(rename = "manJustificativaCancelamento")]
    pub man_justificativa_cancelamento: Option<String>,
    #[serde(rename = "manNomeMotorista")]
    pub man_nome_motorista: Option<String>,
    #[serde(rename = "manObservacao")]
    pub man_observacao: Option<String>,
    #[serde(rename = "manPlacaVeiculo")]
    pub man_placa_veiculo: Option<String>,
    #[serde(rename = "manResponsavel")]
    pub man_responsavel: Option<String>,

    // Flags
    #[serde(rename = "possuiArmazenamentoTemporario")]
    pub possui_armazenamento_temporario: Option<bool>,
    #[serde(rename = "temMtrComplementar")]
    pub tem_mtr_complementar: Option<bool>,
    #[serde(rename = "temSolicitacaoAlteracaoRecebimento")]
    pub tem_solicitacao_alteracao_recebimento: Option<bool>,
    #[serde(rename = "temSolicitacaoAlteracaoRecebimentoAberto")]
    pub tem_solicitacao_alteracao_recebimento_aberto: Option<bool>,

    // Inteiros
    #[serde(rename = "tipoManifesto")]
    pub tipo_manifesto: Option<i32>,
    #[serde(rename = "mtrProvisorioNumero")]
    pub mtr_provisorio_numero: Option<String>,

    // Objetos aninhados
    pub estado: Option<Estado>,
    #[serde(rename = "situacaoManifesto")]
    pub situacao_manifesto: Option<SituacaoManifesto>,
    #[serde(rename = "parceiroAcesso")]
    pub parceiro_acesso: Option<ParceiroAcesso>,
    #[serde(rename = "parceiroGerador")]
    pub parceiro_gerador: Option<ParceiroMTR>,
    #[serde(rename = "parceiroTransportador")]
    pub parceiro_transportador: Option<ParceiroMTR>,
    #[serde(rename = "parceiroDestinador")]
    pub parceiro_destinador: Option<ParceiroMTR>,
    #[serde(rename = "parceiroArmazenadorTemporario")]
    pub parceiro_armazenador_temporario: Option<ParceiroMTR>,
    #[serde(rename = "parceiroTransportadorArmazenadorTemporario")]
    pub parceiro_transportador_armazenador_temporario: Option<ParceiroMTR>,

    // Internacional (sempre null neste perfil, mas mapeados)
    #[serde(rename = "paisExportacao")]
    pub pais_exportacao: Option<Pais>,
    #[serde(rename = "paisTransportador")]
    pub pais_transportador: Option<Pais>,

    // Arrays de resíduos
    #[serde(rename = "listaManifestoResiduo", default)]
    pub lista_manifesto_residuo: Option<Vec<ManifestoResiduo>>,
    #[serde(rename = "listaManifestoResiduoVariacao", default)]
    pub lista_manifesto_residuo_variacao: Option<Vec<ManifestoResiduoVariacao>>,
    #[serde(rename = "listaSistema", default)]
    pub lista_sistema: Option<Vec<serde_json::Value>>,

    // Campos sempre null (mapeados como Option para completude)
    #[serde(rename = "maeCodigo")]
    pub mae_codigo: Option<i64>,
    #[serde(rename = "maeNotaFiscal")]
    pub mae_nota_fiscal: Option<String>,
    #[serde(rename = "maePortoDestino")]
    pub mae_porto_destino: Option<String>,
    #[serde(rename = "maePortoOrigem")]
    pub mae_porto_origem: Option<String>,
    #[serde(rename = "maiCodigo")]
    pub mai_codigo: Option<i64>,
    #[serde(rename = "maiDeclaracao")]
    pub mai_declaracao: Option<String>,
    #[serde(rename = "maiEndereco")]
    pub mai_endereco: Option<String>,
    #[serde(rename = "maiMunicipio")]
    pub mai_municipio: Option<String>,
    #[serde(rename = "maiPortoDestino")]
    pub mai_porto_destino: Option<String>,
    #[serde(rename = "maiPortoOrigem")]
    pub mai_porto_origem: Option<String>,
    #[serde(rename = "maiTransportadorInternacional")]
    pub mai_transportador_internacional: Option<String>,
    #[serde(rename = "manChaveNotaFiscal")]
    pub man_chave_nota_fiscal: Option<String>,
    #[serde(rename = "manCodigoEstadual")]
    pub man_codigo_estadual: Option<String>,
    #[serde(rename = "manCodigoMtrComplementar")]
    pub man_codigo_mtr_complementar: Option<i64>,
    #[serde(rename = "manDataRecebimentoArmazenamentoTemporario")]
    pub man_data_recebimento_armazenamento_temporario: Option<i64>,
    #[serde(rename = "manDataTransporte")]
    pub man_data_transporte: Option<i64>,
    #[serde(rename = "manNomeMotoristaArmazenamentoTemporario")]
    pub man_nome_motorista_armazenamento_temporario: Option<String>,
    #[serde(rename = "manNumeroEstadual")]
    pub man_numero_estadual: Option<String>,
    #[serde(rename = "manNumeroNotaFiscal")]
    pub man_numero_nota_fiscal: Option<String>,
    #[serde(rename = "manObservacaoArmazenamentoTemporario")]
    pub man_observacao_armazenamento_temporario: Option<String>,
    #[serde(rename = "manPlacaVeiculoArmazenamentoTemporario")]
    pub man_placa_veiculo_armazenamento_temporario: Option<String>,
    #[serde(rename = "mapCodigo")]
    pub map_codigo: Option<i64>,
    #[serde(rename = "mtrProvisorioDataRecebimento")]
    pub mtr_provisorio_data_recebimento: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CancelaManifestoDTO {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    pub justificativa: String,
}

impl CancelaManifestoDTO {
    #[must_use]
    pub fn new(man_numero: impl Into<String>, justificativa: impl Into<String>) -> Self {
        Self {
            man_numero: man_numero.into(),
            justificativa: justificativa.into(),
        }
    }

    pub fn validate(&self) -> Result<(), MtrError> {
        if self.justificativa.len() < 20 {
            return Err(MtrError::validation(
                "Justificativa deve ter no mínimo 20 caracteres",
            ));
        }
        if self.man_numero.is_empty() {
            return Err(MtrError::validation(
                "Número do manifesto não pode estar vazio",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct CancelamentoResponse {
    #[serde(rename = "manNumero")]
    pub man_numero: String,
    pub status: String,
    #[serde(rename = "dataCancelamento")]
    pub data_cancelamento: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schemas::common::DataMtr;
    use std::convert::TryFrom;

    #[test]
    fn test_tipo_parceiro_serialize() {
        let json = serde_json::to_value(&TipoParceiro::Gerador).unwrap();
        assert_eq!(json, serde_json::json!(8));
    }

    #[test]
    fn test_tipo_parceiro_deserialize() {
        let parsed: TipoParceiro = serde_json::from_value(serde_json::json!(5)).unwrap();
        assert!(matches!(parsed, TipoParceiro::Transportador));
    }

    #[test]
    fn test_tipo_parceiro_invalido() {
        let result: Result<TipoParceiro, _> = serde_json::from_value(serde_json::json!(99));
        assert!(result.is_err());
    }

    #[test]
    fn test_tipo_parceiro_try_from() -> Result<(), Box<dyn std::error::Error>> {
        let tp = TipoParceiro::try_from(8)?;
        assert!(matches!(tp, TipoParceiro::Gerador));
        Ok(())
    }

    #[test]
    fn test_tipo_parceiro_try_from_invalido() {
        let result = TipoParceiro::try_from(99);
        assert!(result.is_err());
    }

    #[test]
    fn test_tipo_parceiro_default() {
        let tp = TipoParceiro::default();
        assert!(matches!(tp, TipoParceiro::Gerador));
    }

    #[test]
    fn test_cancelamento_validacao() -> Result<(), Box<dyn std::error::Error>> {
        let payload = CancelaManifestoDTO::new("00001", "Curta");
        assert!(payload.validate().is_err());

        let payload = CancelaManifestoDTO::new(
            "00001",
            "Transporte cancelado pelo gerador conforme solicitação",
        );
        payload.validate()?;
        Ok(())
    }

    #[test]
    fn test_lista_manifesto_serialize() {
        let filtro = ApiwsListaManifestoDTO {
            data_inicial: DataMtr::new(chrono::NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            data_final: DataMtr::new(chrono::NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()),
            tipo_parceiro: TipoParceiro::Gerador,
            pagina: None,
            tamanho_pagina: None,
        };

        let json = serde_json::to_value(&filtro).unwrap();
        assert_eq!(json["tipoParceiro"], serde_json::json!(8));
        assert_eq!(json["dataInicial"], "01-01-2024");
    }
}
