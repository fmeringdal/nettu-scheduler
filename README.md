<div align="center">
<img width="400" src="docs/logo.png" alt="logo">
</div>

# Nettu scheduler
[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build status](https://github.com/fmeringdal/nettu-scheduler/actions/workflows/main.yml/badge.svg)](https://github.com/fmeringdal/nettu-scheduler/actions/workflows/main.yml/badge.svg)
[![codecov](https://codecov.io/gh/fmeringdal/nettu-scheduler/branch/master/graph/badge.svg?token=l5z2mzzdHu)](https://codecov.io/gh/fmeringdal/nettu-scheduler)

## Overview

`Nettu scheduler` is a self-hosted calendar and scheduler server that aims to provide the building blocks for building calendar / booking / appointments apps with ease. It has a simple REST API and also a [JavaScript SDK](https://github.com/fmeringdal/nettu-scheduler-js-client) and [Rust SDK](https://crates.io/crates/nettu_scheduler_sdk). 

## Features
- **Authentication**: JWT tokens signed by your server for browser clients and api-keys for server to server communication. 
- **Authorization**: JWT tokens have support for attaching policies which defines what actions the subject can take.
- **Booking**: Create a `Service` and register `User`s on it to make them bookable.
- **Calendar Events**: Supports recurrence rules, flexible querying and reminders.
- **Calendars**: For grouping `Calendar Event`s.
- **Freebusy**: Find out when `User`s are free and when they are busy.
- **Webhooks**: Notifying your server about `Calendar Event` reminders.

<br/>

<div align="center">
<img src="docs/flow.svg" alt="Application flow">
</div>


## Table of contents

  * [Quick start](#quick-start)
  * [Examples](#examples)
     * [Example 1) For every 7 photos display an ad:](#example-1-for-every-7-photos-display-an-ad)
     * [Example 2) For every 4 paragraphs of text include 2 images:](#example-2-for-every-4-paragraphs-of-text-include-2-images)
     * [Example 3) In a group of 8 related links reserve positions 5 and 6 for sponsored links:](#example-3-in-a-group-of-8-related-links-reserve-positions-5-and-6-for-sponsored-links)
     * [Example 4) Display a list of songs including the most successful songs for every 10 songs:](#example-4-display-a-list-of-songs-including-the-most-successful-songs-for-every-10-songs)
     * [Example 5) You can also use larger aliases and the ES6 object shorthand:](#example-5-you-can-also-use-larger-aliases-and-the-es6-object-shorthand)
     * [Example 6) View photos of puppies, kittens and penguins in sequence:](#example-6-view-photos-of-puppies-kittens-and-penguins-in-sequence)
     * [Example 7) Include 1 large photo for every 2 medium size photos followed by 3 small photos:](#example-7-include-1-large-photo-for-every-2-medium-size-photos-followed-by-3-small-photos)
  * [Contributing](#contributing)
  * [License](#license)
  * [Special thanks](#special-thanks)


## Quick start

In order to record trace events, executables have to use a collector
implementation compatible with `tracing`. A collector implements a way of
collecting trace data, such as by logging it to standard output.
[`tracing-subscriber`][tracing-subscriber-docs]'s [`fmt` module][fmt] provides
a collector for logging traces with reasonable defaults. Additionally,
`tracing-subscriber` is able to consume messages emitted by `log`-instrumented
libraries and modules.

To use `tracing-subscriber`, add the following to your `Cargo.toml`:

```toml
[dependencies]
nettu_scheduler_sdk = "0.1"
```

Then create and install a collector, for example using [`init()`]:

```rust
use tracing::info;
use tracing_subscriber;

fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let number_of_yaks = 3;
    // this creates a new event, outside of any spans.
    info!(number_of_yaks, "preparing to shave yaks");

    let number_shaved = yak_shave::shave_all(number_of_yaks);
    info!(
        all_yaks_shaved = number_shaved == number_of_yaks,
        "yak shaving completed."
    );
}
```

Using `init()` calls [`set_global_default()`] so this collector will be used
as the default in all threads for the remainder of the duration of the
program, similar to how loggers work in the `log` crate.

[tracing-subscriber-docs]: https://docs.rs/tracing-subscriber/
[fmt]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html
[`set_global_default`]: https://docs.rs/tracing/latest/tracing/subscriber/fn.set_global_default.html


For more control, a collector can be built in stages and not set globally,
but instead used to locally override the default collector. For example:

```rust
use tracing::{info, Level};
use tracing_subscriber;

fn main() {
    let collector = tracing_subscriber::fmt()
        // filter spans/events with level TRACE or higher.
        .with_max_level(Level::TRACE)
        // build but do not install the subscriber.
        .finish();

    tracing::collector::with_default(collector, || {
        info!("This will be logged to stdout");
    });
    info!("This will _not_ be logged to stdout");
}
```

Any trace events generated outside the context of a collector will not be collected.

This approach allows trace data to be collected by multiple collectors
within different contexts in the program. Note that the override only applies to the
currently executing thread; other threads will not see the change from with_default.

Once a collector has been set, instrumentation points may be added to the
executable using the `tracing` crate's macros.

[`tracing-subscriber`]: https://docs.rs/tracing-subscriber/
[fmt]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html
[`init()`]: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/fn.init.html
[`set_global_default()`]: https://docs.rs/tracing/latest/tracing/subscriber/fn.set_global_default.html

## Examples
* **Authentication**: It supports authentication through JWT tokens signed by your server and api-key for server to server communication. 

* **Calendar**: Grouping of `Calendar Event`s

* **Calendar Events**: Supports recurrence rules and reminders

* **Booking**: Create a `Service` and add bookable `User`s to it with their own preffered availibility `TimePlan`. Then easily query the `Service` for bookingslots

* **Schedule**: An alternative to `Calendar Event` for describing weekly availibility

* **Webhook**: For notifying your server about `CalendarEvent` reminders.


## Contributing

:balloon: Thanks for your help improving the project! We are so happy to have
you! We have a [contributing guide][guide] to help you get involved in the Tracing
project.

[guide]: CONTRIBUTING.md

## License

[MIT](LICENSE) 

## Special thanks

* [Lemmy](https://github.com/LemmyNet/lemmy) for inspiration on how to use cargo workspace to organize a web app. 
* [The author of this blog post](https://www.lpalmieri.com/posts/2020-09-27-zero-to-production-4-are-we-observable-yet/) for an excellent introduction on how to do telemetry in rust. 