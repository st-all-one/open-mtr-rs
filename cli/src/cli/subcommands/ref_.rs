use clap::Subcommand;

/// Tabelas de referência suportadas (conjunto fechado).
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum RefTable {
    /// Resíduos (`residuo`, alias: `residuos`).
    #[value(alias = "residuos")]
    Residuo,
    /// Classes de resíduos (`classe`, alias: `classes`).
    #[value(alias = "classes")]
    Classe,
    /// Unidades de medida (`unidade`, alias: `unidades`).
    #[value(alias = "unidades")]
    Unidade,
    /// Acondicionamentos (`acondicionamento`, alias: `acondicionamentos`).
    #[value(alias = "acondicionamentos")]
    Acondicionamento,
    /// Estados físicos (`estado-fisico`, aliases: `estado_fisico`, `estados_fisicos`).
    #[value(
        name = "estado-fisico",
        alias = "estado_fisico",
        alias = "estados_fisicos"
    )]
    EstadoFisico,
    /// Tratamentos (`tratamento`, alias: `tratamentos`).
    #[value(alias = "tratamentos")]
    Tratamento,
}

#[derive(Subcommand, Debug)]
/// Ações sobre tabelas de referência.
pub enum RefCmd {
    /// Lista uma tabela de referência.
    Listar {
        /// Tabela a listar: residuo|classe|unidade|acondicionamento|estado-fisico|tratamento.
        #[arg(value_name = "TABELA")]
        table: RefTable,
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
    },
    /// Classes compatíveis com um código IBAMA.
    PorResiduo {
        /// Código IBAMA do resíduo.
        #[arg(value_name = "CODIGO")]
        codigo: String,
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
    },
    /// Acondicionamentos de um estado físico.
    PorEstadoFisico {
        /// Código numérico do estado físico.
        #[arg(value_name = "CODIGO")]
        codigo: String,
        /// ID da conta; padrão quando há uma única conta.
        #[arg(long)]
        account: Option<String>,
    },
}
