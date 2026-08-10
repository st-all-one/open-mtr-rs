#![allow(clippy::unwrap_used, clippy::pedantic)]

use std::sync::OnceLock;

static ENV_LOADED: OnceLock<()> = OnceLock::new();

fn load_env() {
    ENV_LOADED.get_or_init(|| {
        dotenvy::from_filename_override(".env").ok();
        dotenvy::from_filename_override("../.env").ok();
    });
}

fn has_sinir() -> bool {
    load_env();
    std::env::var("OPENMTR_SINIR_TOKEN").is_ok()
}

fn has_mtrpe() -> bool {
    load_env();
    std::env::var("MTR_PE_CNPJ").is_ok()
}

// ─── SINIR ───────────────────────────────────────────────────────────────

async fn consultar_sinir() -> Vec<open_mtr_core::Mtr> {
    use open_mtr_core::*;

    let token = std::env::var("OPENMTR_SINIR_TOKEN").unwrap();

    let client = match OpenMtrClient::builder()
        .base(System::Sinir)
        .token(&token)
        .timeout_secs(120)
        .build()
        .await
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("   SINIR cliente falhou: {e}");
            return Vec::new();
        }
    };

    let filtro = FiltroMtr {
        periodo: Some(Periodo::ultimos_dias(30)),
        parte: Some(FiltroParte {
            tipo: TipoParte::Transportador,
            ..Default::default()
        }),
        tamanho: Some(5),
        detalhado: true,
        ..Default::default()
    };

    let resp = client.consultar_mtrs(&filtro).await;

    match resp {
        Ok(r) => {
            println!("   SINIR: {} MTRs, {} erros", r.dados.len(), r.erros.len());
            for e in &r.erros {
                println!("   erro: {}", e.detail);
            }
            r.dados
        }
        Err(e) => {
            eprintln!("   SINIR falhou: {} — {}", e.title, e.detail);
            Vec::new()
        }
    }
}

// ─── MTR-PE ──────────────────────────────────────────────────────────────

fn client_mtrpe() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .unwrap()
}

async fn token_mtrpe() -> Option<String> {
    let cnpj = match std::env::var("MTR_PE_CNPJ") {
        Ok(v) => v,
        Err(_) => return None,
    };
    let cpf = std::env::var("MTR_PE_CPF_USER").unwrap();
    let password = std::env::var("MTR_PE_PASSWORD").unwrap();
    let unity: u32 = std::env::var("MTR_PE_UNITY").unwrap().parse().unwrap();

    let resp = client_mtrpe()
        .post("https://mtr.cprh.pe.gov.br/api/gettoken")
        .json(&serde_json::json!({
            "pessoaCodigo": unity,
            "pessoaCnpj": cnpj,
            "usuarioCpf": cpf,
            "senha": password,
        }))
        .send()
        .await
        .ok()?;
    let json: serde_json::Value = resp.json().await.ok()?;
    json["token"].as_str().map(String::from)
}

async fn consultar_mtrpe_raw() -> Vec<serde_json::Value> {
    let token = match token_mtrpe().await {
        Some(t) => t,
        None => return Vec::new(),
    };
    let client = client_mtrpe();
    use chrono::Duration;
    let hoje = chrono::Utc::now().date_naive();

    // Tenta janelas de 30 dias até encontrar MTRs
    for meses in 0..6 {
        let fim = hoje - Duration::days(meses as i64 * 30);
        let inicio = fim - Duration::days(29);
        let url = format!(
            "https://mtr.cprh.pe.gov.br/api/retornaListaCodigoBarasManifesto/{}/{}",
            inicio.format("%Y%m%d"),
            fim.format("%Y%m%d")
        );
        let resp: serde_json::Value = client
            .post(&url)
            .header("Authorization", format!("Bearer {token}"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();

        let barcodes: Vec<String> = resp["codigos"]
            .as_array()
            .map(|a| a.iter().filter_map(|c| c.as_str().map(String::from)).collect())
            .unwrap_or_default();

        if !barcodes.is_empty() {
            println!("   MTR-PE: {} barcodes encontrados em {inicio} a {fim}", barcodes.len());

            let take = barcodes.len().min(3);
            let mut results = Vec::new();
            for b in barcodes.into_iter().take(take) {
                let mtr: serde_json::Value = client
                    .post(format!("https://mtr.cprh.pe.gov.br/api/retornaManifesto/{b}"))
                    .header("Authorization", format!("Bearer {token}"))
                    .json(&serde_json::json!({}))
                    .send()
                    .await
                    .unwrap()
                    .json()
                    .await
                    .unwrap();
                results.push(mtr);
            }
            return results;
        }
    }
    Vec::new()
}

fn raw_para_mtr_domain(v: &serde_json::Value) -> open_mtr_core::Mtr {
    // Reimplementa a lógica do json_para_mtr do adapter para usar aqui
    let cnpj_str = |val: &serde_json::Value| -> open_mtr_core::CpfCnpj {
        let s = val.as_str().unwrap_or("");
        if s.len() == 14 {
            open_mtr_core::CpfCnpj::new(s).unwrap_or_else(|_| {
                open_mtr_core::CpfCnpj::Cnpj(open_mtr_core::Cnpj::parse_unchecked(s))
            })
        } else {
            open_mtr_core::CpfCnpj::Cnpj(open_mtr_core::Cnpj::parse_unchecked("00000000000000"))
        }
    };

    let parte = |cnpj: &serde_json::Value, unidade: Option<u32>| -> open_mtr_core::Parte {
        open_mtr_core::Parte {
            documento: cnpj_str(cnpj),
            razao_social: String::new(),
            endereco: None,
            municipio: None,
            uf: None,
            unidade,
        }
    };

    let parse_data = |val: &serde_json::Value| -> Option<chrono::NaiveDate> {
        let s = val.as_str().unwrap_or("");
        if s.is_empty() { None } else { chrono::NaiveDate::parse_from_str(s, "%Y%m%d").ok() }
    };

    let residuos: Vec<open_mtr_core::Residuo> = v["itemManifestoJSONs"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|item| open_mtr_core::Residuo {
                    codigo_ibama: item["residuo"].as_str().unwrap_or_default().to_string(),
                    descricao: None,
                    quantidade: item["quantidade"].as_f64().unwrap_or(0.0),
                    unidade: item["codigoUnidade"].as_i64().map(|n| n as u32),
                    estado_fisico: item["codigoTipoEstado"].as_i64().map(|n| n as u32),
                    classe: item["codigoClasse"].as_i64().map(|n| n as u32),
                    acondicionamento: item["codigoAcondicionamento"].as_i64().map(|n| n as u32),
                    tratamento: item["codigoTecnologia"].as_i64().map(|n| n as u32),
                    quantidade_recebida: None,
                    densidade: None,
                    numero_onu: None,
                    classe_risco: None,
                    nome_embarque: None,
                    grupo_embalagem: None,
                    codigo_interno: None,
                    descricao_interna: None,
                })
                .collect()
        })
        .unwrap_or_default();

    let sit = v["situacaoManifestoCodigo"].as_i64();
    let status = match sit {
        Some(1) => open_mtr_core::StatusMtr::Ativo,
        Some(3) => open_mtr_core::StatusMtr::Recebido,
        Some(4) => open_mtr_core::StatusMtr::Cancelado,
        _ => open_mtr_core::StatusMtr::Outro(format!("{:?}", sit)),
    };

    open_mtr_core::Mtr {
        numero: v["manifestoCodigo"].as_i64().map(|n| n.to_string()).unwrap_or_default(),
        barcode: String::new(),
        status,
        gerador: Some(parte(
            &v["cnpGerador"],
            v["codUnidadeGerador"].as_i64().map(|n| n as u32),
        )),
        transportador: Some(parte(
            &v["cnpTransportador"],
            v["codUnidadeTransportador"].as_i64().map(|n| n as u32),
        )),
        destinador: Some(parte(
            &v["cnpDestinador"],
            v["codUnidadeDestinador"].as_i64().map(|n| n as u32),
        )),
        residuos,
        data_criacao: parse_data(&v["manifData"]),
        data_recebimento: parse_data(&v["manifDataRecebimento"]),
        observacao: v["manifObservacao"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        motorista: v["manifTransportadorNomeMotorista"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        placa: v["manifTransportadorPlacaVeiculo"].as_str().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
        tipo_manifesto: None,
        estado_uf: None,
        data_expedicao: parse_data(&v["manifDataExpedicao"]),
        armazenamento_temporario: v["cnpArmazenador"].as_str().map(|s| !s.is_empty()),
        hash: None,
        tem_alteracao_pendente: None,
        numero_estadual: None,
        codigo_estadual: None,
    }
}

// ─── Comparador ──────────────────────────────────────────────────────────

fn print_mtr(label: &str, m: &open_mtr_core::Mtr) {
    println!("\n╔═══ {label} ═══");
    println!("║ numero:         {}", m.numero);
    println!("║ status:         {}", m.status);
    println!("║ data_criacao:   {:?}", m.data_criacao);
    println!("║ data_expedicao: {:?}", m.data_expedicao);
    println!("║ data_recebimento: {:?}", m.data_recebimento);
    println!("║ armaz_temp:     {:?}", m.armazenamento_temporario);
    println!("║ observacao:     {:?}", m.observacao.as_deref().unwrap_or("—"));
    println!("║ motorista:      {:?}", m.motorista.as_deref().unwrap_or("—"));
    println!("║ placa:          {:?}", m.placa.as_deref().unwrap_or("—"));
    println!("║ hash:           {:?}", m.hash);
    println!("║ tipo_manifesto: {:?}", m.tipo_manifesto);
    println!("║ estado_uf:      {:?}", m.estado_uf);
    println!("║ tem_alt_pend:   {:?}", m.tem_alteracao_pendente);
    println!("║ numero_estadual:{:?}", m.numero_estadual);
    println!("║ codigo_estadual:{:?}", m.codigo_estadual);
    if let Some(ref g) = m.gerador {
        println!("║ gerador:         doc={}, unidade={:?}", g.documento, g.unidade);
    }
    if let Some(ref t) = m.transportador {
        println!("║ transportador:   doc={}, unidade={:?}", t.documento, t.unidade);
    }
    if let Some(ref d) = m.destinador {
        println!("║ destinador:      doc={}, unidade={:?}", d.documento, d.unidade);
    }
    println!("║ residuos ({}):", m.residuos.len());
    for (i, r) in m.residuos.iter().enumerate() {
        println!("║   [{i}] ibama={} qtd={} classe={:?} ef={:?} acond={:?} trat={:?} unid={:?}",
            r.codigo_ibama, r.quantidade, r.classe, r.estado_fisico,
            r.acondicionamento, r.tratamento, r.unidade);
    }
    println!("╚═══════════════════════");
}

#[tokio::test]
async fn test_compare_sinir_vs_mtrpe() {
    println!("═══════════════════════════════════════════════════");
    println!(" COMPARAÇÃO: SINIR vs MTR-PE → Modelo OpenMTR ");
    println!("═══════════════════════════════════════════════════");

    // ─── SINIR ───
    println!("\n📡 SINIR:");
    let sinir_mtrs = if has_sinir() {
        consultar_sinir().await
    } else {
        eprintln!("   SKIP: OPENMTR_SINIR_TOKEN not set");
        Vec::new()
    };

    // ─── MTR-PE ───
    println!("\n📡 MTR-PE:");
    let mtrpe_raw = if has_mtrpe() {
        consultar_mtrpe_raw().await
    } else {
        eprintln!("   SKIP: MTR_PE_CNPJ not set");
        Vec::new()
    };
    let mtrpe_domain: Vec<open_mtr_core::Mtr> = mtrpe_raw.iter().map(raw_para_mtr_domain).collect();

    // ─── Exibir ───
    for (i, m) in sinir_mtrs.iter().enumerate().take(2) {
        print_mtr(&format!("SINIR #{}", i + 1), m);
    }

    for (i, m) in mtrpe_domain.iter().enumerate().take(2) {
        print_mtr(&format!("MTR-PE #{}", i + 1), m);
    }

    // ─── Métricas ───
    println!("\n═══════════════ MÉTRICAS ═══════════════");
    let filled = |m: &open_mtr_core::Mtr| -> usize {
        let mut count = 0;
        if !m.numero.is_empty() { count += 1; }
        if m.gerador.is_some() { count += 1; }
        if m.transportador.is_some() { count += 1; }
        if m.destinador.is_some() { count += 1; }
        if !m.residuos.is_empty() { count += 1; }
        if m.data_criacao.is_some() { count += 1; }
        if m.data_recebimento.is_some() { count += 1; }
        if m.observacao.is_some() { count += 1; }
        if m.motorista.is_some() { count += 1; }
        if m.placa.is_some() { count += 1; }
        if m.tipo_manifesto.is_some() { count += 1; }
        if m.estado_uf.is_some() { count += 1; }
        if m.data_expedicao.is_some() { count += 1; }
        if m.armazenamento_temporario.is_some() { count += 1; }
        if m.hash.is_some() { count += 1; }
        if m.tem_alteracao_pendente.is_some() { count += 1; }
        if m.numero_estadual.is_some() { count += 1; }
        if m.codigo_estadual.is_some() { count += 1; }
        count
    };

    for (i, m) in sinir_mtrs.iter().enumerate().take(2) {
        println!("   SINIR #{}: {}/18 campos preenchidos", i + 1, filled(m));
    }
    for (i, m) in mtrpe_domain.iter().enumerate().take(2) {
        println!("   MTR-PE #{}: {}/18 campos preenchidos", i + 1, filled(m));
    }
}
