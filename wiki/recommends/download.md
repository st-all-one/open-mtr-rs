# Salvamento de PDFs Baixados

A biblioteca `open-mtr` baixa os PDFs integralmente em memória (`Vec<u8>` / `Buffer`) e **não salva em disco**. O salvamento é responsabilidade do caller.

Este guia mostra como salvar corretamente em cada linguagem alvo.

---

## Validações embutidas

Antes de retornar os bytes, a biblioteca já valida:

- **Magic bytes** — verifica se o arquivo começa com `%PDF` (ou `<?xml` para XML)
- **Content-Type** — rejeita respostas com `text/html` ou `text/plain`
- **Tamanho mínimo** — rejeita arquivos com menos de 100 bytes
- **Content-Length** — quando o header está presente, confere se os bytes recebidos batem

Se alguma validação falhar, o item aparece em `resposta.erros` com `status: 422` e detalhe da divergência. O lote não é abortado — outros itens podem ter sido baixados com sucesso.

```rust
pub struct DownloadItem {
    pub filename: String,   // ex: "123456.pdf"
    pub buffer: Vec<u8>,    // bytes já validados
    pub formato: String,    // "pdf" | "xml"
}
```

---

## Rust

```rust
use std::fs;
use std::path::Path;

let resposta = client.download_mtrs(vec!["123456".into()]).await?;

for item in &resposta.dados {
    let caminho = Path::new("/tmp/mtrs").join(&item.filename);
    fs::write(&caminho, &item.buffer)?;
    println!("Salvo: {}", caminho.display());
}

// Tratar erros
for erro in &resposta.erros {
    eprintln!("Falha: [{}] {}", erro.status, erro.detail);
}
```

**Para múltiplos arquivos:**

```rust
use std::fs;
use std::path::Path;

let numeros = vec!["A".into(), "B".into(), "C".into()];
let resposta = client.download_mtrs(numeros).await?;
let dir = Path::new("/tmp/mtrs");
fs::create_dir_all(dir)?;

for item in &resposta.dados {
    fs::write(dir.join(&item.filename), &item.buffer)?;
}
```

---

## Node.js / TypeScript (via NAPI)

O buffer vem como **base64**. Use `Buffer.from()` para decodificar antes de salvar.

```typescript
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const resposta = await client.downloadMtrs(['123456']);

for (const item of resposta.dados) {
  const bytes = Buffer.from(item.buffer, 'base64');
  const caminho = join('/tmp/mtrs', item.filename);
  mkdirSync('/tmp/mtrs', { recursive: true });
  writeFileSync(caminho, bytes);
  console.log(`Salvo: ${caminho}`);
}

// Tratar erros
for (const erro of resposta.erros) {
  console.error(`Falha [${erro.status}]: ${erro.detail}`);
}
```

**Com async/await e Promise.all:**

```typescript
const numeros = ['123', '456', '789'];
const resposta = await client.downloadMtrs(numeros);

const saves = resposta.dados.map(async (item) => {
  const bytes = Buffer.from(item.buffer, 'base64');
  const caminho = `/tmp/mtrs/${item.filename}`;
  await fs.promises.writeFile(caminho, bytes);
  return caminho;
});

const salvos = await Promise.all(saves);
console.log('Arquivos salvos:', salvos);
```

---

## Kotlin (via UniFFI)

### Com downloadMtrs

```kotlin
import org.json.JSONArray
import org.json.JSONObject
import java.io.File
import java.util.Base64

val respostaJson = client.downloadMtrs(listOf("123456", "789012"))
val resposta = JSONObject(respostaJson)

val dados = resposta.getJSONArray("dados")
for (i in 0 until dados.length()) {
    val item = dados.getJSONObject(i)
    val filename = item.getString("filename")
    val base64 = item.getString("buffer")
    val bytes = Base64.getDecoder().decode(base64)
    val caminho = File("/tmp/mtrs/$filename")
    caminho.parentFile.mkdirs()
    caminho.writeBytes(bytes)
}

val erros = resposta.getJSONArray("erros")
for (i in 0 until erros.length()) {
    val erro = erros.getJSONObject(i)
    System.err.println("Falha [${erro.getInt("status")}]: ${erro.getString("detail")}")
}
```

---

## Swift (via UniFFI)

### Com downloadMtrs

```swift
import Foundation

let client = try OpenMtrClient(system: "sinir", token: token, baseUrl: baseUrl)

let respostaJson = try client.downloadMtrs(nums: ["123456", "789012"])
let data = respostaJson.data(using: .utf8)!
let resposta = try JSONSerialization.jsonObject(with: data) as! [String: Any]

if let dados = resposta["dados"] as? [[String: Any]] {
    for item in dados {
        let filename = item["filename"] as! String
        let base64 = item["buffer"] as! String
        let bytes = Data(base64Encoded: base64)!
        let url = FileManager.default.temporaryDirectory
            .appendingPathComponent(filename)
        try bytes.write(to: url)
    }
}
```

### Para um único MTR

```swift
import Foundation

let client = try OpenMtrClient(system: "sinir", token: token, baseUrl: baseUrl)

let respostaJson = try client.downloadMtrs(nums: ["123456"])
let data = respostaJson.data(using: .utf8)!
let resposta = try JSONSerialization.jsonObject(with: data) as! [String: Any]

if let dados = resposta["dados"] as? [[String: Any]], let item = dados.first {
    let filename = item["filename"] as! String
    let base64 = item["buffer"] as! String
    let bytes = Data(base64Encoded: base64)!
    let url = FileManager.default.temporaryDirectory.appendingPathComponent(filename)
    try bytes.write(to: url)
}
```

---

## Python

Não há binding oficial para Python ainda. Abaixo, duas alternativas práticas.

### Via subprocess (CLI Rust)

Se o projeto for empacotado como CLI, use `subprocess` para invocá-lo:

```python
import subprocess
import json
import base64
from pathlib import Path

resultado = subprocess.run(
    ["open-mtr", "download", "--nums", "123456", "789012", "--format", "json"],
    capture_output=True, text=True, check=True,
)
resposta = json.loads(resultado.stdout)

for item in resposta["dados"]:
    filename = item["filename"]
    buffer = base64.b64decode(item["buffer"])
    caminho = Path("/tmp/mtrs") / filename
    caminho.parent.mkdir(parents=True, exist_ok=True)
    caminho.write_bytes(buffer)
    print(f"Salvo: {caminho}")

for erro in resposta["erros"]:
    print(f"Falha [{erro['status']}]: {erro['detail']}")
```

### Via requests (chamada direta à API)

Para quem prefere consumir a API do MTR/SINIR diretamente com Python:

```python
import requests
import base64
from pathlib import Path

API_BASE = "https://api.exemplo.com"
TOKEN = "seu-jwt-aqui"

headers = {
    "Authorization": f"Bearer {TOKEN}",
    "Content-Type": "application/json",
}

# Download de MTR
numero = "123456"
resp = requests.post(
    f"{API_BASE}/downloadManifesto/{numero}",
    headers=headers,
    json={},
    timeout=30,
)
resp.raise_for_status()

# Validação manual
content_type = resp.headers.get("Content-Type", "")
if "text/html" in content_type or "text/plain" in content_type:
    raise ValueError(f"Content-Type inesperado: {content_type}")

buffer = resp.content
if not buffer.startswith(b"%PDF"):
    raise ValueError("Arquivo não é um PDF válido (magic bytes %PDF ausentes)")

if len(buffer) < 100:
    raise ValueError(f"Arquivo muito pequeno ({len(buffer)} bytes)")

# Salvar
caminho = Path("/tmp/mtrs") / f"{numero}.pdf"
caminho.parent.mkdir(parents=True, exist_ok=True)
caminho.write_bytes(buffer)
print(f"Salvo: {caminho} ({len(buffer)} bytes)")
```

---

## Boas práticas

1. **Sempre verifique `resposta.erros`** — mesmo com status HTTP 200, itens individuais podem
   ter falhado na validação de integridade do PDF.
2. **Crie o diretório antes de salvar** — use `fs::create_dir_all` (Rust),
   `mkdirSync({ recursive: true })` (Node) ou `FileManager.createDirectory` (Swift).
3. **Use nomes únicos** — o `filename` já inclui o número do MTR/CDF para evitar colisões.
4. **Trate `resposta.ok == false`** — indica que nenhum item foi processado com sucesso.
