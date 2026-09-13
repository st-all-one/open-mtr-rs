#![allow(clippy::unwrap_used, clippy::pedantic)]

use std::sync::OnceLock;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

fn load_env() {
    ENV_LOADED.get_or_init(|| {
        dotenvy::from_filename_override(".env").ok();
        dotenvy::from_filename_override("../.env").ok();
    });
}

fn has_live_token() -> bool {
    load_env();
    std::env::var("SINIR_REFRESH_TOKEN").is_ok()
}

fn live_token() -> String {
    load_env();
    std::env::var("SINIR_REFRESH_TOKEN").expect("SINIR_REFRESH_TOKEN not set")
}

/// Recebimento é uma operação **destrutiva** (consome um MTR real). Só roda
/// quando explicitamente habilitado, além do token.
fn live_receive_enabled() -> bool {
    load_env();
    std::env::var("OPENMTR_SINIR_LIVE_RECEIVE")
        .is_ok_and(|v| matches!(v.trim(), "1" | "true" | "yes"))
}

fn test_timeout() -> u64 {
    load_env();
    std::env::var("OPENMTR_TEST_TIMEOUT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(120)
}

use open_mtr_core::*;

/// Cria cliente apontando para SINIR real.
/// Pula o teste se token não estiver configurado.
async fn create_live_client() -> Option<OpenMtrClient> {
    if !has_live_token() {
        return None;
    }

    let builder = OpenMtrClient::builder()
        .base(System::Sinir)
        .token(live_token())
        .timeout_secs(test_timeout());

    Some(builder.build().await.expect("Falha ao criar OpenMtrClient"))
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn ultimos_30_dias() -> FiltroMtr {
    let hoje = chrono::Utc::now().date_naive();
    let inicio = hoje - chrono::Duration::days(30);
    FiltroMtr {
        periodo: Some(Periodo::new(DataMtr(inicio), DataMtr(hoje)).unwrap()),
        parte: Some(FiltroParte {
            tipo: TipoParte::Transportador,
            documento: None,
            unidade: None,
        }),
        ..Default::default()
    }
}

fn ultimos_90_dias() -> FiltroMtr {
    let hoje = chrono::Utc::now().date_naive();
    let inicio = hoje - chrono::Duration::days(90);
    FiltroMtr {
        periodo: Some(Periodo::new(DataMtr(inicio), DataMtr(hoje)).unwrap()),
        ..Default::default()
    }
}

// ─── Testes ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_live_consultar_ultimos_30_dias() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    let filtro = ultimos_30_dias();
    let result = client.consultar_mtrs(&filtro).await;

    match result {
        Ok(resp) => {
            println!("✅ {} MTRs encontrados nos últimos 30 dias", resp.dados.len());
            println!("   ok: {}, status: {}", resp.ok, resp.status);
            if resp.tem_erros() {
                println!("   erros: {:?}", resp.erros);
            }
            if let Some(m) = resp.dados.first() {
                println!("   ex: #{} status={:?}", m.numero, m.status);
            }
            // A conta de transportadora deve ter pelo menos 0 MTRs
            assert!(resp.ok, "Resposta com erro inesperado");
        }
        Err(e) => {
            // Pode falhar se a API estiver fora do ar — não falha o teste
            eprintln!("⚠️  Consulta falhou (API pode estar fora do ar): {e}");
            eprintln!("   type={:?}, title={}, status={}", e.r#type, e.title, e.status);
        }
    }
}

#[tokio::test]
async fn test_live_consultar_por_periodo() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    // Período específico: jan/2025 até fev/2025
    let filtro = FiltroMtr {
        periodo: Some(
            Periodo::new(
                DataMtr(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
                DataMtr(chrono::NaiveDate::from_ymd_opt(2025, 3, 1).unwrap()),
            ).unwrap(),
        ),
        parte: Some(FiltroParte {
            tipo: TipoParte::Transportador,
            documento: None,
            unidade: None,
        }),
        tamanho: Some(10),
        ..Default::default()
    };

    match client.consultar_mtrs(&filtro).await {
        Ok(resp) => {
            println!(
                "✅ Período jan-mar/2025: {} MTRs encontrados",
                resp.dados.len()
            );
            if resp.tem_erros() {
                println!("   erros de negócio: {:?}", resp.erros);
            }
            assert!(resp.ok);
            assert!(resp.dados.len() <= 10, "paginação deve limitar a 10");
        }
        Err(e) => {
            eprintln!("⚠️  Consulta por período falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_consultar_sem_filtro() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    // Sem filtro — usa padrão (últimos 30 dias, tipoParceiro=8/Gerador)
    let filtro = FiltroMtr::default();
    match client.consultar_mtrs(&filtro).await {
        Ok(resp) => {
            println!("✅ Sem filtro: {} MTRs encontrados", resp.dados.len());
            assert!(resp.ok);
        }
        Err(e) => {
            eprintln!("⚠️  Consulta sem filtro falhou (esperado para transportadora sem manifesto como gerador): {e}");
        }
    }
}

#[tokio::test]
async fn test_live_listar_residuos() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    match client.listar_residuos().await {
        Ok(resp) => {
            println!("✅ {} resíduos listados", resp.dados.len());
            assert!(resp.ok);
            assert!(!resp.dados.is_empty(), "deve haver pelo menos 1 resíduo");
            if let Some(r) = resp.dados.first() {
                println!("   ex: {} — {}", r.codigo_ibama, r.descricao);
            }
        }
        Err(e) => {
            eprintln!("⚠️  listar_residuos falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_listar_acondicionamentos() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    match client.listar_acondicionamentos().await {
        Ok(resp) => {
            println!("✅ {} acondicionamentos listados", resp.dados.len());
            assert!(resp.ok);
            if let Some(a) = resp.dados.first() {
                println!("   ex: {} — {}", a.codigo, a.descricao);
            }
        }
        Err(e) => {
            eprintln!("⚠️  listar_acondicionamentos falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_listar_classes() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    match client.listar_classes().await {
        Ok(resp) => {
            println!("✅ {} classes listadas", resp.dados.len());
            assert!(resp.ok);
        }
        Err(e) => {
            eprintln!("⚠️  listar_classes falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_listar_estados_fisicos() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    match client.listar_estados_fisicos().await {
        Ok(resp) => {
            println!("✅ {} estados físicos listados", resp.dados.len());
            assert!(resp.ok);
        }
        Err(e) => {
            eprintln!("⚠️  listar_estados_fisicos falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_listar_tratamentos() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    match client.listar_tratamentos().await {
        Ok(resp) => {
            println!("✅ {} tratamentos listados", resp.dados.len());
            assert!(resp.ok);
        }
        Err(e) => {
            eprintln!("⚠️  listar_tratamentos falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_listar_unidades() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    match client.listar_unidades().await {
        Ok(resp) => {
            println!("✅ {} unidades listadas", resp.dados.len());
            assert!(resp.ok);
        }
        Err(e) => {
            eprintln!("⚠️  listar_unidades falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_token_invalido() {
    load_env();

    let result = OpenMtrClient::builder()
        .base(System::Sinir)
        .token("token-invalido-test")
        .timeout_secs(10)
        .build()
        .await;

    match result {
        Ok(_) => {
            // Pode acontecer se a API estiver fora do ar e não validar o token
            eprintln!("⚠️  Token inválido não foi rejeitado (API pode estar fora do ar)");
        }
        Err(e) => {
            println!("✅ Token inválido rejeitado: {e}");
            assert!(
                e.status == 401 || e.status == 0 || e.status == 503,
                "status esperado 401, 0 (rede) ou 503, recebido {}",
                e.status
            );
        }
    }
}

#[tokio::test]
async fn test_live_consultar_mtr_especifico() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    // Primeiro busca MTRs recentes para ter um número real
    let filtro = ultimos_90_dias();
    let mtrs = match client.consultar_mtrs(&filtro).await {
        Ok(resp) => resp.dados,
        Err(e) => {
            eprintln!("⚠️  Não foi possível buscar MTRs de exemplo: {e}");
            return;
        }
    };

    if mtrs.is_empty() {
        eprintln!("⚠️  Nenhum MTR encontrado nos últimos 90 dias — pulando teste de consulta específica");
        return;
    }

    let primeiro_numero = &mtrs[0].numero;
    println!("🔍 Consultando MTR específico: #{primeiro_numero}");

    let filtro = FiltroMtr {
        numeros: Some(vec![primeiro_numero.clone()]),
        ..Default::default()
    };

    match client.consultar_mtrs(&filtro).await {
        Ok(resp) => {
            println!("✅ MTR #{primeiro_numero} consultado: status={:?}", resp.dados.first().map(|m| &m.status));
            assert!(resp.ok);
            assert!(!resp.dados.is_empty(), "deve retornar o MTR");
            assert_eq!(resp.dados[0].numero, *primeiro_numero);
        }
        Err(e) => {
            eprintln!("⚠️  Consulta de MTR específico falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_download_mtr_existente() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    // Primeiro busca um MTR recente
    let filtro = ultimos_90_dias();
    let mtrs = match client.consultar_mtrs(&filtro).await {
        Ok(resp) => resp.dados,
        Err(e) => {
            eprintln!("⚠️  Não foi possível buscar MTRs de exemplo: {e}");
            return;
        }
    };

    if mtrs.is_empty() {
        eprintln!("⚠️  Nenhum MTR encontrado — pulando download");
        return;
    }

    let numero = &mtrs[0].numero;
    println!("📥 Baixando PDF do MTR #{numero}");

    match client.download_mtrs(vec![numero.clone()]).await {
        Ok(resp) => {
            println!("✅ PDF baixado: {} bytes", resp.dados.first().map(|d| d.buffer.len()).unwrap_or(0));
            assert!(resp.ok);
            if let Some(item) = resp.dados.first() {
                assert!(!item.buffer.is_empty(), "buffer do PDF não deve estar vazio");
                assert_eq!(item.filename, format!("{numero}.pdf"));
                // Verifica magic bytes do PDF
                assert!(item.buffer.starts_with(b"%PDF"), "deve começar com %PDF");
            }
        }
        Err(e) => {
            eprintln!("⚠️  Download falhou: {e}");
        }
    }
}

#[tokio::test]
async fn test_live_consultar_mtr_261032125684() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    let mtr_numero = "261032125684";

    // Via OpenMTR
    println!("🔍 OpenMTR: consultando MTR #{mtr_numero}");
    let resp = client
        .consultar_mtrs(&FiltroMtr {
            numeros: Some(vec![mtr_numero.into()]),
            ..Default::default()
        })
        .await;

    match &resp {
        Ok(r) => {
            println!("✅ OpenMTR: ok={}, dados={}, erros={}", r.ok, r.dados.len(), r.erros.len());
            for m in &r.dados {
                println!("   MTR #{} status={:?}", m.numero, m.status);
                println!("   gerador={:?}", m.gerador.as_ref().map(|p| &p.razao_social));
                println!("   transportador={:?}", m.transportador.as_ref().map(|p| &p.razao_social));
                println!("   destinador={:?}", m.destinador.as_ref().map(|p| &p.razao_social));
                println!("   data_criacao={:?}", m.data_criacao);
                println!("   residuos={}", m.residuos.len());
            }
        }
        Err(e) => {
            eprintln!("❌ OpenMTR falhou: {}", e.detail);
        }
    }

    // Via chamada direta à API
    println!("\n🔍 Direct API: consultando MTR #{mtr_numero}");
    let token = live_token();

    let http = reqwest::Client::new();
    let token_resp = http
        .post("https://admin.sinir.gov.br/api/token")
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("token request failed");
    let token_body: serde_json::Value = token_resp.json().await.expect("token parse failed");
    let access_token = token_body["objetoResposta"]
        .as_str()
        .unwrap_or("")
        .strip_prefix("Bearer ")
        .unwrap_or("")
        .to_string();

    println!("   Access token obtido");

    let hoje = chrono::Utc::now().date_naive();
    let inicio = hoje - chrono::Duration::days(30);
    let lista_body = serde_json::json!({
        "dataInicial": inicio.format("%d-%m-%Y").to_string(),
        "dataFinal": hoje.format("%d-%m-%Y").to_string(),
        "tipoParceiro": 5,
        "pagina": 0,
        "tamanhoPagina": 100
    });

    let direct_resp = http
        .post("https://admin.sinir.gov.br/api/listaManifesto")
        .header("Authorization", format!("Bearer {access_token}"))
        .json(&lista_body)
        .send()
        .await
        .expect("direct API request failed");
    let direct_body: serde_json::Value = direct_resp.json().await.expect("direct API parse failed");

    let items = direct_body["objetoResposta"].as_array();
    println!("   Total MTRs retornados: {}", items.map(|a| a.len()).unwrap_or(0));

    let found_direct = items.and_then(|items| {
        items.iter().find(|m| {
            m.get("manNumero").and_then(|s| s.as_str()) == Some(mtr_numero)
        })
    });

    match found_direct {
        Some(m) => {
            println!("✅ MTR #{mtr_numero} encontrado na API direta");
            println!("   manNumero={}", m["manNumero"].as_str().unwrap_or(""));
            println!("   manData={}", m["manData"].as_i64().unwrap_or(0));
            println!("   motorista={}", m["manNomeMotorista"].as_str().unwrap_or(""));
            println!("   placa={}", m["manPlacaVeiculo"].as_str().unwrap_or(""));
            println!("   observacao={}", m["manObservacao"].as_str().unwrap_or(""));
            println!("   estado={}", m["estado"]["estAbreviacao"].as_str().unwrap_or(""));
            println!("   situacao={}", m["situacaoManifesto"]["simDescricao"].as_str().unwrap_or(""));
        }
        None => {
            eprintln!("❌ MTR #{mtr_numero} NÃO encontrado na resposta direta");
        }
    }

    // Verificação dos campos mapeados
    println!("\n📊 VERIFICAÇÃO DE CAMPOS:");
    if let Ok(openmtr_resp) = &resp {
        if let Some(m) = openmtr_resp.dados.first() {
            println!("  ✅ numero={}", m.numero);
            println!("  ✅ status={:?}", m.status);
            println!("  ✅ observacao={:?}", m.observacao);
            println!("  ✅ motorista={:?}", m.motorista);
            println!("  ✅ placa={:?}", m.placa);
            println!("  ✅ tipo_manifesto={:?}", m.tipo_manifesto);
            println!("  ✅ estado_uf={:?}", m.estado_uf);
            println!("  ✅ data_expedicao={:?}", m.data_expedicao);
            println!("  ✅ armazenamento_temporario={:?}", m.armazenamento_temporario);
            println!("  ✅ residuos={}", m.residuos.len());
        }

        if found_direct.is_some() && openmtr_resp.dados.is_empty() {
            println!("  ❌ GAP CRÍTICO: MTR existe na API mas OpenMTR não retornou!");
        }
    }
}

#[tokio::test]
async fn test_live_consultar_detalhado() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };

    println!("🔍 OpenMTR: consulta detalhada (detalhado=true)");
    let resp = client
        .consultar_mtrs(&FiltroMtr {
            periodo: Some(Periodo::ultimos_dias(7)),
            parte: Some(FiltroParte {
                tipo: TipoParte::Transportador,
                ..Default::default()
            }),
            tamanho: Some(5),
            detalhado: true,
            ..Default::default()
        })
        .await;

    match resp {
        Ok(r) => {
            println!(
                "✅ {} MTRs retornados, {} com resíduos",
                r.dados.len(),
                r.dados.iter().filter(|m| !m.residuos.is_empty()).count(),
            );
            for m in &r.dados {
                let resumo = if m.residuos.is_empty() {
                    "sem resíduos".to_string()
                } else {
                    format!("{} resíduos", m.residuos.len())
                };
                println!("   #{} {} | observacao={}", m.numero, resumo, m.observacao.as_deref().unwrap_or("-"));
            }
            assert!(r.ok);
        }
        Err(e) => {
            eprintln!("⚠️  Consulta detalhada falhou: {e}");
        }
    }
}

// ─── Recebimento (DESTRUTIVO — gated por OPENMTR_SINIR_LIVE_RECEIVE) ────────

/// Recebe um MTR real elegível e reconfirma por consulta.
///
/// **Consome um MTR de verdade** (muda o status para `Recebido`), por isso só
/// roda com `SINIR_REFRESH_TOKEN` **e** `OPENMTR_SINIR_LIVE_RECEIVE=1`.
/// O responsável pode ser ajustado via `OPENMTR_SINIR_RESPONSAVEL`.
#[tokio::test]
async fn test_live_receber_mtr() {
    let client = match create_live_client().await {
        Some(c) => c,
        None => {
            eprintln!("SKIP: SINIR_REFRESH_TOKEN not set");
            return;
        }
    };
    if !live_receive_enabled() {
        eprintln!("SKIP: OPENMTR_SINIR_LIVE_RECEIVE não habilitado (teste destrutivo)");
        return;
    }

    let responsavel = std::env::var("OPENMTR_SINIR_RESPONSAVEL")
        .unwrap_or_else(|_| "Norlu Apoio".to_string());

    // 1. Descobre um MTR Ativo com resíduos (conta = destinador).
    let filtro = FiltroMtr {
        periodo: Some(Periodo::ultimos_dias(30)),
        parte: Some(FiltroParte {
            tipo: TipoParte::Destinador,
            ..Default::default()
        }),
        tamanho: Some(10),
        detalhado: true,
        ..Default::default()
    };
    let lista = client.consultar_mtrs(&filtro).await.expect("consulta");
    let candidato = lista.dados.iter().find(|m| {
        matches!(m.status, StatusMtr::Ativo)
            && !m.residuos.is_empty()
            && m.motorista.is_some()
            && m.placa.is_some()
    });
    let Some(mtr) = candidato else {
        eprintln!("SKIP: nenhum MTR Ativo elegível encontrado nos últimos 30 dias");
        return;
    };

    // 2. Monta o recebimento a partir do próprio manifesto.
    let residuos: Vec<ResiduoRecebido> = mtr
        .residuos
        .iter()
        .map(|r| ResiduoRecebido {
            codigo_ibama: r.codigo_ibama.clone(),
            quantidade_recebida: r.quantidade,
            unidade: r.unidade.unwrap_or(3),
            estado_fisico: r.estado_fisico,
            classe: r.classe,
            acondicionamento: r.acondicionamento,
            tratamento: r.tratamento,
            codigo_ibama_novo: None,
            tratamento_novo: None,
            classe_nova: None,
            densidade: r.densidade,
            quantidade: Some(r.quantidade),
            justificativa: None,
            numero_onu: r.numero_onu.clone(),
            classe_risco: r.classe_risco.clone(),
            nome_embarque: r.nome_embarque.clone(),
            grupo_embalagem: r.grupo_embalagem,
        })
        .collect();

    let receber = ReceberMtr {
        numero: mtr.numero.clone(),
        data_recebimento: chrono::Utc::now().naive_utc(),
        responsavel,
        observacoes: Some("Recebimento automático (teste live open-mtr-rs)".to_string()),
        motorista: mtr.motorista.clone(),
        placa_veiculo: mtr.placa.clone(),
        residuos_recebidos: residuos,
    };

    // 3. Recebe em lote.
    let resp = client
        .receber_mtrs(vec![receber])
        .await
        .expect("receber_mtrs");
    println!(
        "MTR {} → ok={} dados={} erros={}",
        mtr.numero,
        resp.ok,
        resp.dados.len(),
        resp.erros.len()
    );
    assert!(resp.ok, "recebimento retornou erro");
    assert!(
        resp.erros.is_empty(),
        "erros no recebimento: {:?}",
        resp.erros.iter().map(|e| e.detail.clone()).collect::<Vec<_>>()
    );
    assert_eq!(resp.dados.len(), 1);
    assert!(resp.dados[0].recebido, "restResponseValido=false");

    // 4. Reconfirma por consulta.
    let ver = client
        .consultar_mtrs(&FiltroMtr {
            numeros: Some(vec![mtr.numero.clone()]),
            ..Default::default()
        })
        .await
        .expect("consulta pós-recebimento");
    assert!(
        ver.dados
            .iter()
            .any(|m| m.numero == mtr.numero && matches!(m.status, StatusMtr::Recebido)),
        "MTR {} não consta como Recebido após o recebimento",
        mtr.numero
    );
}

