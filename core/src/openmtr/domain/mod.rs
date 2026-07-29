#![allow(clippy::module_name_repetitions)]

pub mod cancelamento;
pub mod cdf;
pub mod criacao;
pub mod download;
pub mod modificacao;
pub mod mtr;
pub mod newtypes;
pub mod parte;
pub mod recebimento;
pub mod residuo;
pub mod tabela;
pub mod tipo;

pub use cancelamento::{CancelarMtr, MtrCancelado};
pub use cdf::{Cdf, CdfEmitido, FiltroCdf, GerarCdf};
pub use criacao::{CriarMtr, MtrCriado};
pub use download::{DownloadCdf, DownloadItem};
pub use modificacao::{ModificacaoTipo, ModificarMtr, MtrModificado};
pub use mtr::{FiltroMtr, FiltroParte, Mtr, Periodo, StatusMtr};
pub use newtypes::{
    Cnpj, CodigoIbama, Cpf, CpfCnpj, DataMtr, PlacaVeiculo, Quantidade, TimestampMs,
};
pub use parte::Parte;
pub use recebimento::{MtrRecebido, ReceberMtr};
pub use residuo::{Residuo, ResiduoCriacao, ResiduoIbama, ResiduoRecebido};
pub use tabela::{
    Acondicionamento, ClasseResiduo, EstadoFisico, TratamentoResiduo, UnidadeMedida,
    COMPATIBILIDADE_ACOND, ESTADOS_FISICOS,
};
pub use tipo::{FormatoDownload, TipoManifesto, TipoParte};
