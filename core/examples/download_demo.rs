//! Demonstração de download de MTRs do SINIR e MTR-PE.
//!
//! Usa a biblioteca open-mtr-core para consultar e baixar MTRs reais
//! de ambos os sistemas, usando credenciais do .env.
//!
//! Uso:
//!   cargo run --example download_demo
//!
//! O arquivo .env na raiz do projeto deve conter:
//!   SINIR_REFRESH_TOKEN=...           # Token da conta SINIR
//!   MTR_PE_CNPJ=...                   # CNPJ da empresa no MTR-PE
//!   MTR_PE_CPF_USER=...               # CPF do usuário MTR-PE
//!   MTR_PE_PASSWORD=...               # Senha do usuário MTR-PE
//!   MTR_PE_UNITY=...                  # Código da unidade MTR-PE

use std::path::Path;

use chrono::Utc;
use open_mtr_core::*;

fn load_env() {
    dotenvy::from_filename_override("../.env").ok();
    dotenvy::from_filename_override(".env").ok();
}

fn env(key: &str) -> Option<String> {
    load_env();
    std::env::var(key).ok()
}

// ─── Helpers ─────────────────────────────────────────────────────────────

fn salvar_download(item: &DownloadItem, dir: &Path) {
    let caminho = dir.join(&item.filename);
    std::fs::create_dir_all(dir).expect("criar diretório");
    std::fs::write(&caminho, &item.buffer).expect("salvar arquivo");
    println!("   ✅ Salvo: {} ({} bytes)", caminho.display(), item.buffer.len());
}

fn reportar_erros(erros: &[OpenMTRError]) {
    for erro in erros {
        println!("   ❌ [{}] {}", erro.status, erro.detail);
    }
}

/// Tenta consultar MTRs com janelas de 30 dias para trás (até 180 dias).
/// Usa o FiltroMtr com periodo e tenta diferentes tipoParceiro (Gerador, Transportador, Destinador).
async fn descobrir_mtrs_por_periodo(
    client: &OpenMtrClient,
) -> Vec<String> {
    let hoje = Utc::now().date_naive();

    // Tenta com cada tipo de parceiro: Gerador (padrão), Transportador, Destinador
    use TipoParte as Tp;
    for tipo in [None::<Tp>, Some(Tp::Transportador), Some(Tp::Destinador)] {
            for dias in (30..=180).step_by(30) {
                let inicio = hoje - chrono::Duration::days(dias);
                let fim = hoje - chrono::Duration::days(dias - 29);

                let parte = tipo.clone().map(|t| FiltroParte {
                    tipo: t,
                    documento: None,
                    unidade: None,
                });

            let filtro = FiltroMtr {
                periodo: Some(Periodo::new(DataMtr::new(inicio), DataMtr::new(fim)).unwrap()),
                parte,
                pagina: Some(1),
                tamanho: Some(50),
                ..Default::default()
            };

            let nome_tipo = match tipo {
                Some(TipoParte::Transportador) => "Transportador",
                Some(TipoParte::Destinador) => "Destinador",
                _ => "Gerador",
            };

            match client.consultar_mtrs(&filtro).await {
                Ok(resp) if !resp.dados.is_empty() => {
                    let numeros: Vec<String> = resp
                        .dados
                        .iter()
                        .map(|m| m.numero.clone())
                        .collect();
                    println!(
                        "   ✅ Encontrados {} MTRs (tipo={nome_tipo}, período {inicio} a {fim})",
                        numeros.len()
                    );
                    return numeros;
                }
                Ok(_) => {} // sem MTRs nessa janela, continua
                Err(e) => {
                    println!(
                        "   ⚠️  Erro na consulta (tipo={nome_tipo}, {inicio} a {fim}): {e}"
                    );
                }
            }
        }
    }

    Vec::new()
}

// ─── SINIR ───────────────────────────────────────────────────────────────

async fn baixar_do_sinir(token: &str) {
    println!("\n{}", "-".repeat(70));
    println!("  🏭 SINIR Nacional");
    println!("{}", "-".repeat(70));

    let client = match OpenMtrClient::builder()
        .base(System::Sinir)
        .token(token)
        .timeout_secs(30)
        .build()
        .await
    {
        Ok(c) => c,
        Err(e) => {
            println!("   ❌ Erro ao criar cliente SINIR: {e}");
            return;
        }
    };

    let numeros = descobrir_mtrs_por_periodo(&client).await;

    if numeros.is_empty() {
        println!("   ⚠️  Nenhum MTR encontrado nos últimos 180 dias");
        println!(
            "   💡 Defina OPENMTR_SINIR_MTRS no .env com números separados por vírgula"
        );
        println!("   💡 Ex: OPENMTR_SINIR_MTRS=123456,789012");
        return;
    }

    println!("   Baixando {} MTR(s)...", numeros.len());
    match client.download_mtrs(numeros).await {
        Ok(resposta) => {
            let dir = Path::new("/tmp/openmtr-demo/sinir");
            for item in &resposta.dados {
                salvar_download(item, dir);
            }
            reportar_erros(&resposta.erros);
        }
        Err(e) => {
            println!("   ❌ Erro no download: {e}");
        }
    }
}

// ─── MTR-PE ──────────────────────────────────────────────────────────────

async fn autenticar_mtr_pe() -> Option<String> {
    let cnpj = env("MTR_PE_CNPJ")?;
    let cpf = env("MTR_PE_CPF_USER")?;
    let password = env("MTR_PE_PASSWORD")?;
    let unity: u32 = env("MTR_PE_UNITY")?.parse().ok()?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .ok()?;

    let base = System::MtrPe.default_base_url();

    let body = serde_json::json!({
        "pessoaCodigo": unity,
        "pessoaCnpj": cnpj,
        "usuarioCpf": cpf,
        "senha": password,
    });

    let url = format!("{base}/gettoken");
    let resp = client.post(&url).json(&body).send().await.ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    let token = json["token"].as_str()?.to_string();

    println!("   Token obtido: {}...{}", &token[..20], &token[token.len() - 10..]);
    Some(token)
}

async fn descobrir_barcodes_mtr_pe(token: &str) -> Vec<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("reqwest Client");

    let base = System::MtrPe.default_base_url();
    let hoje = Utc::now().date_naive();

    // Tenta janelas de 30 dias nos últimos 180 dias
    for dias in (30..=180).step_by(30) {
        let inicio = hoje - chrono::Duration::days(dias);
        let fim = hoje - chrono::Duration::days(dias - 29);

        let url = format!(
            "{}/retornaListaCodigoBarasManifesto/{}/{}",
            base,
            inicio.format("%Y%m%d"),
            fim.format("%Y%m%d")
        );

        let resp = match client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&serde_json::json!({}))
            .send()
            .await
        {
            Ok(r) => r,
            Err(_) => continue,
        };

        let json: serde_json::Value = match resp.json().await {
            Ok(j) => j,
            Err(_) => continue,
        };

        let codigos: Vec<String> = json["codigos"]
            .as_array()
            .map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect())
            .unwrap_or_default();

        if !codigos.is_empty() {
            println!(
                "   ✅ Encontrados {} códigos de barras em {inicio} a {fim}",
                codigos.len()
            );
            return codigos;
        }
    }

    Vec::new()
}

async fn baixar_do_mtr_pe(token: &str) {
    println!("\n{}", "-".repeat(70));
    println!("  🏭 MTR Pernambuco");
    println!("{}", "-".repeat(70));

    let barcodes = descobrir_barcodes_mtr_pe(token).await;

    if barcodes.is_empty() {
        println!("   ⚠️  Nenhum código de barras encontrado nos últimos 180 dias");
        return;
    }

    let client = match OpenMtrClient::builder()
        .base(System::MtrPe)
        .token(token)
        .timeout_secs(30)
        .build()
        .await
    {
        Ok(c) => c,
        Err(e) => {
            println!("   ❌ Erro ao criar cliente MTR-PE: {e}");
            return;
        }
    };

    // Baixa no máximo 5 MTRs
    let nums: Vec<String> = barcodes.into_iter().take(5).collect();
    println!("   Baixando {} MTR(s)...", nums.len());

    match client.download_mtrs(nums).await {
        Ok(resposta) => {
            let dir = Path::new("/tmp/openmtr-demo/mtr-pe");
            for item in &resposta.dados {
                salvar_download(item, dir);
            }
            reportar_erros(&resposta.erros);
        }
        Err(e) => {
            println!("   ❌ Erro no download: {e}");
        }
    }
}

// ─── Main ────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    println!("{}", "=".repeat(70));
    println!("  open-mtr — Download de MTRs");
    println!("{}", "=".repeat(70));

    // ─── SINIR ──────────────────────────────────────────────────────────
    match env("SINIR_REFRESH_TOKEN") {
        Some(token) => baixar_do_sinir(&token).await,
        None => println!("\n⚠️  SINIR_REFRESH_TOKEN não definido — pulando SINIR"),
    }

    // ─── MTR-PE ─────────────────────────────────────────────────────────
    match autenticar_mtr_pe().await {
        Some(token) => baixar_do_mtr_pe(&token).await,
        None => println!("\n⚠️  MTR_PE_CNPJ/CPF/PASSWORD/UNITY não definidos — pulando MTR-PE"),
    }

    // ─── Resumo ─────────────────────────────────────────────────────────
    println!("\n{}", "=".repeat(70));
    println!("  📁 Arquivos salvos em: /tmp/openmtr-demo/");
    println!("{}", "=".repeat(70));
}
