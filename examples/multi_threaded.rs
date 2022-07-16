use std::io::Write;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use unit_rs::Unit;

fn main() {
    let mut threads = Vec::new();
    let global_visits = Arc::new(AtomicI32::new(0));

    for thread_id in 0..4 {
        let global_visits = global_visits.clone();
        let handle = std::thread::spawn(move || {
            worker(thread_id, global_visits);
        });
        threads.push(handle);
    }

    for handle in threads {
        handle.join().unwrap();
    }
}

fn worker(thread_id: i32, global_visits: Arc<AtomicI32>) {
    let mut unit = Unit::new().unwrap();
    let mut thread_visits = 0;

    unit.set_request_handler(move |req| {
        let headers = &[("Content-Type", "text/plain")];
        let body = "Hello world!\n";
        let mut res = req.create_response(headers, body)?;
        thread_visits += 1;
        global_visits.fetch_add(1, Ordering::Release);

        res.send_buffer(4096, |_req, buf| {
            writeln!(
                buf,
                "Thread {} visits: {} (global visits: {})",
                thread_id,
                thread_visits,
                global_visits.load(Ordering::Acquire),
            )
            .unwrap();
            Ok(())
        })?;

        Ok(())
    });

    unit.run();
}
