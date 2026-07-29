use std::marker::PhantomData;

use crate::client::http::MtrClient;
use crate::core::errors::MtrError;
use crate::schemas::recebimento::TimestampMs;
use crate::schemas::cpf::CpfCnpj;
use crate::schemas::salvar_manifesto::{
    ParteManifesto, ResiduoManifestoRequest, SalvarManifestoLoteRequest, SalvarManifestoResponse,
};

#[derive(Clone)]
pub struct ManifestoLoteService {
    client: MtrClient,
}

impl ManifestoLoteService {
    #[must_use]
    pub fn new(client: MtrClient) -> Self {
        Self { client }
    }

    pub async fn salvar_lote(
        &self,
        manifestos: &[SalvarManifestoLoteRequest],
    ) -> Result<SalvarManifestoResponse, MtrError> {
        self.client
            .request_with_body(
                reqwest::Method::POST,
                "/salvarManifestoLote",
                manifestos,
                "manifesto.salvar_lote",
            )
            .await
    }
}

pub struct SemResiduos;
pub struct ComResiduos;

#[must_use]
pub struct SalvarManifestoLoteBuilder<State = SemResiduos> {
    nome_responsavel: Option<String>,
    gerador: Option<ParteManifesto>,
    transportador: Option<ParteManifesto>,
    destinador: Option<ParteManifesto>,
    armazenador_temporario: Option<ParteManifesto>,
    nome_motorista: Option<String>,
    placa_veiculo: Option<String>,
    data_expedicao: Option<TimestampMs>,
    observacoes: Option<String>,
    tipo_manifesto: Option<i32>,
    sim_codigo: Option<i32>,
    possui_armazenamento_temporario: bool,
    residuos: Vec<ResiduoManifestoRequest>,
    _state: PhantomData<State>,
}

impl Default for SalvarManifestoLoteBuilder<SemResiduos> {
    fn default() -> Self {
        Self::new()
    }
}

impl SalvarManifestoLoteBuilder<SemResiduos> {
    pub fn new() -> Self {
        Self {
            nome_responsavel: None,
            gerador: None,
            transportador: None,
            destinador: None,
            armazenador_temporario: None,
            nome_motorista: None,
            placa_veiculo: None,
            data_expedicao: None,
            observacoes: None,
            tipo_manifesto: None,
            sim_codigo: None,
            possui_armazenamento_temporario: false,
            residuos: Vec::new(),
            _state: PhantomData,
        }
    }
}

impl<State> SalvarManifestoLoteBuilder<State> {
    pub fn nome_responsavel(mut self, nome: impl Into<String>) -> Self {
        self.nome_responsavel = Some(nome.into());
        self
    }

    pub fn gerador(mut self, cpf_cnpj: CpfCnpj, unidade: i32) -> Self {
        self.gerador = Some(ParteManifesto { cpf_cnpj, unidade });
        self
    }

    pub fn transportador(mut self, cpf_cnpj: CpfCnpj, unidade: i32) -> Self {
        self.transportador = Some(ParteManifesto {
            cpf_cnpj,
            unidade,
        });
        self
    }

    pub fn destinador(mut self, cpf_cnpj: CpfCnpj, unidade: i32) -> Self {
        self.destinador = Some(ParteManifesto {
            cpf_cnpj,
            unidade,
        });
        self
    }

    pub fn armazenador_temporario(mut self, cpf_cnpj: CpfCnpj, unidade: i32) -> Self {
        self.armazenador_temporario = Some(ParteManifesto {
            cpf_cnpj,
            unidade,
        });
        self
    }

    pub fn possui_armazenamento_temporario(mut self, val: bool) -> Self {
        self.possui_armazenamento_temporario = val;
        self
    }

    pub fn nome_motorista(mut self, nome: impl Into<String>) -> Self {
        self.nome_motorista = Some(nome.into());
        self
    }

    pub fn placa_veiculo(mut self, placa: impl Into<String>) -> Self {
        self.placa_veiculo = Some(placa.into());
        self
    }

    pub fn data_expedicao(mut self, data: TimestampMs) -> Self {
        self.data_expedicao = Some(data);
        self
    }

    pub fn observacoes(mut self, obs: impl Into<String>) -> Self {
        self.observacoes = Some(obs.into());
        self
    }

    pub fn tipo_manifesto(mut self, t: i32) -> Self {
        self.tipo_manifesto = Some(t);
        self
    }

    pub fn sim_codigo(mut self, cod: i32) -> Self {
        self.sim_codigo = Some(cod);
        self
    }
}

impl SalvarManifestoLoteBuilder<SemResiduos> {
    pub fn add_residuo(
        mut self,
        res_codigo_ibama: impl Into<String>,
        quantidade: f64,
        uni_codigo: i32,
        tie_codigo: i32,
        cla_codigo: i32,
        tia_codigo: i32,
        tra_codigo: i32,
    ) -> SalvarManifestoLoteBuilder<ComResiduos> {
        self.residuos.push(ResiduoManifestoRequest {
            res_codigo_ibama: res_codigo_ibama.into(),
            quantidade,
            uni_codigo,
            tie_codigo,
            cla_codigo,
            tia_codigo,
            tra_codigo,
            descricao_interna: None,
            codigo_interno: None,
        });
        SalvarManifestoLoteBuilder {
            nome_responsavel: self.nome_responsavel,
            gerador: self.gerador,
            transportador: self.transportador,
            destinador: self.destinador,
            armazenador_temporario: self.armazenador_temporario,
            nome_motorista: self.nome_motorista,
            placa_veiculo: self.placa_veiculo,
            data_expedicao: self.data_expedicao,
            observacoes: self.observacoes,
            tipo_manifesto: self.tipo_manifesto,
            sim_codigo: self.sim_codigo,
            possui_armazenamento_temporario: self.possui_armazenamento_temporario,
            residuos: self.residuos,
            _state: PhantomData,
        }
    }
}

impl SalvarManifestoLoteBuilder<ComResiduos> {
    pub fn add_residuo(
        mut self,
        res_codigo_ibama: impl Into<String>,
        quantidade: f64,
        uni_codigo: i32,
        tie_codigo: i32,
        cla_codigo: i32,
        tia_codigo: i32,
        tra_codigo: i32,
    ) -> Self {
        self.residuos.push(ResiduoManifestoRequest {
            res_codigo_ibama: res_codigo_ibama.into(),
            quantidade,
            uni_codigo,
            tie_codigo,
            cla_codigo,
            tia_codigo,
            tra_codigo,
            descricao_interna: None,
            codigo_interno: None,
        });
        self
    }

    pub fn add_residuo_detalhado(mut self, residuo: ResiduoManifestoRequest) -> Self {
        self.residuos.push(residuo);
        self
    }

    pub fn build(self) -> Result<SalvarManifestoLoteRequest, MtrError> {
        let gerador = self
            .gerador
            .ok_or_else(|| MtrError::validation("gerador é obrigatório"))?;
        let transportador = self
            .transportador
            .ok_or_else(|| MtrError::validation("transportador é obrigatório"))?;
        let destinador = self
            .destinador
            .ok_or_else(|| MtrError::validation("destinador é obrigatório"))?;
        let nome_responsavel = self
            .nome_responsavel
            .ok_or_else(|| MtrError::validation("nome_responsavel é obrigatório"))?;

        Ok(SalvarManifestoLoteRequest {
            possui_armazenamento_temporario: self.possui_armazenamento_temporario,
            nome_responsavel,
            gerador,
            transportador,
            destinador,
            armazenador_temporario: self.armazenador_temporario,
            nome_motorista: self.nome_motorista,
            placa_veiculo: self.placa_veiculo,
            data_expedicao: self.data_expedicao.map(|t| t.0),
            observacoes: self.observacoes,
            tipo_manifesto: self.tipo_manifesto,
            sim_codigo: self.sim_codigo,
            lista_residuos: self.residuos,
        })
    }
}
