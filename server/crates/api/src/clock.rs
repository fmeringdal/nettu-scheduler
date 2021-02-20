use actix_web::rt::time::{delay_until, interval, Instant};
use nettu_scheduler_infra::Context;
use std::sync::Arc;
use std::time::Duration;

pub fn get_start_delay(now_ts: usize, secs_before_min: usize) -> usize {
    let secs_to_next_minute = 60 - (now_ts / 1000) % 60;
    if secs_to_next_minute > secs_before_min {
        secs_to_next_minute + (60 - secs_before_min)
    } else {
        secs_to_next_minute - secs_before_min
    }
}

pub async fn start_clock(ctx: Arc<Context>) {
    actix_web::rt::spawn(async move {
        let now = ctx.sys.get_utc_timestamp();
        let secs_to_next_run = get_start_delay(now as usize, 5);
        let start = Instant::now() + Duration::from_secs(secs_to_next_run as u64);

        delay_until(start).await;
        let mut minutely_interval = interval(Duration::from_secs(1));
        loop {
            minutely_interval.tick().await;
            // perform the action
        }
    });
}
