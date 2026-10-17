//! Tasks spawned, aborted and joined in any order keep their contract: a joined task gives its own value or `Cancelled`.

#![no_main]
#![expect(clippy::disallowed_methods, reason = "the libFuzzer entry macro expands to `expect`")]

use engine::thread::{Executor, Task, Time};
use libfuzzer_sys::fuzz_target;
use engine::error::Code;
use core::time::Duration;

async fn drive ( data: &[u8] ) {

    let tasks = data.iter().take(64).map(|&byte| ( byte, Task::spawn(async move { if byte % 3 == 0 { Time::sleep(Duration::ZERO).await; } byte }) )).collect::<Vec<_>>();

    for ( byte, task ) in tasks {

        if byte % 5 == 0 { task.abort(); }

        let joined = task.join().await;

        assert!(joined.as_ref().map_or_else(|failure| byte % 5 == 0 && failure.code() == Code::Cancelled, |value| *value == byte));

    }

}

fuzz_target!(|data: &[u8]| {

    if let Ok(executor) = Executor::new() { executor.block_on(drive(data)); }

});
