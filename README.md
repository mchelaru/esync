# Extended synchronization primitives

This package contains some primitives that I am using across my projects.

## Semaphore

Example:
```
let sem = Semaphore::new(10);
sem.wait();
// do important things here
sem.release();
```
