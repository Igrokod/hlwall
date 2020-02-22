use crate::packet::GoldSrcPacket;
use crate::remote_server::RemoteServer;
use std::io;
use std::time::Duration;
use ttl_cache::TtlCache;

pub(crate) struct CachingServer {
    cache: TtlCache<GoldSrcPacket, Vec<u8>>,
    cache_duration: Duration,
    inner: RemoteServer,
}

impl CachingServer {
    pub(crate) fn new(inner: RemoteServer, cache_duration: Duration) -> Self {
        let cache = TtlCache::new(20);
        CachingServer {
            inner,
            cache,
            cache_duration,
        }
    }

    pub(crate) fn request(&mut self, item: &GoldSrcPacket) -> io::Result<Vec<u8>> {
        match self.cache.get(&item) {
            Some(v) => Ok(v.to_owned()),
            // Cache miss
            None => {
                let result = self.inner.request(&item)?;
                self.cache
                    .insert(*item, result.clone(), self.cache_duration);
                Ok(result)
            }
        }
    }
}
