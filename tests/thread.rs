//! Public-API contract of `thread`: workers, executors, tasks, deadlines and the stop signal.

#[cfg(test)]
mod tests {

    use engine::thread::{Executor, Runtime, Stop, Task, Time, Workers};
    use engine::error::{Code, Failure, Result};
    use core::time::Duration;
    use std::time::Instant;

    #[test]
    fn workers_run_one_runtime_each_and_report_the_first_failure () {

        let fine = Runtime::workers(3, |index| async move { Failure::ensure(index < 3, Code::Internal, "no") });

        assert_eq!(fine.map(|workers| ( workers.len(), workers.is_empty() )).ok(), Some(( 3, false )));
        assert_eq!(Runtime::workers(2, |_index| async { Ok(()) }).and_then(Workers::join).ok(), Some(()));
        assert_eq!(Runtime::workers(2, |index| async move { Failure::ensure(index != 1, Code::Config, "bad") }).and_then(Workers::join).err().map(|failure| failure.code()), Some(Code::Config));
        assert_eq!(Runtime::block_on(async { 41 + 1 }).ok(), Some(42));
        assert!(Runtime::cores() >= 1);

    }

    async fn signaled () -> ( bool, bool, bool, bool, bool ) {

        let stop  = Stop::new();
        let other = stop.clone();

        let before = stop.raised();
        let quiet  = Time::within(Duration::from_millis(50), other.wait()).await.is_none();

        stop.raise();

        let after = other.raised();
        let woke  = Time::within(Duration::from_millis(50), other.wait()).await.is_some();

        ( before, quiet, after, woke, Stop::default().raised() )

    }

    #[test]
    fn a_stop_signal_reaches_every_clone () {

        assert_eq!(Runtime::block_on(signaled()).ok(), Some(( false, true, true, true, false )));

    }

    #[test]
    fn run_hands_back_the_future_s_own_result () {

        assert_eq!(Runtime::run(async { Ok(7) }).ok(), Some(7));
        assert_eq!(Runtime::run(async { Err::<u8, Failure>(Failure::conflict("taken")) }).err().map(|failure| failure.code()), Some(Code::Conflict));

    }

    async fn joined () -> Option<u8> {

        Task::spawn(async { 22 }).join().await.ok()

    }

    #[test]
    fn an_executor_runs_one_future_after_another_on_this_thread () {

        let executor = Executor::new().ok();
        let Some(executor) = executor else { return assert!(executor.is_some()); };

        let first  = executor.block_on(async { 20 });
        let second = executor.block_on(joined());

        assert_eq!(( first, second ), ( 20, Some(22) ));

    }

    async fn tasks () -> Result<( u32, u64, bool, Code )> {

        let local    = Task::spawn(async { 40 + 2 }).join().await?;
        let blocking = Task::blocking(|| ( 1..=10u64 ).product::<u64>()).join().await?;
        let sleeper  = Task::spawn(Time::sleep(Duration::from_secs(30)));

        sleeper.abort();

        let aborted = sleeper.join().await.err().map_or(Code::Internal, |failure| failure.code());
        let done    = Task::spawn(async {});

        Time::sleep(Duration::from_millis(20)).await;

        Ok(( local, blocking, done.is_finished(), aborted ))

    }

    #[test]
    fn tasks_join_abort_and_run_off_the_worker () {

        assert_eq!(Runtime::run(tasks()).ok(), Some(( 42, 3_628_800, true, Code::Cancelled )));

    }

    async fn bounded () -> ( Option<u8>, Option<u8>, bool ) {

        let started = Instant::now();
        let quick   = Time::within(Duration::from_secs(5), async { 1 }).await;
        let slow    = Time::within(Duration::from_millis(30), Time::sleep(Duration::from_secs(30))).await.map(|()| 2);

        Time::sleep(Duration::from_millis(10)).await;

        ( quick, slow, started.elapsed() >= Duration::from_millis(40) && started.elapsed() < Duration::from_secs(10) )

    }

    #[test]
    fn deadlines_cut_slow_work_and_keep_quick_work () {

        assert_eq!(Runtime::block_on(bounded()).ok(), Some(( Some(1), None, true )));
        assert_eq!(Runtime::block_on(Time::within(Duration::from_secs(1), async { 5 })).ok(), Some(Some(5)));

    }

}
