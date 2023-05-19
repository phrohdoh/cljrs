use archery::{ArcK, SharedPointerKind};
use bevy::prelude::*;
use bevy::utils::tracing;
use bevy_tokio_tasks::{TokioTasksPlugin, TokioTasksRuntime};
use crossbeam_channel::{unbounded, Receiver, SendError, Sender};
use std::io::{self, BufRead as _};
use cljrs_reader::value::ValuePtr;

// TODO: runtime values

#[derive(Resource)]
struct CljrsSourceSender(Sender<String>);

#[derive(Resource)]
struct CljrsSourceReceiver(Receiver<String>);

#[derive(Resource)]
struct CljrsValuesSender<P: SharedPointerKind>(Sender<Vec<ValuePtr<P>>>);

#[derive(Resource)]
struct CljrsValuesReceiver<P: SharedPointerKind>(Receiver<Vec<ValuePtr<P>>>);

fn main() {
    println!();
    println!("USAGE: exec this program with stdin hooked up to a FIFO/named pipe, then in another terminal write to that FIFO");
    println!();
    println!();

    let (cljrs_src_tx, cljrs_src_rx) = unbounded();
    let (cljrs_vals_tx, cljrs_vals_rx) = unbounded();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(TokioTasksPlugin::default())
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        //
        .insert_resource(CljrsSourceSender(cljrs_src_tx))
        .insert_resource(CljrsSourceReceiver(cljrs_src_rx))
        .insert_resource(CljrsValuesSender::<ArcK>(cljrs_vals_tx))
        .insert_resource(CljrsValuesReceiver::<ArcK>(cljrs_vals_rx))
        //
        .add_startup_system(setup_cljrs_source_reader_stdin)
        .add_startup_system(setup_cljrs_source_to_cljrs_values)
        .add_startup_system(setup_log_cljrs_values)
        //
        .run();
}

fn setup_cljrs_source_reader_stdin(
    tasks_rt: ResMut<TokioTasksRuntime>,
    send_cljrs_src: ResMut<CljrsSourceSender>,
) {
    let send_cljrs_src = Sender::clone(&send_cljrs_src.0);
    tasks_rt.spawn_background_task(|_ctx| async move {
        let mut str_reader = io::stdin().lock();
        loop {
            let mut buf = String::new();
            if let Err(err) = str_reader.read_line(&mut buf) {
                tracing::error!("error reading line: {}", err);
                continue;
            }
            tracing::error!("non-empty buf from stdin");
            if !buf.is_empty() {
                tracing::error!("sending: {:?}", buf);
                if let Err(SendError(err)) = send_cljrs_src.send(buf) {
                    tracing::error!("error sending: {}", err);
                };
            }
        }
    });
}

fn setup_cljrs_source_to_cljrs_values(
    tasks_rt: ResMut<TokioTasksRuntime>,
    recv_cljrs_src: ResMut<CljrsSourceReceiver>,
    send_cljrs_vals: ResMut<CljrsValuesSender<ArcK>>,
) {
    use cljrs_reader::ReadError;
    let recv_cljrs_src = Receiver::clone(&recv_cljrs_src.0);
    let send_cljrs_vals = Sender::clone(&send_cljrs_vals.0);
    tasks_rt.spawn_background_task(|_ctx| async move {
        loop {
            for src in recv_cljrs_src.iter() {
                // tracing::trace!(src);
                let mut vals = vec![];
                match cljrs_reader::reader::<ArcK>(&src) {
                    Some(mut rdr) => loop {
                        match rdr.try_read_one() {
                            Ok(Some(cljrs_reader::SpanValue { data: value, .. })) => {
                                vals.push(value.to_ptr())
                            }
                            Ok(None) => break,
                            Err(err) => {
                                tracing::debug!("{:?}", err);
                                match err {
                                    ReadError::InvalidInput((begin_idx, end_idx)) => {
                                        tracing::error!(
                                            src = &src[..=end_idx],
                                            err_src = &src[begin_idx..=end_idx],
                                            err_begin_idx = begin_idx,
                                            err_end_idx = end_idx,
                                            "invalid input",
                                        )
                                    }
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
                    if let Err(err) = send_cljrs_vals.send(vals) {
                        tracing::error!("{:?}", err);
                    };
                }
            }
        }
    });
}

fn setup_log_cljrs_values(
    tasks_rt: ResMut<TokioTasksRuntime>,
    recv_cljrs_vals: ResMut<CljrsValuesReceiver<ArcK>>,
) {
    let recv_cljrs_vals = Receiver::clone(&recv_cljrs_vals.0);
    tasks_rt.spawn_background_task(|_ctx| async move {
        loop {
            for vals in recv_cljrs_vals.iter() {
                let vals = vals
                    .into_iter()
                    .map(ValuePtr::try_unwrap)
                    .map(Result::unwrap);
                for val in vals {
                    tracing::error!("val: {:?}", val);
                }
            }
        }
    });
}
