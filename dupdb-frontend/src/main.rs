mod fixedthreadpool;
use fixedthreadpool::FixedThreadPool;
const THREAD_POOL_SIZE: usize = 4;

fn main() {
    println!("Hello, world!");
    let pool = FixedThreadPool::new(THREAD_POOL_SIZE);
}
