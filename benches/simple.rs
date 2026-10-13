use spinlock::Lock;

static COUNTER: Lock<u64> = Lock::new(0);

fn main() {
    divan::main();
}

#[divan::bench(threads = [1, 2, 4, 8, 16], args = [1024, 16384, 65536, 262144])]
fn counter(bencher: divan::Bencher, niter: u32) {
    bencher.bench(|| {
        for _k in 0..niter {
            let mut c = COUNTER.lock();
            *c += 1;
            drop(c);
            busy();
        }
    });
}

fn busy() {
    for k in 0..(1 << 12) {
        std::hint::black_box(k);
    }
}
