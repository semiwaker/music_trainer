mod play;
use play::*;
mod ui;
use ui::*;
mod data;
use data::*;

use std::sync::{
  atomic::{AtomicBool, Ordering},
  mpsc, Arc, Condvar, Mutex,
};
use std::time;

fn main() {
  let (device, config) = default_device();

  let play_data = PlayData::new(&config);
  let (stream, to_play_send, to_front_recv) = make_stream(play_data, &device, &config);

  make_ui(stream, to_play_send, to_front_recv).unwrap();
}
