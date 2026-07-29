use criterion::{black_box, criterion_group, criterion_main, Criterion};

// We need to reference the types. Since RawResponse is pub(crate) in the http module,
// and benchmarks are separate crates, we recreate minimal types for benchmarking.
struct BenchRawResponse {
    bytes: Vec<u8>,
    content_type: Option<String>,
    content_length: Option<u64>,
}

impl BenchRawResponse {
    fn validate(&self, expected: &str) -> Result<(), String> {
        if let Some(ref ct) = self.content_type {
            let ct_lower = ct.to_lowercase();
            if ct_lower.contains("text/html") || ct_lower.contains("text/plain") {
                return Err(format!("Content-Type inesperado: {ct} (esperado {expected})"));
            }
            if expected == "xml" && !ct_lower.contains("xml") && !ct_lower.contains("octet-stream") {
                return Err(format!("Content-Type inesperado: {ct} (esperado application/xml)"));
            }
        }

        let magic_ok = match expected {
            "pdf" => self.bytes.starts_with(b"%PDF"),
            "xml" => self.bytes.starts_with(b"<?xml") || self.bytes.starts_with(b"<"),
            _ => true,
        };
        if !magic_ok {
            return Err(format!("Arquivo inválido: magic bytes não correspondem ao formato {expected}"));
        }

        if let Some(cl) = self.content_length {
            if cl != self.bytes.len() as u64 {
                return Err(format!("Content-Length ({cl}) difere dos bytes recebidos ({})", self.bytes.len()));
            }
        }

        Ok(())
    }
}

fn bench_validate_pdf_10kb(c: &mut Criterion) {
    let mut pdf = Vec::with_capacity(10 * 1024);
    pdf.extend_from_slice(b"%PDF-1.4\n");
    pdf.extend(std::iter::repeat_n(b'a', 10 * 1024 - 8));
    pdf.extend_from_slice(b"\n%%EOF");

    let resp = BenchRawResponse {
        bytes: black_box(pdf),
        content_type: Some("application/pdf".to_string()),
        content_length: Some(10 * 1024),
    };

    c.bench_function("validate_pdf_10kb", |b| {
        b.iter(|| resp.validate(black_box("pdf")).ok());
    });
}

fn bench_validate_pdf_sem_content_type(c: &mut Criterion) {
    let pdf = vec![b'%', b'P', b'D', b'F'];
    let resp = BenchRawResponse {
        bytes: black_box(pdf),
        content_type: None,
        content_length: None,
    };

    c.bench_function("validate_pdf_sem_content_type", |b| {
        b.iter(|| resp.validate(black_box("pdf")).ok());
    });
}

fn bench_validate_pdf_magic_invalido(c: &mut Criterion) {
    let dados = b"<html>error</html>".to_vec();
    let resp = BenchRawResponse {
        bytes: black_box(dados),
        content_type: Some("text/html".to_string()),
        content_length: None,
    };

    c.bench_function("validate_pdf_rejeita_html", |b| {
        b.iter(|| resp.validate(black_box("pdf")));
    });
}

fn bench_validate_pdf_100kb(c: &mut Criterion) {
    let mut pdf = Vec::with_capacity(100 * 1024);
    pdf.extend_from_slice(b"%PDF-1.4\n");
    pdf.extend(std::iter::repeat_n(b'a', 100 * 1024 - 8));
    pdf.extend_from_slice(b"\n%%EOF");

    let resp = BenchRawResponse {
        bytes: black_box(pdf),
        content_type: Some("application/pdf".to_string()),
        content_length: Some(100 * 1024),
    };

    c.bench_function("validate_pdf_100kb", |b| {
        b.iter(|| resp.validate(black_box("pdf")).ok());
    });
}

criterion_group!(benches,
    bench_validate_pdf_10kb,
    bench_validate_pdf_sem_content_type,
    bench_validate_pdf_magic_invalido,
    bench_validate_pdf_100kb,
);
criterion_main!(benches);
