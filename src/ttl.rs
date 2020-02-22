use std::time::{Duration, Instant};

pub(crate) struct Ttl<V> {
    inner: V,
    expiration_time: Instant,
}

impl<V> Ttl<V> {
    pub fn new(inner: V, duration: Duration) -> Self {
        let expiration_time = Instant::now() + duration;
        Self {
            inner,
            expiration_time,
        }
    }

    #[inline]
    pub fn is_fresh(&self) -> bool {
        self.expiration_time > Instant::now()
    }

    pub fn get(&self) -> Option<&V> {
        match self.is_fresh() {
            true => Some(&self.inner),
            false => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Ttl;
    use std::thread::sleep;
    use std::time::Duration;

    const EXPIRATION_MILLIS: u64 = 50;

    #[test]
    fn test_new() {
        Ttl::new((), Duration::default());
    }

    #[test]
    fn test_freshness() {
        let record = Ttl::new((), Duration::from_millis(EXPIRATION_MILLIS));
        assert!(record.is_fresh());

        sleep(Duration::from_millis(EXPIRATION_MILLIS));
        assert!(!record.is_fresh());
    }

    #[test]
    fn test_get() {
        let record = Ttl::new((), Duration::from_millis(EXPIRATION_MILLIS));
        match record.get() {
            Some(_) => {}
            None => panic!("Record is expired when it should not"),
        }

        sleep(Duration::from_millis(EXPIRATION_MILLIS));
        match record.get() {
            Some(_) => panic!("Record is fresh but it had to expire"),
            None => {}
        }
    }
}
