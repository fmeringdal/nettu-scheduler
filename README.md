<div align="center">
<img width="400" src="logo.png" alt="logo">
</div>

# Nettu scheduler
[![MIT licensed](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Build status](https://github.com/fmeringdal/nettu-scheduler/actions/workflows/main.yml/badge.svg)](https://github.com/fmeringdal/nettu-scheduler/actions/workflows/main.yml/badge.svg)
[![codecov](https://codecov.io/gh/fmeringdal/nettu-scheduler/branch/master/graph/badge.svg?token=l5z2mzzdHu)](https://codecov.io/gh/fmeringdal/nettu-scheduler)

## Overview

`Nettu scheduler` is a calendar and scheduler server for instrumenting Rust programs to collect
structured, event-based diagnostic information. `tracing` is maintained by the
Tokio project, but does _not_ require the `tokio` runtime to be used.

<div align="center">
<img src="docs/flow.svg" alt="Application flow">
</div>

## Usage

### In Applications

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
tracing = "0.1"
tracing-subscriber = "0.2"
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

### In Libraries

Libraries should only rely on the `tracing` crate and use the provided macros
and types to collect whatever information might be useful to downstream consumers.

```rust
use std::{error::Error, io};
use tracing::{debug, error, info, span, warn, Level};

// the `#[tracing::instrument]` attribute creates and enters a span
// every time the instrumented function is called. The span is named after the
// the function or method. Parameters passed to the function are recorded as fields.
#[tracing::instrument]
pub fn shave(yak: usize) -> Result<(), Box<dyn Error + 'static>> {
    // this creates an event at the DEBUG level with two fields:
    // - `excitement`, with the key "excitement" and the value "yay!"
    // - `message`, with the key "message" and the value "hello! I'm gonna shave a yak."
    //
    // unlike other fields, `message`'s shorthand initialization is just the string itself.
    debug!(excitement = "yay!", "hello! I'm gonna shave a yak.");
    if yak == 3 {
        warn!("could not locate yak!");
        // note that this is intended to demonstrate `tracing`'s features, not idiomatic
        // error handling! in a library or application, you should consider returning
        // a dedicated `YakError`. libraries like snafu or thiserror make this easy.
        return Err(io::Error::new(io::ErrorKind::Other, "shaving yak failed!").into());
    } else {
        debug!("yak shaved successfully");
    }
    Ok(())
}

pub fn shave_all(yaks: usize) -> usize {
    // Constructs a new span named "shaving_yaks" at the TRACE level,
    // and a field whose key is "yaks". This is equivalent to writing:
    //
    // let span = span!(Level::TRACE, "shaving_yaks", yaks = yaks);
    //
    // local variables (`yaks`) can be used as field values
    // without an assignment, similar to struct initializers.
    let span = span!(Level::TRACE, "shaving_yaks", yaks);
    let _enter = span.enter();

    info!("shaving yaks");

    let mut yaks_shaved = 0;
    for yak in 1..=yaks {
        let res = shave(yak);
        debug!(yak, shaved = res.is_ok());

        if let Err(ref error) = res {
            // Like spans, events can also use the field initialization shorthand.
            // In this instance, `yak` is the field being initalized.
            error!(yak, error = error.as_ref(), "failed to shave yak!");
        } else {
            yaks_shaved += 1;
        }
        debug!(yaks_shaved);
    }

    yaks_shaved
}
```

```toml
[dependencies]
tracing = "0.1"
```

Note: Libraries should *NOT* install a collector by using a method that calls
[`set_global_default()`], as this will cause conflicts when executables try to
set the default later.

### In Asynchronous Code

To trace `async fn`s, the preferred method is using the [`#[instrument]`] attribute:

```rust
use tracing::{info, instrument};
use tokio::{io::AsyncWriteExt, net::TcpStream};
use std::io;

#[instrument]
async fn write(stream: &mut TcpStream) -> io::Result<usize> {
    let result = stream.write(b"hello world\n").await;
    info!("wrote to stream; success={:?}", result.is_ok());
    result
}
```

The [`tracing-futures`] crate must be specified as a dependency to enable
`async` support.

Special handling is needed for the general case of code using
[`std::future::Future`][std-future] or blocks with `async`/`await`, as the
following example _will not_ work:

```rust
async {
    let _s = span.enter();
    // ...
}
```

The span guard `_s` will not exit until the future generated by the `async` block is complete.
Since futures and spans can be entered and exited _multiple_ times without them completing,
the span remains entered for as long as the future exists, rather than being entered only when
it is polled, leading to very confusing and incorrect output.
For more details, see [the documentation on closing spans][closing].

This problem can be solved using the [`Future::instrument`] combinator:

```rust
use tracing::Instrument;

let my_future = async {
    // ...
};

my_future
    .instrument(tracing::info_span!("my_future"))
    .await
```

`Future::instrument` attaches a span to the future, ensuring that the span's lifetime
is as long as the future's.

Under the hood, the `#[instrument]` macro performs same the explicit span
attachment that `Future::instrument` does.

[std-future]: https://doc.rust-lang.org/stable/std/future/trait.Future.html
[`tracing-futures`]: https://docs.rs/tracing-futures
[closing]: https://docs.rs/tracing/latest/tracing/span/index.html#closing-spans
[`Future::instrument`]: https://docs.rs/tracing/latest/tracing/trait.Instrument.html#method.instrument
[`#[instrument]`]: https://docs.rs/tracing/0.1.11/tracing/attr.instrument.html

## Supported Rust Versions

Tracing is built against the latest stable release. The minimum supported
version is 1.42. The current Tracing version is not guaranteed to build on Rust
versions earlier than the minimum supported version.

Tracing follows the same compiler support policies as the rest of the Tokio
project. The current stable Rust compiler and the three most recent minor
versions before it will always be supported. For example, if the current stable
compiler version is 1.45, the minimum supported version will not be increased
past 1.42, three minor versions prior. Increasing the minimum supported compiler
version is not considered a semver breaking change as long as doing so complies
with this policy.

## Getting Help

First, see if the answer to your question can be found in the API documentation.
If the answer is not there, there is an active community in
the [Tracing Discord channel][chat]. We would be happy to try to answer your
question. Last, if that doesn't work, try opening an [issue] with the question.

[chat]: https://discord.gg/EeF3cQw
[issue]: https://github.com/tokio-rs/tracing/issues/new

## Contributing

:balloon: Thanks for your help improving the project! We are so happy to have
you! We have a [contributing guide][guide] to help you get involved in the Tracing
project.

[guide]: CONTRIBUTING.md

## Project layout

The [`tracing`] crate contains the primary _instrumentation_ API, used for
instrumenting libraries and applications to emit trace data. The [`tracing-core`]
crate contains the _core_ API primitives on which the rest of `tracing` is
instrumented. Authors of trace subscribers may depend on `tracing-core`, which
guarantees a higher level of stability.

Additionally, this repository contains several compatibility and utility
libraries built on top of `tracing`. Some of these crates are in a pre-release
state, and are less stable than the `tracing` and `tracing-core` crates.

The crates included as part of Tracing are:

* [`tracing-futures`]: Utilities for instrumenting `futures`.
  ([crates.io][fut-crates]|[docs][fut-docs])

* [`tracing-macros`]: Experimental macros for emitting trace events (unstable).

* [`tracing-attributes`]: Procedural macro attributes for automatically
    instrumenting functions. ([crates.io][attr-crates]|[docs][attr-docs])

* [`tracing-log`]: Compatibility with the `log` crate (unstable).

* [`tracing-opentelemetry`]: Provides a layer that connects spans from multiple
  systems into a trace and emits them to [OpenTelemetry]-compatible distributed
  tracing systems for processing and visualization.
  ([crates.io][otel-crates]|[docs][otel-docs])

* [`tracing-serde`]: A compatibility layer for serializing trace data with
    `serde` (unstable).

* [`tracing-subscriber`]: Collector implementations, and utilities for
  implementing and composing `Collector`s.
  ([crates.io][sub-crates]|[docs][sub-docs])

* [`tracing-tower`]: Compatibility with the `tower` ecosystem (unstable).

* [`tracing-appender`]: Utilities for outputting tracing data, including a file appender
   and non-blocking writer. ([crates.io][app-crates]|[docs][app-docs])

* [`tracing-error`]: Provides `SpanTrace`, a type for instrumenting errors with
  tracing spans

* [`tracing-flame`]; Provides a layer for generating flame graphs based on
  tracing span entry / exit events.

* [`tracing-journald`]: Provides a layer for recording events to the
  Linux `journald` service, preserving structured data.

[`tracing`]: tracing
[`tracing-core`]: tracing-core
[`tracing-futures`]: tracing-futures
[`tracing-macros`]: tracing-macros
[`tracing-attributes`]: tracing-attributes
[`tracing-log`]: tracing-log
[`tracing-opentelemetry`]: tracing-opentelemetry
[`tracing-serde`]: tracing-serde
[`tracing-subscriber`]: tracing-subscriber
[`tracing-tower`]: tracing-tower
[`tracing-appender`]: tracing-appender
[`tracing-error`]: tracing-error
[`tracing-flame`]: tracing-flame
[`tracing-journald`]: tracing-journald

[fut-crates]: https://crates.io/crates/tracing-futures
[fut-docs]: https://docs.rs/tracing-futures

[attr-crates]: https://crates.io/crates/tracing-attributes
[attr-docs]: https://docs.rs/tracing-attributes

[sub-crates]: https://crates.io/crates/tracing-subscriber
[sub-docs]: https://docs.rs/tracing-subscriber

[otel-crates]: https://crates.io/crates/tracing-opentelemetry
[otel-docs]: https://docs.rs/tracing-opentelemetry
[OpenTelemetry]: https://opentelemetry.io/

[app-crates]: https://crates.io/crates/tracing-appender
[app-docs]: https://docs.rs/tracing-appender