#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::openmtr::domain::FiltroMtr;

    use super::super::json_escrita::{escrita_filtro_json, escrita_lista_manifesto_path};
    use super::super::json_residuos::json_residuo_detalhe;

    #[test]
    fn path_sem_tamanho_nao_pagina() {
        let filtro = FiltroMtr::default();
        assert_eq!(escrita_lista_manifesto_path(&filtro), "/listaManifesto");
    }

    #[test]
    fn path_pagina_1_mapeia_offset_0() {
        let filtro = FiltroMtr {
            pagina: Some(1),
            tamanho: Some(50),
            ..FiltroMtr::default()
        };
        assert_eq!(
            escrita_lista_manifesto_path(&filtro),
            "/listaManifesto?page=0&size=50"
        );
    }

    #[test]
    fn path_pagina_3_mapeia_offset_2x_tamanho() {
        let filtro = FiltroMtr {
            pagina: Some(3),
            tamanho: Some(50),
            ..FiltroMtr::default()
        };
        assert_eq!(
            escrita_lista_manifesto_path(&filtro),
            "/listaManifesto?page=100&size=50"
        );
    }

    #[test]
    fn path_sem_pagina_usando_tamanho() {
        let filtro = FiltroMtr {
            tamanho: Some(10),
            ..FiltroMtr::default()
        };
        assert_eq!(
            escrita_lista_manifesto_path(&filtro),
            "/listaManifesto?page=0&size=10"
        );
    }

    #[test]
    fn residuo_detalhe_captura_mar_observacao() {
        let v = json!({
            "residuo": {
                "resCodigoIbama": "170201",
                "resDescricao": "Madeira"
            },
            "marQuantidade": 5.0,
            "marObservacao": "CX 387"
        });
        let r = json_residuo_detalhe(&v);
        assert_eq!(r.observacao.as_deref(), Some("CX 387"));
    }

    #[test]
    fn residuo_detalhe_observacao_em_branco_vira_none() {
        let v = json!({ "marObservacao": "   " });
        assert!(json_residuo_detalhe(&v).observacao.is_none());
    }

    #[test]
    fn body_nao_contem_paginacao() {
        let filtro = FiltroMtr {
            pagina: Some(2),
            tamanho: Some(50),
            ..FiltroMtr::default()
        };
        let body = escrita_filtro_json(
            &filtro,
            chrono::NaiveDate::from_ymd_opt(2026, 6, 1).expect("data de teste"),
        );
        assert!(body.get("pagina").is_none(), "pagina não deve ir no body");
        assert!(
            body.get("tamanhoPagina").is_none(),
            "tamanhoPagina não deve ir no body"
        );
        assert_eq!(body.get("tipoParceiro").and_then(Value::as_i64), Some(8));
    }
}
