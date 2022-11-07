use criterion::async_executor::FuturesExecutor;
use criterion::*;
use ethers::types::U256;
use ethock_lib::server::{Entry, ServerType};
use std::{thread, time::Duration};
use guild_oracle::{data::OracleRequestArgs, requirements::process_request};

const IP: &str = "127.0.0.1";
const PORT: u16 = 9001;

fn bench_extrinsic(c: &mut Criterion) {
    let mut group = c.benchmark_group("extrinsic");

    thread::spawn(|| Entry::new(ServerType::WS, &format!("{IP}:{PORT}"), "info").serve_silent());
    thread::sleep(Duration::from_millis(100));

    let request = Box::leak(Box::new(OracleRequestArgs::dummy()));
    let minimum_balace = U256::from_dec_str("12345").expect("This should be fine");

    group.bench_function("bench_extrinsic", |b| {
        b.to_async(FuturesExecutor).iter(|| {
            let rt = tokio::runtime::Runtime::new().expect("This should work");

            rt.spawn(process_request(
                request.request_id,
                minimum_balace,
                &request.data,
            ))
        })
    });

    group.finish();
}

criterion_group!(benches, bench_extrinsic);
criterion_main!(benches);
