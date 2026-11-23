/// Cliente HTTP de alto nível e gerenciamento de tokens de acesso do openMTR.
///
/// [`http::MtrHttpClient`] concentra retry, autenticação Bearer e limite de
/// concorrência; [`token::TokenManager`] mantém o access token em cache e o
/// refresh token usado na renovação.
pub mod http;

/// Cache e renovação de access tokens JWT dos portais SINIR/MTR.
pub mod token;
