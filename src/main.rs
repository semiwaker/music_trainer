mod play;
use play::*;
use std::sync::{
  atomic::{AtomicBool, Ordering},
  mpsc, Arc, Condvar, Mutex,
};
use std::time;

const F_ORDER: [&'static str; 32] = [
  "E3", "F3", "F_3", "G3", "G_3", "A3", "A_3", "B3", "C4", "C_4", "D4", "D_4", "E4", "F4", "F_4",
  "G4", "G_4", "A4", "A_4", "B4", "C5", "C_5", "D5", "D_5", "E5", "F5", "F_5", "G5", "G_5", "A5",
  "A_5", "B5",
];

const NAMES: [&'static str; 32] = [
  "E3", "F3", "F#3", "G3", "G#3", "A3", "A#3", "B3", "C4", "C#4", "D4", "D#4", "E4", "F4", "F#4",
  "G4", "G#4", "A4", "A#4", "B4", "C5", "C#5", "D5", "D#5", "E5", "F5", "F#5", "G5", "G#5", "A5",
  "A#5", "B5",
];

fn main() {
  let (device, config) = default_device();

  let play_state = Arc::new(Mutex::new(PlayData::new(&config)));
  let (_stream, to_play_send, to_front_recv) =
    make_stream(Arc::clone(&play_state), &device, &config);

  for (i, &name) in NAMES.iter().enumerate() {
    println!("{}", name);
    to_play_send.send(ToPlayMsg::Play(i, 1000)).unwrap();
    to_front_recv.recv().unwrap();
  }
}
