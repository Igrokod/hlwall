use crate::packet::GoldSrcPacket;
use crate::remote_server::RemoteServer;
use crate::ttl::Ttl;
use std::collections::HashMap;
use std::io;
use std::time::Duration;

pub struct CachingServer {
    cache: HashMap<GoldSrcPacket, Ttl<Vec<u8>>>,
    cache_duration: Duration,
    inner: RemoteServer,
}

impl CachingServer {
    pub fn new(inner: RemoteServer, cache_duration: Duration) -> Self {
        let mut cache = HashMap::new();

        // Insert instant expired items
        // for each request kind
        cache.insert(
            GoldSrcPacket::A2sInfoRequest,
            Ttl::new(vec![], Duration::default()),
        );

        CachingServer {
            inner,
            cache,
            cache_duration,
        }
    }

    pub fn request(&mut self, item: &GoldSrcPacket) -> io::Result<Vec<u8>> {
        let cached_item = self
            .cache
            .get(&item)
            .expect("All request kinds had to be prepopulated");

        match cached_item.get() {
            Some(v) => Ok(v.to_owned()),
            // Cache miss
            None => {
                let result = self.inner.request(&item)?;
                self.cache
                    .insert(*item, Ttl::new(result.clone(), self.cache_duration));
                Ok(result)
            }
        }
    }
}
