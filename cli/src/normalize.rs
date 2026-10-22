use open_mtr_core::openmtr::domain::Mtr;
use serde::Serialize;

/// Linha normalizada de dump. Campos fixos para CSV/golden estáveis.
#[derive(Debug, Clone, Serialize, serde::Deserialize)]
pub struct DumpRow {
    pub account: String,
    pub system: String,
    pub numero: String,
    pub barcode: String,
    pub status: String,
    pub tipo_manifesto: Option<i32>,
    pub estado_uf: Option<String>,
    pub data_criacao: Option<String>,
    pub data_expedicao: Option<String>,
    pub data_recebimento: Option<String>,
    pub armazenamento_temporario: Option<bool>,
    pub tem_alteracao_pendente: Option<bool>,
    pub hash: Option<String>,
    pub observacao: Option<String>,
    pub motorista: Option<String>,
    pub placa: Option<String>,
    pub numero_estadual: Option<String>,
    pub codigo_estadual: Option<i64>,
    pub gerador_documento: Option<String>,
    pub gerador_razao_social: Option<String>,
    pub gerador_unidade: Option<u32>,
    pub transportador_documento: Option<String>,
    pub transportador_razao_social: Option<String>,
    pub transportador_unidade: Option<u32>,
    pub destinador_documento: Option<String>,
    pub destinador_razao_social: Option<String>,
    pub destinador_unidade: Option<u32>,
    pub residuos_json: String,
    pub qtd_residuo_esperada: f64,
    pub qtd_residuo_recebida: Option<f64>,
    pub access_priority: u8,
}

impl DumpRow {
    pub fn from_mtr(account: &str, system: &str, access_priority: u8, mtr: Mtr) -> Self {
        let doc = |p: &Option<open_mtr_core::openmtr::domain::Parte>| {
            p.as_ref().map(|x| x.documento.to_string())
        };
        let razao = |p: &Option<open_mtr_core::openmtr::domain::Parte>| {
            p.as_ref().map(|x| x.razao_social.clone())
        };
        let unidade =
            |p: &Option<open_mtr_core::openmtr::domain::Parte>| p.as_ref().and_then(|x| x.unidade);
        let esperada = round4(mtr.residuos.iter().map(|r| r.quantidade).sum());
        let recebidas: Vec<f64> = mtr
            .residuos
            .iter()
            .filter_map(|r| r.quantidade_recebida)
            .collect();
        let recebida = if recebidas.is_empty() {
            None
        } else {
            Some(round4(recebidas.iter().sum()))
        };
        Self {
            account: account.to_string(),
            system: system.to_string(),
            numero: mtr.numero.clone(),
            barcode: mtr.barcode.clone(),
            status: mtr.status.to_string(),
            tipo_manifesto: mtr.tipo_manifesto,
            estado_uf: mtr.estado_uf.clone(),
            data_criacao: mtr.data_criacao.map(|d| d.format("%Y-%m-%d").to_string()),
            data_expedicao: mtr.data_expedicao.map(|d| d.format("%Y-%m-%d").to_string()),
            data_recebimento: mtr
                .data_recebimento
                .map(|d| d.format("%Y-%m-%d").to_string()),
            armazenamento_temporario: mtr.armazenamento_temporario,
            tem_alteracao_pendente: mtr.tem_alteracao_pendente,
            hash: mtr.hash.clone(),
            observacao: mtr.observacao.clone(),
            motorista: mtr.motorista.clone(),
            placa: mtr.placa.clone(),
            numero_estadual: mtr.numero_estadual.clone(),
            codigo_estadual: mtr.codigo_estadual,
            gerador_documento: doc(&mtr.gerador),
            gerador_razao_social: razao(&mtr.gerador),
            gerador_unidade: unidade(&mtr.gerador),
            transportador_documento: doc(&mtr.transportador),
            transportador_razao_social: razao(&mtr.transportador),
            transportador_unidade: unidade(&mtr.transportador),
            destinador_documento: doc(&mtr.destinador),
            destinador_razao_social: razao(&mtr.destinador),
            destinador_unidade: unidade(&mtr.destinador),
            residuos_json: serde_json::to_string(&mtr.residuos)
                .unwrap_or_else(|_| "[]".to_string()),
            qtd_residuo_esperada: esperada,
            qtd_residuo_recebida: recebida,
            access_priority,
        }
    }

    /// Chave de identidade para dedup dentro de um sistema.
    pub fn dedup_key(&self) -> String {
        let base = if self.numero.trim().is_empty() {
            &self.barcode
        } else {
            &self.numero
        };
        format!("{}|{}", self.system, base.trim())
    }
}

fn round4(v: f64) -> f64 {
    (v * 10_000.0).round() / 10_000.0
}

pub const CSV_HEADER: &str = "account,system,numero,barcode,status,tipo_manifesto,estado_uf,data_criacao,data_expedicao,data_recebimento,armazenamento_temporario,tem_alteracao_pendente,hash,observacao,motorista,placa,numero_estadual,codigo_estadual,gerador_documento,gerador_razao_social,gerador_unidade,transportador_documento,transportador_razao_social,transportador_unidade,destinador_documento,destinador_razao_social,destinador_unidade,residuos_json,qtd_residuo_esperada,qtd_residuo_recebida";

pub fn csv_line(row: &DumpRow) -> String {
    let fields: Vec<String> = vec![
        csv_escape(&row.account),
        csv_escape(&row.system),
        csv_escape(&row.numero),
        csv_escape(&row.barcode),
        csv_escape(&row.status),
        opt_num(row.tipo_manifesto),
        opt_str(&row.estado_uf),
        opt_str(&row.data_criacao),
        opt_str(&row.data_expedicao),
        opt_str(&row.data_recebimento),
        opt_bool(row.armazenamento_temporario),
        opt_bool(row.tem_alteracao_pendente),
        opt_str(&row.hash),
        opt_str(&row.observacao),
        opt_str(&row.motorista),
        opt_str(&row.placa),
        opt_str(&row.numero_estadual),
        row.codigo_estadual
            .map(|v| v.to_string())
            .unwrap_or_default(),
        opt_str(&row.gerador_documento),
        opt_str(&row.gerador_razao_social),
        row.gerador_unidade
            .map(|v| v.to_string())
            .unwrap_or_default(),
        opt_str(&row.transportador_documento),
        opt_str(&row.transportador_razao_social),
        row.transportador_unidade
            .map(|v| v.to_string())
            .unwrap_or_default(),
        opt_str(&row.destinador_documento),
        opt_str(&row.destinador_razao_social),
        row.destinador_unidade
            .map(|v| v.to_string())
            .unwrap_or_default(),
        csv_escape(&row.residuos_json),
        format!("{}", row.qtd_residuo_esperada),
        row.qtd_residuo_recebida
            .map(|v| format!("{v}"))
            .unwrap_or_default(),
    ];
    fields.join(",")
}

pub fn escape_csv_field(s: &str) -> String {
    csv_escape(s)
}

fn csv_escape(s: &str) -> String {
    let needs = s.contains(',') || s.contains('"') || s.contains('\n') || s.contains('\r');
    let s = s.replace(['\n', '\r'], " ");
    if needs {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s
    }
}

fn opt_str(v: &Option<String>) -> String {
    v.as_deref().map(csv_escape).unwrap_or_default()
}

fn opt_num(v: Option<i32>) -> String {
    v.map(|x| x.to_string()).unwrap_or_default()
}

fn opt_bool(v: Option<bool>) -> String {
    v.map(|b| b.to_string()).unwrap_or_default()
}
