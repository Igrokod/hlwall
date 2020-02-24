use crate::packet::GoldSrcPacket;
use crate::remote_server::RemoteServer;
use crate::ttl::Ttl;
use std::collections::HashMap;
use std::io::{self, Cursor, Write};
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
            Ttl::new(vec![], cache_duration),
        );

        CachingServer {
            inner,
            cache,
            cache_duration,
        }
    }

    pub fn request(
        &mut self,
        item: &GoldSrcPacket,
        mut return_buf: &mut [u8],
    ) -> io::Result<usize> {
        let cached_item = self
            .cache
            .get(&item)
            .expect("All request kinds had to be prepopulated");

        match cached_item.get() {
            Some(v) => {
                let mut writer = Cursor::new(return_buf);
                writer.write_all(v)?;
                Ok(writer.position() as usize)
            }
            // Cache miss
            None => {
                let bytes_written = self.inner.request(&item, &mut return_buf)?;
                let packet = &return_buf[0..bytes_written];
                self.cache
                    .insert(*item, Ttl::new(packet.to_owned(), self.cache_duration));
                Ok(bytes_written)
            }
        }
    }
}
