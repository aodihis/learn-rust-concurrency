fn main() {
    println!("Concurrency");
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicI32, Ordering};
    use std::sync::{mpsc, Arc, Barrier, Mutex, MutexGuard};
    use std::thread;
    use std::thread::{JoinHandle};
    use std::time::Duration;
    use tokio::runtime;
    use tokio::runtime::Runtime;

    fn calculate() -> i32 {
        let mut counter: i32 = 0;
        for i in 1..10 {
            println!("Counter: {}", i);
            counter += 1;
            thread::sleep(Duration::from_secs(1));
        }
        counter
    }

    #[test]
    fn test_create_thread() {
        thread::spawn(|| {
            for i in 1..10 {
                println!("Counter: {}", i);
                thread::sleep(Duration::from_secs(1));
            }
        });

        println!("finish");
        thread::sleep(Duration::from_secs(10));
    }

    #[test]
    fn test_join_thread() {
        let handle: JoinHandle<i32> = thread::spawn(|| {
            let mut counter: i32 = 0;
            for i in 1..10 {
                println!("Counter: {}", i);
                counter += 1;
                thread::sleep(Duration::from_secs(1));
            }
            return counter;
        });

        let result = handle.join();
        match result {
            Ok(counter) => println!("Total: {}", counter),
            Err(e) => println!("Error: {:?}", e),
        }

        println!("finish");
    }

    #[test]
    fn test_closure() {
        let name = String::from("Yoh");
        let closure = move || {
            thread::sleep(Duration::from_secs(1));
            println!("Hello, {}!", name);
        };
        let handle = thread::spawn(closure);
        handle.join().unwrap();
    }
    #[test]
    fn test_thread_factory() {
        let factory = thread::Builder::new().name("Thread test".to_string());

        let handle = factory.spawn(calculate).expect("failed to create thread");
        let total = handle.join().unwrap();
        println!("Total: {}", total);
    }
    
    #[test]
    fn test_channel() {
        let (tx, rx) = mpsc::channel::<String>();
        let tx2 = tx.clone();
        let handle1 = thread::spawn(move || {
            for _ in 0..10 {
                thread::sleep(Duration::from_secs(1));
                tx.send(String::from("handle 1")).unwrap();
            }
        });

        let handle3 = thread::spawn(move || {
            for _ in 0..5 {
                thread::sleep(Duration::from_secs(3));
                tx2.send(String::from("handle 3")).unwrap();
            }
        });
        let handle2 = thread::spawn(move ||  {
            for val in rx.iter() {
                println!("Message: {}", val);
            }

        });

        handle1.join().expect("Error");
        handle2.join().expect("Error");
        handle3.join().expect("Error");
    }
    
    #[test]
    fn test_atomic() {
        static COUNTER: AtomicI32 = AtomicI32::new(0);
        let mut handles = vec![];

        for _ in 0..10 {
            let handle = thread::spawn(|| {
                for _ in 0..1000000{
                    COUNTER.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }

        println!("Counter : {}", COUNTER.load(Ordering::Relaxed));
    }

    #[test]
    fn test_atomic_reference() {
        let counter: Arc<AtomicI32> = Arc::new(AtomicI32::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..1000000{
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }

        println!("Counter : {}", counter.load(Ordering::Relaxed));
    }

    #[test]
    fn test_mutex() {
        let counter: Arc<Mutex<i32>> = Arc::new(Mutex::new(0));
        let mut handles = vec![];

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..1000000{
                    let mut data: MutexGuard<i32> = counter.lock().unwrap();
                    *data += 1
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }

        println!("Counter : {}", *counter.lock().unwrap());
    }

    #[test]
    fn test_barrier() {
        let barrier = Arc::new(Barrier::new(10));
        let mut handles = vec![];
        for i in 0..10 {
            let barrier = Arc::clone(&barrier);
            let handle = thread::spawn(move || {
                println!("Start {}", i);
                barrier.wait();
                println!("End {}", i);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }

    async fn get_async_data() -> String{
        thread::sleep(Duration::from_secs(1));
        println!("Async data");
        String::from("Async data")
    }

    #[tokio::test]
    async fn test_async() {
        let asn = get_async_data();
        let data = asn.await;
        println!("Data: {:?}", data);
    }

    async fn get_data(wait: u64) -> String{
        println!("a: thread id: {:?}", thread::current().id());
        tokio::time::sleep(Duration::from_secs(wait)).await;
        println!("b: thread id: {:?}", thread::current().id());
        String::from("thread data")
    }

    #[tokio::test]
    async fn test_concurrent() {
        let mut handles = vec![];
        for i in 0..10 {
            let handle = tokio::spawn(get_data(i));
            handles.push(handle);
        }

        for handle in handles {
            let data = handle.await.unwrap();
            println!("data: {:?}", data);
        }
    }

    async fn run_concurrent(runtime: Arc<Runtime>){
        let mut handles = vec![];
        for i in 0..10 {
            let handle = runtime.spawn(get_data(i));
            handles.push(handle);
        }

        for handle in handles {
            let data = handle.await.unwrap();
            println!("data: {:?}", data);
        }
    }
    
    #[test]
    fn test_runtime() {
        let runtime = Arc::new(runtime::Builder::new_multi_thread()
                                   .worker_threads(10)
                                    .enable_time().build().unwrap());

        runtime.block_on(run_concurrent(Arc::clone(&runtime)));
    }

}