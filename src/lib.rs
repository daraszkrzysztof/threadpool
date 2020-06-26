use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

pub struct ThreadPool {
    _handles: Vec<std::thread::JoinHandle<()>>, //() - nothing
    sender: Sender<Box<dyn FnMut() + Send>>,
}

/**
    :: accesses the items of a module, . is used for fields and methods.
    ::<> turbofish, generic type of function allows you to use it; function_name::<>()
    if function is not generic you need to declare type of receiver explicitly

    iter() iterates over the items by reference
    into_iter() iterates over the items, moving them into the new scope
    iter_mut() iterates over the items, giving a mutable reference to each item
    So for x in my_vec { ... } is essentially equivalent to my_vec.into_iter().for_each(|x| ... ) - both move the elements of my_vec into the ... scope.
    If you just need to "look at" the data, use iter, if you need to edit/mutate it, use iter_mut, and if you need to give it a new owner, use into_iter.

    () - primitive type unit, is used when there is no other meaningful value that could be returned

    dyn - uses virtual funcion call table

    move - moves ownership

    Sync -> Mutex
    Send -> Arc
**/
impl ThreadPool {
    pub fn new(num_threads: u8) -> Self {
        let (sender, receiver) = channel::<Box<dyn FnMut() + Send>>();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut _handles = vec![];

        for _ in 0..num_threads {
            let clone = receiver.clone(); //need to clone , can't borrow
            let handle = std::thread::spawn(move || loop {
                let mut work = match clone.lock().unwrap().recv() {
                    Ok(work) => work,
                    Err(_) => break,
                };
                println!("Before work");
                work();
                println!("After work");
            });
            _handles.push(handle);
        }

        Self { _handles, sender }
    }

    pub fn execute<T: FnMut() + Send + 'static>(&self, work: T) {
        //provided work closure can live as long program is executing - static
        self.sender.send(Box::new(work)).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::ThreadPool;
    use std::sync::Arc;

    #[test]
    fn it_works() {
        let pool = ThreadPool::new(10);
        let foo = || std::thread::sleep(std::time::Duration::from_secs(1));
        pool.execute(foo.clone());
        pool.execute(foo.clone());
        pool.execute(foo);

        use std::sync::atomic::{AtomicU32, Ordering};
        let n = AtomicU32::new(0);
        let refN = Arc::new(n);

        let add = move || {
            refN.fetch_add(1, Ordering::SeqCst);
        };

        std::thread::sleep(std::time::Duration::from_secs(2));

        assert_eq!(refN.load(Ordering::SeqCst));
    }
}
