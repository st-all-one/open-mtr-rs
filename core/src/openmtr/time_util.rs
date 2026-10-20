use std::sync::Arc;

use chrono::{DateTime, NaiveDate, Utc};
use uuid::Uuid;

/// Fonte de tempo injetável para testes determinísticos.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;

    fn today(&self) -> NaiveDate {
        self.now().date_naive()
    }

    fn now_unix_secs(&self) -> u64 {
        self.now().timestamp().max(0).cast_unsigned()
    }
}

/// Gerador de identificadores injetável para snapshots/golden tests.
pub trait IdGenerator: Send + Sync {
    fn new_id(&self) -> Uuid;
}

/// Relógio do sistema (implementação de produção).
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
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
    now: DateTime<Utc>,
}

impl FixedClock {
    #[must_use]
    pub fn new(now: DateTime<Utc>) -> Self {
        Self { now }
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
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

#[must_use]
pub fn shared_clock(clock: impl Clock + 'static) -> Arc<dyn Clock> {
    Arc::new(clock)
}

#[must_use]
pub fn shared_id_generator(generator: impl IdGenerator + 'static) -> Arc<dyn IdGenerator> {
    Arc::new(generator)
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn utc_now() -> DateTime<Utc> {
    let ms = js_sys::Date::now() as i64;
    DateTime::from_timestamp_millis(ms).unwrap_or_default()
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn utc_now() -> DateTime<Utc> {
    Utc::now()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fixed_clock_is_deterministic() {
        let now = Utc
            .with_ymd_and_hms(2026, 6, 1, 12, 0, 0)
            .single()
            .expect("data válida");
        let clock = FixedClock::new(now);
        assert_eq!(clock.now(), now);
        assert_eq!(clock.today(), NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    }

    #[test]
    fn sequential_ids_are_stable() {
        let generator = SequentialIdGenerator::default();
        assert_eq!(generator.new_id(), Uuid::from_u128(1));
        assert_eq!(generator.new_id(), Uuid::from_u128(2));
    }
}
