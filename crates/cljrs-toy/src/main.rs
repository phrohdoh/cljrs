use archery::ArcK;
use cljrs_reader::{
    value::{ArcValuePtrs, ValuePtr},
    ReadError,
};
use std::sync::mpsc;
use std::thread;
use std::{io, sync::Arc};
use tracing_subscriber::{prelude::*, EnvFilter};

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(EnvFilter::from_env("CLJRS_LOG_LEVEL"))
        .init();
    app();
}

fn app() {
    println!();
    println!("USAGE: exec this program with stdin hooked up to a FIFO/named pipe, then in another terminal write to that FIFO");
    println!();
    println!();

    let (send_str, recv_str) = mpsc::channel();
    let (send_vals, recv_vals) = mpsc::channel();
    //
    // stdin -> recv_str -> recv_vals
    setup_read_str_channels(Arc::new(|| Box::new(io::stdin().lock())), send_str);
    setup_values_reader_channels(recv_str, send_vals);
    //
    loop {
        for vals in recv_vals.iter() {
            let vals = vals
                .into_iter()
                .map(ValuePtr::try_unwrap)
                .map(Result::unwrap);
            for val in vals {
                tracing::error!("val: {:?}", val);
            }
        }
    }
}

fn setup_read_str_channels(
    mut str_reader_fn: Arc<dyn Fn() -> Box<dyn io::BufRead> + Send + Sync>,
    send_str: mpsc::Sender<String>,
) {
    thread::spawn(move || {
        let str_reader_fn = Arc::get_mut(&mut str_reader_fn).unwrap();
        let mut str_reader = str_reader_fn();
        loop {
            let mut buf = String::new();
            if let Err(err) = str_reader.read_line(&mut buf) {
                tracing::error!("error reading line: {}", err);
                continue;
            }
            if !buf.is_empty() {
                tracing::debug!("recv'd from input: {}", buf);
                if let Err(mpsc::SendError(err)) = send_str.send(buf) {
                    tracing::error!("error sending: {}", err);
                };
            }
        }
    });
}

fn setup_values_reader_channels(
    recv_str: mpsc::Receiver<String>,
    send_vals: mpsc::Sender<ArcValuePtrs>,
) {
    use cljrs_reader::{reader, WithSpan};
    thread::spawn(move || loop {
        for src in recv_str.iter() {
            tracing::trace!(src);
            let mut vals = vec![];
            match reader::<ArcK>(&src) {
                Some(mut rdr) => loop {
                    match rdr.try_read_one() {
                        Ok(Some(WithSpan { data: value, .. })) => vals.push(value.to_ptr()),
                        Ok(None) => break,
                        Err(err) => {
                            tracing::trace!("{:?}", err);
                            match err {
                                ReadError::InvalidInput((begin_idx, end_idx)) => tracing::error!(
                                    src = &src[..=end_idx],
                                    err_src = &src[begin_idx..=end_idx],
                                    err_begin_idx = begin_idx,
                                    err_end_idx = end_idx,
                                    "invalid input",
                                ),
                                ReadError::InsufficientInput((begin_idx, end_idx)) => {
                                    tracing::error!(
                                        src = &src[..=end_idx],
                                        err_src = &src[begin_idx..=end_idx],
                                        err_begin_idx = begin_idx,
                                        err_end_idx = end_idx,
                                        "insufficient input",
                                    )
                                }
                                ReadError::UnclosedCollection((begin_idx, end_idx)) => {
                                    tracing::error!(
                                        src = &src[..=end_idx],
                                        err_src = &src[begin_idx..=end_idx],
                                        err_begin_idx = begin_idx,
                                        err_end_idx = end_idx,
                                        "unclosed collection",
                                    )
                                }
                            }
                        }
                    }
                },
                None => tracing::error!("failed to create cljrs reader"),
            }
            if !vals.is_empty() {
                if let Err(err) = send_vals.send(vals) {
                    tracing::error!("{:?}", err);
                };
            }
        }
    });
}

// similar to above (`app`) but without threads/channels
fn app2() {
    println!();
    println!("USAGE: exec this program with input from stdin (e.g. `cargo run <<< \"hi\"`)");
    println!();
    println!();

    use io::BufRead as _;
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut buf = String::new();
    if let Err(err) = stdin.read_line(&mut buf) {
        tracing::error!("error reading line: {}", err);
        return;
    }
    if !buf.is_empty() {
        let mut vals = vec![];
        match cljrs_reader::reader::<ArcK>(&buf) {
            Some(mut rdr) => loop {
                match rdr.try_read_one() {
                    Ok(Some(cljrs_reader::SpanValue { data: value, .. })) => {
                        vals.push(value.to_ptr())
                    }
                    Ok(None) => break,
                    Err(err) => {
                        tracing::trace!("{:?}", err);
                        match err {
                            ReadError::InvalidInput((begin_idx, end_idx)) => tracing::error!(
                                src = &buf[..=end_idx],
                                err_src = &buf[begin_idx..=end_idx],
                                err_begin_idx = begin_idx,
                                err_end_idx = end_idx,
                                "invalid input",
                            ),
                            ReadError::InsufficientInput((begin_idx, end_idx)) => {
                                tracing::error!(
                                    src = &buf[..=end_idx],
                                    err_src = &buf[begin_idx..=end_idx],
                                    err_begin_idx = begin_idx,
                                    err_end_idx = end_idx,
                                    "insufficient input",
                                )
                            }
                            ReadError::UnclosedCollection((begin_idx, end_idx)) => {
                                tracing::error!(
                                    src = &buf[..=end_idx],
                                    err_src = &buf[begin_idx..=end_idx],
                                    err_begin_idx = begin_idx,
                                    err_end_idx = end_idx,
                                    "unclosed collection",
                                )
                            }
                        }
                    }
                }
            },
            None => tracing::error!("failed to create cljrs reader"),
        }
        for val in vals
            .into_iter()
            .map(ValuePtr::try_unwrap)
            .map(Result::unwrap)
            .collect::<Vec<_>>()
        {
            tracing::info!("{}", val);
        }
    }
}
