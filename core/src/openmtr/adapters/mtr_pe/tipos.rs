//! Conversão de respostas JSON do MTR-PE para os tipos de domínio.
//!
//! As conversões estão separadas por manifestos, serialização e listas.

pub(crate) mod listas;
pub(crate) mod mtr;
pub(crate) mod serializacao;

pub(super) use listas::{
    json_para_acondicionamento, json_para_classe, json_para_estado_fisico, json_para_mtr_criado,
    json_para_residuo_ibama, json_para_tratamento, json_para_unidade,
};
pub(super) use mtr::json_para_mtr;
pub(super) use serializacao::{criar_mtr_para_json, manifesto_recuperavel, retorno_eh_sucesso};
