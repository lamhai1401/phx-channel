use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};

/// Provides an atomic counter trait that can be shared across threads.
pub trait AtomicCounter: Send + Sync {
    /// Underlying primitive type that is being shared atomically.
    type PrimitiveType;

    /// Atomically increments the counter by one, returning the _previous_ value.
    fn inc(&self) -> Self::PrimitiveType;

    /// Atomically increments the counter by amount, returning the _previous_ value.
    fn add(&self, amount: Self::PrimitiveType) -> Self::PrimitiveType;

    /// Atomically gets the current value of the counter, without modifying the counter.
    fn get(&self) -> Self::PrimitiveType;

    /// Atomically returns the current value of the counter, while resetting to count to zero.
    fn reset(&self) -> Self::PrimitiveType;

    /// Consume the atomic counter and return the primitive type.
    ///
    /// This is safe because passing self by value guarantees that no other threads are concurrently accessing the atomic data.
    fn into_inner(self) -> Self::PrimitiveType;
}

/// Implementation of [`AtomicCounter`](trait.AtomicCounter.html) that uses
/// [`Relaxed`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html#variant.Relaxed)
/// memory ordering.
///
/// See [crate level documentation](index.html) for more details.
///
/// Note that all operations wrap if the counter is incremented beyond usize::max_value().
#[derive(Debug, Default)]
pub struct RelaxedCounter(AtomicUsize);

impl RelaxedCounter {
    /// Creates a new counter with initial_value
    pub fn new(initial_count: usize) -> RelaxedCounter {
        RelaxedCounter(AtomicUsize::new(initial_count))
    }
}

impl AtomicCounter for RelaxedCounter {
    type PrimitiveType = usize;

    fn add(&self, amount: Self::PrimitiveType) -> Self::PrimitiveType {
        self.0.fetch_add(amount, Relaxed)
    }

    fn inc(&self) -> Self::PrimitiveType {
        let num = self.add(1);
        if num > 255 {
            self.reset();
            self.inc()
        } else {
            num
        }

        // return num;
    }

    fn get(&self) -> usize {
        self.0.load(Relaxed)
    }

    fn reset(&self) -> usize {
        self.0.swap(0, Relaxed)
    }

    fn into_inner(self) -> usize {
        self.0.into_inner()
    }
}

/// Implementation of [`AtomicCounter`](trait.AtomicCounter.html) that uses
/// [`Sequentially Consistent`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html#variant.SeqCst)
/// memory ordering.
///
/// See [crate level documentation](index.html) for more details.
///
/// Note that all operations wrap if the counter is incremented beyond usize::max_value().
#[derive(Debug, Default)]
pub struct ConsistentCounter(AtomicUsize);

impl ConsistentCounter {
    /// Creates a new counter with initial_value
    pub fn new(initial_count: usize) -> ConsistentCounter {
        ConsistentCounter(AtomicUsize::new(initial_count))
    }
}

impl AtomicCounter for ConsistentCounter {
    type PrimitiveType = usize;

    fn inc(&self) -> usize {
        self.add(1)
    }

    fn add(&self, amount: usize) -> usize {
        self.0.fetch_add(amount, SeqCst)
    }

    fn get(&self) -> usize {
        self.0.load(SeqCst)
    }

    fn reset(&self) -> usize {
        self.0.swap(0, SeqCst)
    }

    fn into_inner(self) -> usize {
        self.0.into_inner()
    }
}

#[cfg(test)]
mod tests {

    use std::fmt::Debug;
    use std::ops::Deref;
    use std::sync::Arc;
    use std::thread;

    use super::*;

    const NUM_THREADS: usize = 29;
    const NUM_ITERATIONS: usize = 7_000_000;

    fn test_simple_with<Counter>(counter: Counter)
    where
        Counter: AtomicCounter<PrimitiveType = usize>,
    {
        counter.reset();
        assert_eq!(0, counter.add(5));
        assert_eq!(5, counter.add(3));
        assert_eq!(8, counter.inc());
        assert_eq!(9, counter.inc());
        assert_eq!(10, counter.get());
        assert_eq!(10, counter.get());
    }

    #[test]
    fn test_simple_relaxed() {
        test_simple_with(RelaxedCounter::new(0))
    }

    #[test]
    fn test_simple_consistent() {
        test_simple_with(ConsistentCounter::new(0))
    }

    fn test_inc_with<Counter>(counter: Arc<Counter>)
    where
        Counter: AtomicCounter<PrimitiveType = usize> + 'static + Debug,
    {
        let mut join_handles = Vec::new();
        println!(
            "test_inc: Spawning {} threads, each with {} iterations...",
            NUM_THREADS, NUM_ITERATIONS
        );
        for _ in 0..NUM_THREADS {
            let counter_ref = counter.clone();
            join_handles.push(thread::spawn(move || {
                //make sure we're not going though Arc on each iteration
                let counter: &Counter = counter_ref.deref();
                for _ in 0..NUM_ITERATIONS {
                    counter.inc();
                }
            }));
        }
        for handle in join_handles {
            handle.join().unwrap();
        }
        let count = Arc::try_unwrap(counter).unwrap().into_inner();
        println!("test_inc: Got count: {}", count);
        assert_eq!(NUM_THREADS * NUM_ITERATIONS, count);
    }

    #[test]
    fn test_inc_relaxed() {
        test_inc_with(Arc::new(RelaxedCounter::new(0)));
    }

    #[test]
    fn test_inc_consistent() {
        test_inc_with(Arc::new(ConsistentCounter::new(0)));
    }

    fn test_add_with<Counter>(counter: Arc<Counter>)
    where
        Counter: AtomicCounter<PrimitiveType = usize> + 'static + Debug,
    {
        let mut join_handles = Vec::new();
        println!(
            "test_add: Spawning {} threads, each with {} iterations...",
            NUM_THREADS, NUM_ITERATIONS
        );
        let mut expected_count = 0;
        for to_add in 0..NUM_THREADS {
            let counter_ref = counter.clone();
            expected_count += to_add * NUM_ITERATIONS;
            join_handles.push(thread::spawn(move || {
                //make sure we're not going though Arc on each iteration
                let counter: &Counter = counter_ref.deref();
                for _ in 0..NUM_ITERATIONS {
                    counter.add(to_add);
                }
            }));
        }
        for handle in join_handles {
            handle.join().unwrap();
        }
        let count = Arc::try_unwrap(counter).unwrap().into_inner();
        println!(
            "test_add: Expected count: {}, got count: {}",
            expected_count, count
        );
        assert_eq!(expected_count, count);
    }

    #[test]
    fn test_add_relaxed() {
        test_add_with(Arc::new(RelaxedCounter::new(0)));
    }

    #[test]
    fn test_add_consistent() {
        test_add_with(Arc::new(ConsistentCounter::new(0)));
    }

    fn test_reset_with<Counter>(counter: Arc<Counter>)
    where
        Counter: AtomicCounter<PrimitiveType = usize> + 'static + Debug,
    {
        let mut join_handles = Vec::new();
        println!(
            "test_add_reset: Spawning {} threads, each with {} iterations...",
            NUM_THREADS, NUM_ITERATIONS
        );
        let mut expected_count = 0;
        for to_add in 0..NUM_THREADS {
            expected_count += to_add * NUM_ITERATIONS;
        }

        // setup thread that `reset()`s all the time
        let counter_ref = counter.clone();
        let reset_handle = thread::spawn(move || {
            // Usually, you would check for some better termination condition.
            // I don't want to pollute my test with thread synchronization
            // operations outside of AtomicCounter, hence this approach.
            let mut total_count = 0;
            let counter: &Counter = counter_ref.deref();
            while total_count < expected_count {
                total_count += counter.reset();
            }
            // Ok, now we got the total_count but this could just be lucky.
            // Better do some more resets to be sure... ;)
            for _ in 0..NUM_ITERATIONS {
                total_count += counter.reset();
            }
            total_count
        });

        for to_add in 0..NUM_THREADS {
            let counter_ref = counter.clone();

            join_handles.push(thread::spawn(move || {
                //make sure we're not going though Arc on each iteration
                let counter: &Counter = counter_ref.deref();
                for _ in 0..NUM_ITERATIONS {
                    counter.add(to_add);
                }
            }));
        }
        for handle in join_handles {
            handle.join().unwrap();
        }
        let actual_count = reset_handle.join().unwrap();
        println!(
            "test_add_reset: Expected count: {}, got count: {}",
            expected_count, actual_count
        );
        assert_eq!(expected_count, actual_count);
    }

    #[test]
    fn test_reset_consistent() {
        test_reset_with(Arc::new(ConsistentCounter::new(0)));
    }

    #[test]
    fn test_reset_relaxed() {
        test_reset_with(Arc::new(RelaxedCounter::new(0)));
    }
}
