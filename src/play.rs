use super::*;
use cpal;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use hound;
use itertools::Itertools;

pub use cpal::{Device, Stream, StreamConfig};

fn get_config_f32(device: &Device) -> StreamConfig {
  let supported_configs_range = device
    .supported_output_configs()
    .expect("error while querying configs");

  for supported_config_range in supported_configs_range {
    // println!("{:?}", supported_config_range);
    let cfg = supported_config_range.with_max_sample_rate();
    if let cpal::SampleFormat::F32 = cfg.sample_format() {
      return cfg.config();
    }
  }
  panic!()
}

pub fn default_device() -> (Device, StreamConfig) {
  let host = cpal::default_host();

  let device = host
    .default_output_device()
    .expect("no output device available");
  let config = get_config_f32(&device);

  (device, config)
}

fn trans_sample(input: Vec<Vec<f32>>, sample_rate: u32, target_sample_rate: u32) -> Vec<Vec<f32>> {
  (0..input.len())
    .map(|i| get_sample(&input, i, sample_rate, target_sample_rate))
    .collect()
}

fn get_sample(
  samples: &[Vec<f32>],
  pos: usize,
  sample_rate: u32,
  target_sample_rate: u32,
) -> Vec<f32> {
  let p = (pos as f32) * (sample_rate as f32) / (target_sample_rate as f32);
  let i = p.floor() as usize;
  let s = p - p.floor();
  if i + 1 < samples.len() {
    samples[i]
      .iter()
      .zip(&samples[i + 1])
      .map(|(a, b)| a * (1.0 - s) + b * s)
      .collect()
  } else if i < samples.len() {
    samples[i].clone()
  } else {
    (0..samples[0].len())
      .map(|_| cpal::Sample::EQUILIBRIUM)
      .collect()
  }
}

pub fn read_samples(path: &str, config: &StreamConfig) -> (Vec<Vec<f32>>, usize) {
  let reader = hound::WavReader::open(path).unwrap();
  let sample_rate = reader.spec().sample_rate;
  let channels = reader.spec().channels as usize;

  let samples: Vec<Vec<f32>> = reader
    .into_samples::<f32>()
    .into_iter()
    .chunks(channels)
    .into_iter()
    .map(|c| c.into_iter().map(|x| x.unwrap()).collect())
    .collect();
  let target_sample_rate = config.sample_rate.0;

  let samples = trans_sample(samples, sample_rate, target_sample_rate);
  (samples, channels)
}

#[allow(unused)]
pub enum ToPlayMsg {
  Play(usize, usize),
  Stop,
}

pub enum ToFrontMsg {
  Finish,
}

pub enum PlayState {
  Idle,
  Playing {
    id: usize,
    start: time::Instant,
    milisecs: usize,
    pos: usize,
  },
}

pub struct PlayData {
  pub samples: Vec<Vec<Vec<f32>>>,
  pub channels: Vec<usize>,
  pub state: PlayState,
}

impl PlayData {
  pub fn new(config: &StreamConfig) -> Self {
    let mut samples = Vec::new();
    let mut channels = Vec::new();
    for &f in &F_ORDER {
      let (s, c) = read_samples(&format!("data/{f}.wav"), config);
      samples.push(s);
      channels.push(c);
    }
    Self {
      samples,
      channels,
      state: PlayState::Idle,
    }
  }
}

pub fn make_stream(
  play_data: Arc<Mutex<PlayData>>,
  device: &Device,
  config: &StreamConfig,
) -> (Stream, mpsc::Sender<ToPlayMsg>, mpsc::Receiver<ToFrontMsg>) {
  let (to_play_send, to_play_recv) = mpsc::channel();
  let (to_front_send, to_front_recv) = mpsc::channel();
  let stream = device
    .build_output_stream(
      &config,
      move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        let mut pdata = play_data.lock().unwrap();
        if let Ok(msg) = to_play_recv.try_recv() {
          match msg {
            ToPlayMsg::Play(id, milisecs) => {
              pdata.state = PlayState::Playing {
                id,
                start: time::Instant::now(),
                milisecs,
                pos: 0,
              };
            }
            ToPlayMsg::Stop => {
              pdata.state = PlayState::Idle;
            }
          }
        }
        match pdata.state {
          PlayState::Idle => {
            for d in data {
              *d = cpal::Sample::EQUILIBRIUM;
            }
          }
          PlayState::Playing {
            id,
            start,
            milisecs,
            pos,
          } => {
            let samples = &pdata.samples[id];
            let channels = pdata.channels[id];
            let now = time::Instant::now();
            if now.duration_since(start) > time::Duration::from_millis(milisecs as u64) {
              to_front_send.send(ToFrontMsg::Finish).unwrap();
              pdata.state = PlayState::Idle;
              for d in data {
                *d = cpal::Sample::EQUILIBRIUM;
              }
            } else {
              let n = data.len();
              for i in 0..n / channels {
                for j in 0..channels {
                  data[i * channels + j] =
                    samples.get(pos + i).unwrap_or(&samples.last().unwrap())[j];
                }
              }
              pdata.state = PlayState::Playing {
                id,
                start,
                milisecs,
                pos: pos + n / channels,
              };
            }
          }
        }
      },
      move |_| {},
      None,
    )
    .unwrap();
  stream.play().unwrap();
  (stream, to_play_send, to_front_recv)
}
