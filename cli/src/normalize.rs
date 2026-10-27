use open_mtr_core::openmtr::domain::{Mtr, Residuo};
use serde::Serialize;
use std::collections::BTreeMap;

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
    #[serde(default)]
    pub classe_residuo: String,
    #[serde(default)]
    pub observacoes_residuos: String,
    pub qtd_residuo_esperada: f64,
    pub qtd_residuo_recebida: Option<f64>,
    pub access_priority: u8,
}

impl DumpRow {
    pub fn from_mtr(account: &str, system: &str, access_priority: u8, mtr: Mtr) -> Self {
        Self::from_mtr_with_classes(account, system, access_priority, mtr, &BTreeMap::new())
    }

    pub fn from_mtr_with_classes(
        account: &str,
        system: &str,
        access_priority: u8,
        mtr: Mtr,
        classes: &BTreeMap<u32, String>,
    ) -> Self {
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
            residuos_json: residuos_json_detalhado(&mtr.residuos),
            classe_residuo: classe_residuo(&mtr.residuos, classes),
            observacoes_residuos: observacoes_residuos(&mtr.residuos),
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

fn residuos_json_detalhado(residuos: &[Residuo]) -> String {
    let detalhado: Vec<serde_json::Value> = residuos
        .iter()
        .map(|r| {
            serde_json::json!({
                "codigo_ibama": r.codigo_ibama,
                "descricao": r.descricao,
                "quantidade": r.quantidade,
                "quantidade_recebida": r.quantidade_recebida,
                "unidade": r.unidade,
                "estado_fisico": r.estado_fisico,
                "classe": r.classe,
                "acondicionamento": r.acondicionamento,
                "tratamento": r.tratamento,
                "densidade": r.densidade,
                "numero_onu": r.numero_onu,
                "classe_risco": r.classe_risco,
                "nome_embarque": r.nome_embarque,
                "grupo_embalagem": r.grupo_embalagem,
                "codigo_interno": r.codigo_interno,
                "descricao_interna": r.descricao_interna,
                "observacao": r.observacao,
            })
        })
        .collect();
    serde_json::to_string(&detalhado).unwrap_or_else(|_| "[]".to_string())
}

fn classe_residuo(residuos: &[Residuo], classes: &BTreeMap<u32, String>) -> String {
    let mut nomes: Vec<String> = Vec::new();
    for r in residuos {
        let Some(c) = r.classe else { continue };
        let nome = classes
            .get(&c)
            .cloned()
            .unwrap_or_else(|| format!("CLASSE {c}"));
        if !nomes.contains(&nome) {
            nomes.push(nome);
        }
    }
    nomes.join(" | ")
}

fn push_observacao(obs: &mut Vec<String>, valor: Option<&String>) {
    let Some(valor) = valor else { return };
    let v = valor.trim();
    if !v.is_empty() && !obs.iter().any(|x| x == v) {
        obs.push(v.to_string());
    }
}

fn observacoes_residuos(residuos: &[Residuo]) -> String {
    let mut obs: Vec<String> = Vec::new();
    for r in residuos {
        push_observacao(&mut obs, r.descricao.as_ref());
        push_observacao(&mut obs, r.descricao_interna.as_ref());
        push_observacao(&mut obs, r.observacao.as_ref());
    }
    obs.join(" | ")
}

fn round4(v: f64) -> f64 {
    (v * 10_000.0).round() / 10_000.0
}

pub const CSV_HEADER: &str = "OTR,MTR_NUMERO,DATA_EMISSAO,DATA_RECEBIMENTO,QNT_RECEBIDA,QNT_ESPERADA,STATUS,OBSERVACOES,OBSERVACOES_RESIDUOS,GERADOR_CNPJ,GERADOR_NOME,DESTINADOR_CNPJ,DESTINADOR_NOME,MOTORISTA,PLACA,CLASSE_RESIDUO,RESIDUO_JSON_DETALHADO,SISTEMA";

pub fn csv_line(row: &DumpRow) -> String {
    let emissao = row
        .data_criacao
        .as_deref()
        .or(row.data_expedicao.as_deref());
    let fields: Vec<String> = vec![
        "-".to_string(),
        csv_escape(&row.numero),
        emissao.map(csv_escape).unwrap_or_default(),
        opt_str(&row.data_recebimento),
        row.qtd_residuo_recebida
            .map(|v| format!("{v}"))
            .unwrap_or_default(),
        format!("{}", row.qtd_residuo_esperada),
        csv_escape(&row.status),
        opt_str(&row.observacao),
        csv_escape(&row.observacoes_residuos),
        opt_str(&row.gerador_documento),
        opt_str(&row.gerador_razao_social),
        opt_str(&row.destinador_documento),
        opt_str(&row.destinador_razao_social),
        opt_str(&row.motorista),
        opt_str(&row.placa),
        csv_escape(&row.classe_residuo),
        csv_escape(&row.residuos_json),
        csv_escape(&row.system),
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
