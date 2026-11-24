use std::sync::Arc;

use jiff::{Timestamp, civil::Date};
use uuid::Uuid;

/// Fonte de tempo injetável para testes determinísticos.
///
/// O cliente usa `now`/`today` para obrigações sensíveis ao tempo (datas de
/// manifestos e filtros); injetar um [`FixedClock`] torna esses valores
/// reproduzíveis em golden tests.
pub trait Clock: Send + Sync {
    /// Instante atual (UTC, com resolução de nanossegundos).
    fn now(&self) -> Timestamp;

    /// Data civil (calendário de UTC) derivada de [`Clock::now`]; é o `today()` do contexto.
    fn today(&self) -> Date {
        self.now().to_zoned_datetime(jiff::tz::TimeZone::UTC).date()
    }

    /// Segundos desde o epoch Unix (a partir de `now()`), nunca negativo.
    fn now_unix_secs(&self) -> u64 {
        self.now().as_second().max(0).cast_unsigned()
    }
}

/// Gerador de identificadores injetável para snapshots/golden tests.
pub trait IdGenerator: Send + Sync {
    /// Gera o identificador (UUID) usado no envelope de resposta e nos erros.
    fn new_id(&self) -> Uuid;
}

/// Relógio do sistema (implementação de produção).
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        utc_now()
    }
}

/// Gerador padrão de UUID v7.
#[derive(Debug, Default, Clone, Copy)]
pub struct UuidV7Generator;

impl IdGenerator for UuidV7Generator {
    fn new_id(&self) -> Uuid {
        Uuid::now_v7()
    }
}

/// Relógio fixo, útil em testes e fixtures.
#[derive(Debug, Clone, Copy)]
pub struct FixedClock {
    now: Timestamp,
}

impl FixedClock {
    /// Cria um relógio fixo no instante UTC informado.
    #[must_use]
    pub fn new(now: Timestamp) -> Self {
        Self { now }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> Timestamp {
        self.now
    }
}

/// Gerador sequencial determinístico, útil em testes e fixtures.
#[derive(Debug)]
pub struct SequentialIdGenerator {
    next: std::sync::atomic::AtomicU64,
}

impl Default for SequentialIdGenerator {
    fn default() -> Self {
        Self {
            next: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

impl IdGenerator for SequentialIdGenerator {
    fn new_id(&self) -> Uuid {
        let n = self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Uuid::from_u128(u128::from(n))
    }
}

/// Envolve um relógio em `Arc` para compartilhá-lo entre o cliente e o contexto.
#[must_use]
pub fn shared_clock(clock: impl Clock + 'static) -> Arc<dyn Clock> {
    Arc::new(clock)
}

/// Envolve um gerador de IDs em `Arc` para compartilhá-lo entre o cliente e o contexto.
#[must_use]
pub fn shared_id_generator(generator: impl IdGenerator + 'static) -> Arc<dyn IdGenerator> {
    Arc::new(generator)
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn utc_now() -> Timestamp {
    let ms = js_sys::Date::now() as i64;
    Timestamp::from_millisecond(ms).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn utc_now() -> Timestamp {
    // `Timestamp::now` lê o relógio do sistema; sem features de tz ele não abre a tzdb.
    Timestamp::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jiff::civil::date;

    #[test]
    fn fixed_clock_is_deterministic() {
        let now = Timestamp::from_second(1_780_000_000).expect("instante válido");
        let clock = FixedClock::new(now);
        assert_eq!(clock.now(), now);
        assert_eq!(clock.today(), date(2026, 5, 28));
        assert_eq!(clock.now_unix_secs(), 1_780_000_000);
    }

    #[test]
    fn sequential_ids_are_stable() {
        let generator = SequentialIdGenerator::default();
        assert_eq!(generator.new_id(), Uuid::from_u128(1));
        assert_eq!(generator.new_id(), Uuid::from_u128(2));
    }
}
