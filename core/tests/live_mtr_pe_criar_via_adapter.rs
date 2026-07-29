#![allow(clippy::unwrap_used, clippy::pedantic)]

use std::sync::OnceLock;
use open_mtr_core::*;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

fn load_env() {
    ENV_LOADED.get_or_init(|| {
        dotenvy::from_filename_override(".env").ok();
        dotenvy::from_filename_override("../.env").ok();
    });
}

fn has_env() -> bool {
    load_env();
    std::env::var("MTR_PE_CNPJ").is_ok()
}

fn env(key: &str) -> String {
    load_env();
    std::env::var(key).unwrap_or_default()
}

#[tokio::test]
async fn test_pe_criar_via_adapter() {
    if !has_env() {
        eprintln!("SKIP: MTR_PE_CNPJ not set");
        return;
    }

    println!("🔐 Autenticando no MTR-PE via OpenMtrAuthBuilder...");

    let cnpj_str = env("MTR_PE_CNPJ");
    let cpf = env("MTR_PE_CPF_USER");
    let password = env("MTR_PE_PASSWORD");
    let unity: u32 = env("MTR_PE_UNITY").parse().unwrap();

    // 1. Autentica via CPF+senha+cnpj
    let token = OpenMtrAuthBuilder::default()
        .base(System::MtrPe)
        .credenciais(&cpf, &password, unity)
        .cnpj(&cnpj_str)
        .timeout_secs(60)
        .build()
        .await
        .expect("Falha na autenticação MTR-PE");

    println!("   Token obtido: {}...", &token[..40]);

    // 2. Cria o cliente OpenMTR com o token (que é o JWT do MTR-PE)
    let mtr = OpenMtrClient::builder()
        .base(System::MtrPe)
        .token(&token)
        .timeout_secs(60)
        .build()
        .await
        .expect("Falha ao criar OpenMtrClient para MTR-PE");

    println!("   Cliente OpenMTR (MtrPeAdapter) inicializado ✅");

    // 3. Monta o CriarMtr usando tipos padronizados
    let gerador = Parte {
        documento: CpfCnpj::new(&cnpj_str).unwrap(),
        razao_social: "UBA USINA DE BENEFICIAMENTO AMBIENTAL LTDA".into(),
        endereco: None,
        municipio: None,
        uf: None,
        unidade: Some(unity),
    };

    let transportador = Parte {
        documento: CpfCnpj::new("11201870000145").unwrap(),
        razao_social: "ATR SOLUCOES EM RESIDUOS LTDA".into(),
        endereco: None,
        municipio: None,
        uf: None,
        unidade: Some(917),
    };

    let destinador = Parte {
        documento: CpfCnpj::new(&cnpj_str).unwrap(),
        razao_social: "UBA USINA DE BENEFICIAMENTO AMBIENTAL LTDA".into(),
        endereco: None,
        municipio: None,
        uf: None,
        unidade: Some(unity),
    };

    let residuos = vec![ResiduoCriacao {
        codigo_ibama: "170201".into(),
        quantidade: 1.0,
        unidade: 4,
        estado_fisico: 1,
        classe: 2,
        acondicionamento: 2,
        tratamento: 1,
        descricao_interna: Some("MADEIRA CLASSE B - TESTE VIA ADAPTER".into()),
        codigo_interno: Some("TST-ADP-001".into()),
    }];

    let criar = CriarMtr {
        gerador,
        transportador,
        destinador,
        residuos,
        responsavel: "STALLONE TESTE".into(),
        observacoes: Some("MTR CRIADO VIA OpenMtrClient (adapter MTR-PE)".into()),
        data_expedicao: Some(chrono::Utc::now().naive_utc()),
        placa_veiculo: Some("ABC-1A34".into()),
        motorista: Some("MOTORISTA TESTE".into()),
        armazenador_temporario: None,
        tipo_manifesto: None,
        cargo: Some("ANALISTA".into()),
        codigo_referencia: Some("170201".into()),
    };

    println!("\n📝 Criando MTR via OpenMtrClient::criar_mtrs...");
    println!("   Gerador:      {}", criar.gerador.documento);
    println!("   Transportador: {}", criar.transportador.documento);
    println!("   Destinador:   {}", criar.destinador.documento);
    println!("   Resíduo:      {} qtd: {}", criar.residuos[0].codigo_ibama, criar.residuos[0].quantidade);

    // 4. Executa a criação via adapter
    let result = mtr.criar_mtrs(vec![criar]).await;

    match result {
        Ok(resp) => {
            println!("\n📥 Resposta do adapter:");
            println!("   ok: {}, total dados: {}, erros: {}", resp.ok, resp.dados.len(), resp.erros.len());

            for m in &resp.dados {
                println!(
                    "   ✅ MTR #{} criado (codigo_gerado: {:?}, sucesso: {})",
                    m.numero.as_deref().unwrap_or("?"),
                    m.codigo_gerado,
                    m.sucesso
                );
            }

            for e in &resp.erros {
                println!("   ⚠️  Erro: [{}] {}", e.status, e.detail);
            }

            if let Some(m) = resp.dados.first() {
                if m.sucesso {
                    println!("\n🎉 MTR criado com sucesso via adapter OpenMTR!");
                }
            }
        }
        Err(e) => {
            println!("\n❌ Erro catastrófico: [{}] {} — {}", e.status, e.title, e.detail);
        }
    }
}
