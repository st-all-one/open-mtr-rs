//! Download de PDFs e XMLs de manifestos e CDF nos sistemas SINIR, SIGOR e
//! MTR-PE. Os adaptadores baixam os bytes diretamente da API; não há URL nem
//! caminho de arquivo nestes tipos.

use serde::{Deserialize, Serialize};

use super::tipo::FormatoDownload;

/// Arquivo baixado de um manifesto ou CDF.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadItem {
    /// Nome sugerido do arquivo, montado com o número/código e a extensão do
    /// formato (por exemplo `<numero>.pdf` ou `<codigo>.xml`).
    pub filename: String,
    /// Conteúdo bruto do arquivo, validado pelo adaptador antes de ser devolvido.
    pub buffer: Vec<u8>,
    /// Extensão do conteúdo em minúsculas (`"pdf"` ou `"xml"`) — não é o enum
    /// [`FormatoDownload`], que aparece apenas na solicitação.
    pub formato: String,
}

/// Pedido de download do arquivo de um CDF (`download_cdfs`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadCdf {
    /// Código do certificado no sistema de origem, usado na URL de download
    /// (`/downloadCertificado` no SINIR, `/buscaPdfCdf` no MTR-PE).
    pub codigo: String,
    /// Formato desejado; o SINIR honra PDF e XML, enquanto SIGOR e MTR-PE só
    /// entregam PDF.
    pub formato: FormatoDownload,
}
