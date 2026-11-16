use crate::error::CliError;
use open_mtr_core::openmtr::domain::newtypes::DataMtr;
use open_mtr_core::openmtr::domain::{FiltroCdf, FiltroMtr, Periodo};
use serde_json::Value;

pub fn periodo_from_json(v: &Value) -> Result<Option<Periodo>, CliError> {
    let Some(per) = v.get("periodo") else {
        return Ok(None);
    };
    let di = per
        .get("dataInicial")
        .and_then(Value::as_str)
        .ok_or_else(|| CliError::Usage("periodo.dataInicial ausente".into()))?;
    let df = per
        .get("dataFinal")
        .and_then(Value::as_str)
        .ok_or_else(|| CliError::Usage("periodo.dataFinal ausente".into()))?;
    let ini = DataMtr::parse(di).map_err(CliError::Usage)?;
    let fim = DataMtr::parse(df).map_err(CliError::Usage)?;
    Periodo::new(ini, fim)
        .map(Some)
        .map_err(|e| CliError::Usage(e.to_string()))
}

pub fn filtro_mtr_from_json(v: &Value) -> Result<FiltroMtr, CliError> {
    let mut f = FiltroMtr {
        periodo: periodo_from_json(v)?,
        ..FiltroMtr::default()
    };
    if v.get("numeros").is_some() {
        f.numeros = Some(string_array(v, "numeros")?);
    }
    if let Some(p) = v.get("pagina").and_then(Value::as_u64) {
        f.pagina = Some(u32::try_from(p).map_err(|_| CliError::Usage("pagina inválida".into()))?);
    }
    if let Some(t) = v.get("tamanho").and_then(Value::as_u64) {
        f.tamanho = Some(u32::try_from(t).map_err(|_| CliError::Usage("tamanho inválido".into()))?);
    }
    if let Some(d) = v.get("detalhado").and_then(Value::as_bool) {
        f.detalhado = d;
    }
    Ok(f)
}

pub fn filtro_cdf_from_json(v: &Value) -> Result<FiltroCdf, CliError> {
    let mut f = FiltroCdf {
        periodo: periodo_from_json(v)?,
        ..FiltroCdf::default()
    };
    if v.get("codigos").is_some() {
        f.codigos = Some(string_array(v, "codigos")?);
    }
    Ok(f)
}

/// Lê `v[field]` como array não vazio de strings, sem descartar itens.
pub fn string_array(v: &Value, field: &str) -> Result<Vec<String>, CliError> {
    let items = v
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| CliError::Usage(format!("params.{field} deve ser um array de strings")))?;
    if items.is_empty() {
        return Err(CliError::Usage(format!("params.{field} vazio")));
    }
    items
        .iter()
        .map(|item| {
            item.as_str().map(str::to_string).ok_or_else(|| {
                CliError::Usage(format!("params.{field} deve conter apenas strings"))
            })
        })
        .collect()
}

pub fn parse_items<T: serde::de::DeserializeOwned>(v: &Value) -> Result<Vec<T>, CliError> {
    let array = match v {
        Value::Array(_) => v.clone(),
        Value::Object(map) => {
            let mut arrays = map.values().filter(|x| x.is_array());
            match (arrays.next(), arrays.next()) {
                (Some(arr), None) => arr.clone(),
                _ => {
                    return Err(CliError::Usage(
                        "params deve ser um array JSON (ou objeto com um único array)".into(),
                    ));
                }
            }
        }
        _ => {
            return Err(CliError::Usage(
                "params deve ser um array JSON de itens".into(),
            ));
        }
    };
    serde_json::from_value(array).map_err(|e| CliError::Usage(format!("params inválidos: {e}")))
}

pub fn parse_params(raw: Option<&str>) -> Result<Value, CliError> {
    match raw {
        Some("-") => {
            use std::io::Read;
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| CliError::Usage(e.to_string()))?;
            serde_json::from_str(&buf)
                .map_err(|e| CliError::Usage(format!("params inválidos: {e}")))
        }
        Some(s) => {
            serde_json::from_str(s).map_err(|e| CliError::Usage(format!("params inválidos: {e}")))
        }
        None => Ok(serde_json::json!({})),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_items_accepts_array() {
        let v = serde_json::json!([{ "numero": "1", "justificativa": "x" }]);
        let items: Vec<open_mtr_core::openmtr::domain::CancelarMtr> =
            parse_items(&v).expect("array");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].numero, "1");
    }

    #[test]
    fn parse_items_accepts_single_array_field() {
        let v = serde_json::json!({ "mtrs": [{ "numero": "1", "justificativa": "x" }] });
        let items: Vec<open_mtr_core::openmtr::domain::CancelarMtr> =
            parse_items(&v).expect("objeto");
        assert_eq!(items.len(), 1);
    }

    #[test]
    fn parse_items_rejects_scalar() {
        let v = serde_json::json!({ "numero": "1" });
        let result: Result<Vec<open_mtr_core::openmtr::domain::CancelarMtr>, _> = parse_items(&v);
        assert!(result.is_err());
    }

    #[test]
    fn filtro_mtr_maps_periodo_and_pagination() {
        let v = serde_json::json!({
            "periodo": { "dataInicial": "01-05-2026", "dataFinal": "31-05-2026" },
            "pagina": 2,
            "tamanho": 100,
            "detalhado": true
        });
        let f = filtro_mtr_from_json(&v).expect("filtro");
        assert_eq!(f.pagina, Some(2));
        assert_eq!(f.tamanho, Some(100));
        assert!(f.detalhado);
        assert!(f.periodo.is_some());
    }

    #[test]
    fn periodo_rejects_inverted_dates() {
        let v = serde_json::json!({
            "periodo": { "dataInicial": "31-05-2026", "dataFinal": "01-05-2026" }
        });
        assert!(filtro_mtr_from_json(&v).is_err());
    }

    #[test]
    fn string_array_rejects_non_strings_empty_and_missing() {
        assert!(string_array(&serde_json::json!({"numeros": ["1", 2]}), "numeros").is_err());
        assert!(string_array(&serde_json::json!({"numeros": []}), "numeros").is_err());
        assert!(string_array(&serde_json::json!({}), "numeros").is_err());
        assert!(string_array(&serde_json::json!({"numeros": "1"}), "numeros").is_err());
    }

    #[test]
    fn filtro_cdf_rejects_non_string_codigo() {
        let v = serde_json::json!({"codigos": ["1", 2]});
        assert!(filtro_cdf_from_json(&v).is_err());
    }
}
