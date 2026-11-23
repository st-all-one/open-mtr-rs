//! Reexportações dos tipos mais usados da API openMTR.
//!
//! Inclui os tipos de domínio, o erro público, o cliente e seu builder, a
//! autenticação clássica, o envelope [`crate::openmtr::resposta::Resposta`], o
//! enum [`crate::openmtr::system::System`] e o relógio/gerador de IDs para
//! testes determinísticos.
pub use crate::openmtr::domain::{
    AceitarAlteracao, Acondicionamento, AlteracaoAceita, COMPATIBILIDADE_ACOND, CancelarMtr, Cdf,
    CdfEmitido, ClasseResiduo, Cnpj, Cpf, CpfCnpj, CriarMtr, CriarMtrComplementar, DataMtr,
    DownloadCdf, DownloadItem, ESTADOS_FISICOS, EstadoFisico, FiltroCdf, FiltroMtr, FiltroParte,
    FormatoDownload, GerarCdf, ModificacaoTipo, ModificarMtr, Mtr, MtrCancelado, MtrCriado,
    MtrModificado, MtrRecebido, Parte, Periodo, ReceberMtr, Residuo, ResiduoCriacao, ResiduoIbama,
    ResiduoRecebido, StatusMtr, TimestampMs, TipoManifesto, TipoParte, TratamentoResiduo,
    UnidadeMedida,
};
pub use crate::openmtr::error::OpenMTRError;
pub use crate::openmtr::open_mtr_auth::{AuthParams, OpenMtrAuthBuilder, OpenMtrClassicAuth};
pub use crate::openmtr::open_mtr_client::{MtrResult, OpenMtrClient, OpenMtrClientBuilder};
pub use crate::openmtr::resposta::Resposta;
pub use crate::openmtr::system::System;

pub use crate::openmtr::time_util::{
    Clock, FixedClock, IdGenerator, SequentialIdGenerator, SystemClock, UuidV7Generator,
    shared_clock, shared_id_generator,
};
