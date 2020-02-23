use hlwall::{RemoteServer, ListenServer, CachingServer, A2S_INFO_REQUEST};
use std::time::Duration;
use std::thread;
use std::net::UdpSocket;

use criterion::{criterion_group, criterion_main, Criterion, Throughput};

pub fn flood_a2s_info_request(c: &mut Criterion) {
    let remote_server = RemoteServer::connect("127.0.0.1:27015").unwrap();
    let cache_duration = Duration::from_secs(9999999999999);
    let caching_server = CachingServer::new(remote_server, cache_duration);

    thread::spawn(|| {
        ListenServer::bind("127.0.0.1:27016", caching_server).unwrap().serve().unwrap();
    });

    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.connect("127.0.0.1:27016").unwrap();

    let mut group = c.benchmark_group("flood");
    group.throughput(Throughput::Elements(A2S_INFO_REQUEST.len() as u64));
    group.bench_function("throughput", |b| b.iter(|| {
        socket.send(A2S_INFO_REQUEST).unwrap();
        socket.recv(&mut []).unwrap();
    }));
    group.finish();
}

criterion_group!(benches, flood_a2s_info_request);
criterion_main!(benches);
