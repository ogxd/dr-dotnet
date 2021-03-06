use profiling_api::*;
use uuid::Uuid;
use std::time::{Instant, Duration};

use crate::report::*;
use crate::profilers::*;

#[derive(Clone)]
pub struct RuntimePause {
    start: Instant,
    end: Instant,
    reason: ffi::COR_PRF_SUSPEND_REASON,
    gc_reason: ffi::COR_PRF_GC_REASON
}

pub struct RuntimePauseProfiler {
    profiler_info: Option<ProfilerInfo>,
    session_id: Uuid,
    gc_pauses: Vec<RuntimePause>,
    last_start: Instant,
    last_suspend_reason: ffi::COR_PRF_SUSPEND_REASON,
    last_gc_reason: ffi::COR_PRF_GC_REASON,
    profiling_start: Instant,
    profiling_end: Instant,
}

impl Default for RuntimePauseProfiler {
    fn default() -> RuntimePauseProfiler {
        RuntimePauseProfiler {
            last_start: Instant::now(),
            profiling_start: Instant::now(),
            profiling_end: Instant::now(),
            ..Default::default()
        }
    }
}

impl Profiler for RuntimePauseProfiler {

    fn get_info() -> ProfilerData {
        return ProfilerData {
            profiler_id: Uuid::parse_str("805A308B-061C-47F3-9B30-F785C3186E85").unwrap(),
            name: "GC Pause Profiler".to_owned(),
            description: "Measures the impact of GC pauses on response time".to_owned(),
            is_released: false,
        }
    }

    fn profiler_info(&self) -> &ProfilerInfo {
        self.profiler_info.as_ref().unwrap()
    }
}

impl RuntimePauseProfiler {

    fn get_durations(&self, interval: Duration) -> Vec<Duration> {
        let mut current = self.profiling_start;
        let mut last_start = self.gc_pauses[0].start;
        let mut i: usize = 0;
        let mut durations = Vec::new();
        'outer: while current < self.profiling_end {
            current = current.checked_add(interval).unwrap();
            let mut duration= Duration::ZERO;
            while last_start < current {
                if self.gc_pauses[i].end < current {
                    duration += self.gc_pauses[i].end - last_start;
                    i += 1;
                    if  i >= self.gc_pauses.len() {
                        break 'outer
                    }
                    last_start = self.gc_pauses[i].start;
                }
                else
                {
                    duration += current - last_start;
                    last_start = current;
                }
            }
            durations.push(duration);
        }

        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        return durations;
    }
}

impl CorProfilerCallback for RuntimePauseProfiler {

    fn runtime_suspend_started(&mut self, suspend_reason: ffi::COR_PRF_SUSPEND_REASON) -> Result<(), ffi::HRESULT> {
        self.last_start = Instant::now();
        self.last_suspend_reason = suspend_reason;
        Ok(())
    }

    fn runtime_resume_started(&mut self) -> Result<(), ffi::HRESULT> {
        let current = Instant::now();
        let gc_pause = RuntimePause { start: self.last_start, end: current, reason: self.last_suspend_reason.clone(), gc_reason: self.last_gc_reason.clone() };
        self.gc_pauses.push(gc_pause);
        Ok(())
    }
}

impl CorProfilerCallback2 for RuntimePauseProfiler {

    fn garbage_collection_started(&mut self, generation_collected: &[ffi::BOOL], reason: ffi::COR_PRF_GC_REASON) -> Result<(), ffi::HRESULT> {
        let mut max_gen = 0;
        let mut gen_index = 0;
        for gen in generation_collected {
            if *gen == 1 {
                max_gen = gen_index;
            }
            gen_index += 1;
        }
        //self.last_start = Instant::now();
        self.last_gc_reason = reason;
        Ok(())
    }

    fn garbage_collection_finished(&mut self) -> Result<(), ffi::HRESULT> {
        //let current = Instant::now();
        //let gc_pause = RuntimePause { start: self.last_start, end: current, reason: self.last_suspend_reason.clone() };
        //self.gc_pauses.push(gc_pause);
        Ok(())
    }
}

impl CorProfilerCallback3 for RuntimePauseProfiler
{
    fn initialize_for_attach(&mut self, profiler_info: ProfilerInfo, client_data: *const std::os::raw::c_void, client_data_length: u32) -> Result<(), ffi::HRESULT>
    {
        self.profiler_info = Some(profiler_info);

        match self.profiler_info().set_event_mask(ffi::COR_PRF_MONITOR::COR_PRF_MONITOR_SUSPENDS) {
            Ok(_) => (),
            Err(hresult) => error!("Error setting event mask: {:x}", hresult)
        }
        
        match init_session(client_data, client_data_length) {
            Ok(uuid) => {
                self.session_id = uuid;
                Ok(())
            },
            Err(err) => Err(err)
        }
    }

    fn profiler_attach_complete(&mut self) -> Result<(), ffi::HRESULT>
    {
        self.profiling_start = Instant::now();
        detach_after_duration::<RuntimePauseProfiler>(&self, 20);
        Ok(())
    }

    fn profiler_detach_succeeded(&mut self) -> Result<(), ffi::HRESULT>
    {
        self.profiling_end = Instant::now();

        let session = Session::get_session(self.session_id, RuntimePauseProfiler::get_info());

        let mut report = session.create_report("summary.md".to_owned());

        report.write_line(format!("# Runtime Pauses Report"));
        report.write_line(format!("## General"));

        let total_time = self.profiling_end - self.profiling_start;
        let mut total_suspended_time = Duration::ZERO;
        for pause in self.gc_pauses.iter() {
            total_suspended_time += pause.end - pause.start;
        }
        let percentage_of_time_suspended = 100f64 * (total_suspended_time.as_secs_f64() / total_time.as_secs_f64());

        report.write_line(format!("- Number of pauses: {}", self.gc_pauses.len()));
        report.write_line(format!("- Percentage of time suspended: {}%", percentage_of_time_suspended));

        let max_time_spent = self.gc_pauses
            .iter()
            .map(|pause| pause.end - pause.start)
            .max()
            .unwrap();

        report.write_line(format!("- Longuest pause: {}ms", max_time_spent.as_millis()));

        report.write_line(format!("## Quantiles"));

        report.write_line(format!("Suspending the runtime will inevitably increase latency of an application. The symptoms depends on the type of application. For instance, for a backend application, that means increased response time."));
        report.write_line(format!("It is however not an easy task to quantify the impact of a suspended runtime, because it depends on how often it occurs and for how long the runtime is paused."));
        report.write_line(format!("One interesting approach is to measure the pause time quantiles accordingly to a time frame."));
        report.new_line();
        report.write_line(format!("**For instance:** Let's say for a backend application we have a latency of about 200ms each for 100 consecutive requests. We can measure how much pause time we have on a 200ms time interval and compute a quantile based of the results."));
        report.write_line(format!("In this example, a 95p for 200ms of 50ms means that for 95 of the requests, less than 50ms was wasted, but more than 50ms was wasted in runtime pause for the remaining 5 requests."));

        report.new_line();
        report.write_line(format!("Interval | 50p | 95p | 99p"));
        report.write_line(format!("-: | -: | -: | -:"));

        let mut write_row = |duration: Duration| {
            let durations = self.get_durations(duration);
            let quantile_99p = durations[(0.99f64 * (durations.len() as f64)).floor() as usize];
            let quantile_95p = durations[(0.95f64 * (durations.len() as f64)).floor() as usize];
            let quantile_50p = durations[(0.50f64 * (durations.len() as f64)).floor() as usize];
            report.write_line(format!("{}ms | {}ms | {}ms | {}ms", duration.as_millis(), quantile_50p.as_millis(), quantile_95p.as_millis(), quantile_99p.as_millis()));
        };

        write_row(Duration::from_millis(1000));
        write_row(Duration::from_millis(750));
        write_row(Duration::from_millis(500));
        write_row(Duration::from_millis(250));
        write_row(Duration::from_millis(100));
        write_row(Duration::from_millis(50));

        report.write_line(format!("## All Pauses"));
        report.new_line();
        report.write_line(format!("Iteration | Reason | Duration (ms)"));
        report.write_line(format!("-: | -: | -:"));

        let mut i = 1;
        for pause in self.gc_pauses.iter() {
            report.write_line(format!("{} | {:?} | {}", i, pause.reason, (pause.end - pause.start).as_millis()));
            i += 1;
        }

        info!("Report written");

        Ok(())
    }
}

impl CorProfilerCallback4 for RuntimePauseProfiler {}
impl CorProfilerCallback5 for RuntimePauseProfiler {}
impl CorProfilerCallback6 for RuntimePauseProfiler {}
impl CorProfilerCallback7 for RuntimePauseProfiler {}
impl CorProfilerCallback8 for RuntimePauseProfiler {}
impl CorProfilerCallback9 for RuntimePauseProfiler {}