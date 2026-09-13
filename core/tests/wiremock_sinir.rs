mod common;

use chrono::{Duration, NaiveDate};
use open_mtr_core::*;
use serde_json::{json, Value};
use wiremock::{Mock, matchers::{method, path}};

#[tokio::test]
async fn test_sinir_consultar_mtr_por_numero() {
    let server = common::setup_sinir_server().await;

    Mock::given(method("GET"))
        .and(path("/retornaManifesto/123456"))
        .respond_with(common::json_response(200, serde_json::json!({
            "erro": false,
            "objetoResposta": {
                "manNumero": "123456",
                "situacaoManifesto": { "simDescricao": "Salvo" },
                "parceiroGerador": { "parCodigo": 1, "parDescricao": "Gerador Ltda" },
                "parceiroTransportador": { "parCodigo": 2, "parDescricao": "Transportador Ltda" },
                "parceiroDestinador": { "parCodigo": 3, "parDescricao": "Destinador Ltda" }
            }
        })))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;

    let resp = client.consultar_mtrs(&FiltroMtr {
        numeros: Some(vec!["123456".into()]),
        ..Default::default()
    }).await;

    assert!(resp.is_ok(), "consulta falhou: {:?}", resp.err());
    let resposta = resp.unwrap();
    assert!(resposta.ok, "resposta não foi ok: {:?}", resposta.erros);
    assert_eq!(resposta.dados.len(), 1);
    assert_eq!(resposta.dados[0].numero, "123456");
}

#[tokio::test]
async fn test_sinir_consultar_mtr_por_periodo() {
    let server = common::setup_sinir_server().await;

    Mock::given(method("POST"))
        .and(path("/listaManifesto"))
        .respond_with(common::json_response(200, serde_json::json!({
            "erro": false,
            "objetoResposta": [
                {
                    "manNumero": "123456",
                    "manStatus": "ATIVO"
                }
            ],
            "totalRecords": 1,
            "mensagem": null
        })))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;

    let resp = client.consultar_mtrs(&FiltroMtr {
        periodo: Some(Periodo::ultimos_dias(30)),
        ..Default::default()
    }).await;

    assert!(resp.is_ok(), "consulta falhou: {:?}", resp.err());
    let resposta = resp.unwrap();
    assert!(resposta.ok, "resposta não foi ok: {:?}", resposta.erros);
    assert_eq!(resposta.dados.len(), 1);
    assert_eq!(resposta.dados[0].numero, "123456");
}

// ─── Recebimento em lote ──────────────────────────────────────────────────

fn receber_exemplo(numero: &str, com_substituicao: bool) -> ReceberMtr {
    let data = NaiveDate::from_ymd_opt(2026, 9, 13)
        .expect("data válida")
        .and_hms_opt(12, 0, 0)
        .expect("hora válida");
    ReceberMtr {
        numero: numero.into(),
        data_recebimento: data,
        responsavel: "Norlu Apoio".into(),
        observacoes: Some("recebimento de teste".into()),
        motorista: Some("WELLINGTON".into()),
        placa_veiculo: Some("RLZ4A36".into()),
        residuos_recebidos: vec![ResiduoRecebido {
            codigo_ibama: "Classe A".into(),
            quantidade_recebida: 5.0,
            unidade: 3,
            estado_fisico: Some(4),
            classe: Some(11),
            acondicionamento: Some(4),
            tratamento: Some(43),
            codigo_ibama_novo: com_substituicao.then(|| "170107".to_string()),
            tratamento_novo: com_substituicao.then_some(43),
            classe_nova: None,
            densidade: None,
            quantidade: Some(5.0),
            justificativa: com_substituicao.then(|| "Substituição RCC".to_string()),
            numero_onu: None,
            classe_risco: None,
            nome_embarque: None,
            grupo_embalagem: None,
        }],
    }
}

#[tokio::test]
async fn test_sinir_receber_lote_envia_array_e_campos() {
    let server = common::setup_sinir_server().await;
    Mock::given(method("POST"))
        .and(path("/receberManifestoLote"))
        .respond_with(common::json_response(
            200,
            json!({
                "erro": false,
                "mensagem": "Manifesto recebido com sucesso!",
                "totalRecords": 1,
                "objetoResposta": [{
                    "manNumero": "251032769556",
                    "manCodigo": 32853390,
                    "restResponseValido": true,
                    "restResponseMensagem": "Manifesto recebido com sucesso!"
                }]
            }),
        ))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;
    let resp = client
        .receber_mtrs(vec![receber_exemplo("251032769556", true)])
        .await
        .expect("recebimento deve retornar Ok");

    assert!(resp.ok);
    assert_eq!(resp.dados.len(), 1);
    assert!(resp.dados[0].recebido);
    assert_eq!(resp.dados[0].numero, "251032769556");
    assert!(resp.erros.is_empty());

    let reqs = server.received_requests().await.expect("requisições capturadas");
    let req = reqs
        .iter()
        .find(|r| r.url.path() == "/receberManifestoLote")
        .expect("POST de recebimento");
    let body: Value = serde_json::from_slice(&req.body).expect("body JSON");
    assert!(body.is_array(), "corpo deve ser array");

    let item = &body[0];
    assert_eq!(item["manNumero"], "251032769556");
    assert_eq!(item["nomeResponsavelRecebimento"], "Norlu Apoio");
    assert_eq!(item["nomeMotorista"], "WELLINGTON");
    assert_eq!(item["placaVeiculo"], "RLZ4A36");
    assert!(item.get("nomeResponsavel").is_none(), "campo antigo não deve existir");
    assert!(item.get("listaResiduosRecebidos").is_none());

    let res = &item["listaManifestoResiduos"][0];
    assert_eq!(res["resCodigoIbama"], "Classe A");
    assert_eq!(res["marQuantidade"], 5.0);
    assert_eq!(res["marQuantidadeRecebida"], 5.0);
    assert_eq!(res["traCodigo"], 43);
    assert_eq!(res["resCodigoIbamaNovo"], "170107");
    assert_eq!(res["marJustificativa"], "Substituição RCC");
    assert!(res.get("quantidadeRecebida").is_none());
    assert!(res.get("marSequencial").is_none());
}

#[tokio::test]
async fn test_sinir_receber_lote_parcial() {
    let server = common::setup_sinir_server().await;
    Mock::given(method("POST"))
        .and(path("/receberManifestoLote"))
        .respond_with(common::json_response(
            200,
            json!({
                "erro": false,
                "mensagem": "Manifesto recebido com sucesso!",
                "objetoResposta": [
                    {"manNumero": "111", "restResponseValido": true, "restResponseMensagem": "ok"},
                    {"manNumero": "222", "restResponseValido": false,
                     "restResponseMensagem": "Resíduo/Quantidade/Tratamento não encontrado"}
                ]
            }),
        ))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;
    let resp = client
        .receber_mtrs(vec![receber_exemplo("111", false), receber_exemplo("222", false)])
        .await
        .expect("Ok");

    assert_eq!(resp.dados.len(), 1);
    assert_eq!(resp.dados[0].numero, "111");
    assert_eq!(resp.erros.len(), 1);
    assert!(resp.erros[0].detail.contains("não encontrado"));
}

#[tokio::test]
async fn test_sinir_receber_lote_erro_top_level() {
    let server = common::setup_sinir_server().await;
    Mock::given(method("POST"))
        .and(path("/receberManifestoLote"))
        .respond_with(common::json_response(
            200,
            json!({
                "erro": true,
                "mensagem": "Responsável pelo recebimento não encontrado!"
            }),
        ))
        .mount(&server)
        .await;

    let client = common::create_sinir_client(&server).await;
    let resp = client
        .receber_mtrs(vec![receber_exemplo("111", false)])
        .await
        .expect("Ok");

    assert!(resp.dados.is_empty());
    assert_eq!(resp.erros.len(), 1);
    assert!(resp.erros[0].detail.contains("Responsável"));
}

#[tokio::test]
async fn test_sinir_receber_lote_invalido_nao_envia() {
    let server = common::setup_sinir_server().await;
    let client = common::create_sinir_client(&server).await;

    let mut receber = receber_exemplo("111", false);
    receber.motorista = None;
    let resp = client.receber_mtrs(vec![receber]).await.expect("Ok");

    assert!(resp.dados.is_empty());
    assert_eq!(resp.erros.len(), 1);
    assert!(resp.erros[0].detail.contains("nomeMotorista"));

    let reqs = server.received_requests().await.expect("requisições capturadas");
    assert!(
        !reqs.iter().any(|r| r.url.path() == "/receberManifestoLote"),
        "não deve chamar a API quando o item é inválido"
    );
}

#[tokio::test]
async fn test_sinir_periodo_maior_30_dias_rejeitado() {
    let server = common::setup_sinir_server().await;
    let client = common::create_sinir_client(&server).await;

    let fim = NaiveDate::from_ymd_opt(2026, 9, 13).expect("data válida");
    let inicio = fim - Duration::days(31);
    let resp = client
        .consultar_mtrs(&FiltroMtr {
            periodo: Some(Periodo::new(DataMtr(inicio), DataMtr(fim)).expect("período válido")),
            ..Default::default()
        })
        .await;

    let err = resp.expect_err("período > 30 dias deve ser rejeitado localmente");
    assert_eq!(err.status, 422);
    assert!(err.detail.contains("30 dias"));
}

#[tokio::test]
async fn test_sinir_listar_classes_por_residuo_nao_suportado() {
    let server = common::setup_sinir_server().await;
    let client = common::create_sinir_client(&server).await;

    let err = client
        .listar_classes_por_residuo("170107")
        .await
        .expect_err("SINIR não suporta listar_classes_por_residuo");
    assert_eq!(err.status, 501);
}
