fn main() {
    let lock = spinlock::Lock::new(1);
    let mut v = lock.lock();
    *v -= 1;
    std::mem::forget(v);
    unsafe {
        lock.force_unlock();
    }
    let v = threaded_add(16, 128);
    println!("v = {v}");
}

#[inline(never)]
fn threaded_add(nthread: u16, niter: u16) -> u32 {
    let lock = spinlock::Lock::new(0);
    std::thread::scope(|scope| {
        for _ in 0..nthread {
            scope.spawn(|| count(&lock, niter));
        }
    });
    lock.into_inner()
}

#[inline(never)]
fn count(lock: &spinlock::Lock<u32>, niter: u16) {
    for _k in 0..niter {
        let mut v = lock.lock();
        *v += 1;
    }
}

#[cfg(test)]
#[test]
fn threaded_add_works() {
    assert_eq!(threaded_add(16, 1024), 16384);
}
