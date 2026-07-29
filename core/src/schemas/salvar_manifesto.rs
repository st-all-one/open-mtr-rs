use serde::{Deserialize, Serialize};

use crate::schemas::cpf::CpfCnpj;

#[derive(Debug, Serialize)]
pub struct SalvarManifestoLoteRequest {
    #[serde(rename = "possuiArmazenamentoTemporario")]
    pub possui_armazenamento_temporario: bool,
    #[serde(rename = "nomeResponsavel")]
    pub nome_responsavel: String,
    pub gerador: ParteManifesto,
    pub transportador: ParteManifesto,
    pub destinador: ParteManifesto,
    #[serde(rename = "armazenadorTemporario", skip_serializing_if = "Option::is_none")]
    pub armazenador_temporario: Option<ParteManifesto>,
    #[serde(rename = "nomeMotorista", skip_serializing_if = "Option::is_none")]
    pub nome_motorista: Option<String>,
    #[serde(rename = "placaVeiculo", skip_serializing_if = "Option::is_none")]
    pub placa_veiculo: Option<String>,
    #[serde(rename = "dataExpedicao", skip_serializing_if = "Option::is_none")]
    pub data_expedicao: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observacoes: Option<String>,
    #[serde(rename = "tipoManifesto", skip_serializing_if = "Option::is_none")]
    pub tipo_manifesto: Option<i32>,
    #[serde(rename = "simCodigo", skip_serializing_if = "Option::is_none")]
    pub sim_codigo: Option<i32>,
    #[serde(rename = "listaManifestoResiduos")]
    pub lista_residuos: Vec<ResiduoManifestoRequest>,
}

#[derive(Debug, Serialize)]
pub struct ParteManifesto {
    #[serde(rename = "cpfCnpj")]
    pub cpf_cnpj: CpfCnpj,
    pub unidade: i32,
}

#[derive(Debug, Serialize)]
pub struct ResiduoManifestoRequest {
    #[serde(rename = "resCodigoIbama")]
    pub res_codigo_ibama: String,
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
    #[serde(rename = "marDescricaoInterna", skip_serializing_if = "Option::is_none")]
    pub descricao_interna: Option<String>,
    #[serde(rename = "marCodigoInterno", skip_serializing_if = "Option::is_none")]
    pub codigo_interno: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SalvarManifestoResponse {
    #[serde(rename = "erroNacional")]
    pub erro_nacional: bool,
    #[serde(rename = "mensagemErroNacional")]
    pub mensagem_erro: Option<String>,
    #[serde(rename = "respostaApiwsManifestoDTO")]
    pub respostas: Option<Vec<RespostaManifestoLote>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RespostaManifestoLote {
    #[serde(rename = "codigoGerado", default)]
    pub codigo_gerado: Option<i64>,
    #[serde(rename = "manifestoNumeroNacional", default)]
    pub manifesto_numero: Option<String>,
    #[serde(rename = "restResponseValido")]
    pub valido: bool,
    #[serde(rename = "restResponseMensagem")]
    pub mensagem: String,
    #[serde(rename = "manifestoNumeroEstadual", default)]
    pub manifesto_numero_estadual: Option<String>,
    #[serde(rename = "listaResiduos", default)]
    pub lista_residuos: Option<Vec<RespostaResiduoLote>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespostaResiduoLote {
    #[serde(rename = "resCodigoIbama")]
    pub res_codigo_ibama: String,
    #[serde(rename = "marSequencial")]
    pub sequencial: Option<i32>,
    #[serde(rename = "marQuantidade")]
    pub quantidade: Option<f64>,
    #[serde(rename = "uniCodigo")]
    pub uni_codigo: Option<i32>,
    #[serde(rename = "valido", default)]
    pub valido: Option<bool>,
    #[serde(rename = "mensagem", default)]
    pub mensagem: Option<String>,
    #[serde(rename = "codigoGerado", default)]
    pub codigo_gerado: Option<i64>,
}
