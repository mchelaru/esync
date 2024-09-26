use std::sync::{Condvar, Mutex};

pub struct Semaphore {
    _mutex: Mutex<u32>,
    _cv: Condvar,
}

/// Basic semaphore implementation
///
/// # Examples
///
/// Create a semaphore with initial value 1, that is taken and then released.
/// ```
/// # use esync::semaphore::Semaphore;
/// let sem = Semaphore::new(1);
/// sem.wait();
/// assert_eq!(0, sem.get_current_value());
/// sem.release();
/// ```
impl Semaphore {
    /// Instanties a semaphore with a given initial value
    pub fn new(initial_value: u32) -> Self {
        Self {
            _mutex: Mutex::new(initial_value),
            _cv: Condvar::new(),
        }
    }

    /// Acquires the semaphore or waits in order to do so until another consumer
    /// releases the resource.
    pub fn wait(&self) {
        loop {
            let mut guard = self._mutex.lock().unwrap();
            if *guard > 0 {
                *guard -= 1;
                {
                    return;
                }
            }
            while *guard == 0 {
                guard = self._cv.wait(guard).unwrap();
            }
        }
    }

    /// Releases once the semaphore
    pub fn release(&self) {
        let mut guard = self._mutex.lock().unwrap();
        *guard += 1;
        self._cv.notify_all();
    }

    /// Get the current value of the semaphore.
    ///
    /// The semaphore starts with an initial value, that is decremented until
    /// zero every time a wait() call is completed. On the other hand, the
    /// semaphore value increments every time a release() call is completed.
    pub fn get_current_value(&self) -> u32 {
        *self._mutex.lock().unwrap()
    }
}

#[cfg(test)]
mod test {
    use std::{thread, time::Duration};

    use rand::Rng;

    use super::Semaphore;

    #[test]
    fn wait_and_release() {
        let s = Semaphore::new(1);
        s.wait();
        s.release();
    }

    #[test]
    fn release_while_wait() {
        let sem = Semaphore::new(1);
        sem.wait();
        thread::scope(|s| {
            let waiter = s.spawn(|| {
                sem.wait();
            });
            thread::sleep(Duration::from_millis(100));
            // let's first make sure that the thread is waiting
            assert!(!waiter.is_finished());
            // and now release the lock and make sure the thread died
            sem.release();
            thread::sleep(Duration::from_millis(100));
            assert!(waiter.is_finished());
        });
        // we should be here still holding the semaphore
        assert_eq!(0, sem.get_current_value());
        // so release it
        sem.release();
        assert_eq!(1, sem.get_current_value());
    }

    fn stress(initial_count: u32) {
        let sem = Semaphore::new(initial_count);
        thread::scope(|scope| {
            for _ in 0..initial_count * 4 {
                scope.spawn(|| {
                    let mut rng = rand::thread_rng();
                    // first generate 10k random integers in the 1,21 range
                    (0..10000)
                        .map(|_| (rng.gen::<f64>() * 20.0) as u64 + 1)
                        // transform that into a vec of microsecond Durations
                        .map(|f| Duration::from_micros(f))
                        .collect::<Vec<_>>()
                        // and now use them to hold the semaphore for a given duration
                        .into_iter()
                        .for_each(|d| {
                            sem.wait();
                            thread::sleep(d);
                            sem.release();
                        });
                });
            }
        });
        assert_eq!(initial_count, sem.get_current_value());
    }

    #[test]
    fn stress1() {
        stress(1);
    }

    #[test]
    fn stress2() {
        stress(2);
    }
    #[test]
    fn stress4() {
        stress(4);
    }
    #[test]
    fn stress8() {
        stress(8);
    }
}
