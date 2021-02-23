use actix_web::rt::time::{delay_until, interval, Instant};
use nettu_scheduler_infra::NettuContext;
use std::sync::Arc;
use std::time::Duration;

pub fn get_start_delay(now_ts: usize, secs_before_min: usize) -> usize {
    let secs_to_next_minute = 60 - (now_ts / 1000) % 60;
    if secs_to_next_minute > secs_before_min {
        secs_to_next_minute - secs_before_min
    } else {
        secs_to_next_minute + (60 - secs_before_min)
    }
}

pub async fn start_clock(ctx: Arc<NettuContext>) {
    actix_web::rt::spawn(async move {
        let now = ctx.sys.get_utc_timestamp();
        let secs_to_next_run = get_start_delay(now as usize, 3);
        let start = Instant::now() + Duration::from_secs(secs_to_next_run as u64);

        delay_until(start).await;
        let mut minutely_interval = interval(Duration::from_secs(60));
        loop {
            minutely_interval.tick().await;
            // perform the action

            // let client = actix_web::client::Client::new();
            // for (acc, reminders) in account_reminders {
            //     match acc.settings.webhook {
            //         None => continue,
            //         Some(webhook) => {
            //             if let Err(e) = client
            //                 .post(webhook.url)
            //                 .header("nettu-scheduler-webhook-key", webhook.key)
            //                 .send_json(&reminders)
            //                 .await
            //             {
            //                 println!("Error informing client of reminders: {:?}", e);
            //             }
            //         }
            //     }
            // }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_delay_works() {
        assert_eq!(get_start_delay(50 * 1000, 5), 5);
        assert_eq!(get_start_delay(50 * 1000, 10), 60);
        assert_eq!(get_start_delay(50 * 1000, 15), 55);
        assert_eq!(get_start_delay(60 * 1000, 60), 60);
        assert_eq!(get_start_delay(60 * 1000, 10), 50);
        assert_eq!(get_start_delay(59 * 1000, 0), 1);
        assert_eq!(get_start_delay(59 * 1000, 1), 60);
    }
}
