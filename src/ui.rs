use eframe::egui;
use eframe::{self, App};
use egui::RichText;
use egui_extras;
use env_logger;
use rand::prelude::*;

use super::*;

pub fn make_ui(
  stream: Stream,
  to_play_send: mpsc::Sender<ToPlayMsg>,
  to_front_recv: mpsc::Receiver<ToFrontMsg>,
) -> eframe::Result {
  env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 640.0]),
    ..Default::default()
  };
  eframe::run_native(
    "Music Trainer",
    options,
    Box::new(|cc| {
      // This gives us image support:
      egui_extras::install_image_loaders(&cc.egui_ctx);

      Ok(Box::new(MyApp {
        _stream: stream,
        to_play_send,
        to_front_recv,
        state: AppState::PlayAny {
          playing: false,
          id: 0,
        },
        rng: ThreadRng::default(),
      }))
    }),
  )
}

struct MyApp {
  _stream: Stream,
  to_play_send: mpsc::Sender<ToPlayMsg>,
  to_front_recv: mpsc::Receiver<ToFrontMsg>,
  state: AppState,
  rng: ThreadRng,
}

enum AppState {
  PlayAny { playing: bool, id: usize },
  Distinguish2(DistinguishIntervalState),
  Distinguish3(DistinguishIntervalState),
  Distinguish45(DistinguishIntervalState),
  Distinguish6(DistinguishIntervalState),
  Distinguish7(DistinguishIntervalState),
  Distinguish8(DistinguishIntervalState),
  DistinguishAll(DistinguishIntervalState),
}

#[derive(Clone, Default)]
struct DistinguishIntervalState {
  correct: usize,
  wrong: usize,
  id: Option<(usize, usize, usize)>,
  last: Option<(bool, usize, usize, usize)>,
  dir: Direction,
}

#[derive(Clone, PartialEq, Eq)]
enum Direction {
  Up,
  Down,
  Rand,
}

impl Default for Direction {
  fn default() -> Self {
    Self::Up
  }
}

impl eframe::App for MyApp {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::CentralPanel::default().show(ctx, |ui| {
      ui.menu_button(RichText::new("Contents").size(18.0).strong(), |ui| {
        if ui.button(RichText::new("Play any").size(16.0)).clicked() {
          self.state = AppState::PlayAny {
            playing: false,
            id: 0,
          };
          ui.close_menu();
        }
        if ui
          .button(RichText::new("Distinguish 2").size(16.0))
          .clicked()
        {
          self.state = AppState::Distinguish2(DistinguishIntervalState::default());
          ui.close_menu();
        }
        if ui
          .button(RichText::new("Distinguish 3").size(16.0))
          .clicked()
        {
          self.state = AppState::Distinguish3(DistinguishIntervalState::default());
          ui.close_menu();
        }
        if ui
          .button(RichText::new("Distinguish 4 & 5").size(16.0))
          .clicked()
        {
          self.state = AppState::Distinguish45(DistinguishIntervalState::default());
          ui.close_menu();
        }
        if ui
          .button(RichText::new("Distinguish 6").size(16.0))
          .clicked()
        {
          self.state = AppState::Distinguish6(DistinguishIntervalState::default());
          ui.close_menu();
        }
        if ui
          .button(RichText::new("Distinguish 7").size(16.0))
          .clicked()
        {
          self.state = AppState::Distinguish7(DistinguishIntervalState::default());
          ui.close_menu();
        }
        if ui
          .button(RichText::new("Distinguish 8").size(16.0))
          .clicked()
        {
          self.state = AppState::Distinguish8(DistinguishIntervalState::default());
          ui.close_menu();
        }
        if ui
          .button(RichText::new("Distinguish All").size(16.0))
          .clicked()
        {
          self.state = AppState::DistinguishAll(DistinguishIntervalState::default());
          ui.close_menu();
        }
      });

      ui.separator();
      match self.state {
        AppState::PlayAny { playing, id } => self.play_any_ui(ui, playing, id),
        AppState::Distinguish2(ref state) => {
          self.distinguish_interval_ui(ui, state.clone(), &[1, 2], "Distinguish Interval 2")
        }
        AppState::Distinguish3(ref state) => {
          self.distinguish_interval_ui(ui, state.clone(), &[3, 4], "Distinguish Interval 3")
        }
        AppState::Distinguish45(ref state) => {
          self.distinguish_interval_ui(ui, state.clone(), &[5, 6, 7], "Distinguish Interval 4 & 5")
        }
        AppState::Distinguish6(ref state) => {
          self.distinguish_interval_ui(ui, state.clone(), &[8, 9], "Distinguish Interval 6")
        }
        AppState::Distinguish7(ref state) => {
          self.distinguish_interval_ui(ui, state.clone(), &[10, 11], "Distinguish Interval 7")
        }
        AppState::Distinguish8(ref state) => {
          self.distinguish_interval_ui(ui, state.clone(), &[0, 12], "Distinguish Interval 8")
        }
        AppState::DistinguishAll(ref state) => self.distinguish_interval_ui(
          ui,
          state.clone(),
          &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
          "Distinguish All Intervals",
        ),
      }
    });
  }
}

impl MyApp {
  fn play_any_ui(&mut self, ui: &mut egui::Ui, mut playing: bool, mut id: usize) {
    if let Ok(msg) = self.to_front_recv.try_recv() {
      match msg {
        ToFrontMsg::Finish => playing = false,
      }
    }
    ui.horizontal(|ui| {
      if playing {
        ui.heading(
          RichText::new(&format!("Playing: {}", NAMES[id]))
            .strong()
            .size(25.0),
        );
        ui.spinner();
      } else {
        ui.heading(RichText::new("Click to play:").strong().size(25.0));
      }
    });
    egui::Grid::new("play_grid").striped(true).show(ui, |ui| {
      ui.label(RichText::new("3:").size(20.0));
      for _ in 0..4 {
        ui.label("");
      }
      for i in 0..8 {
        if ui.button(RichText::new(NAMES[i]).size(20.0)).clicked() {
          self.to_play_send.send(ToPlayMsg::Play(i, 1000)).unwrap();
          playing = true;
          id = i;
        }
      }
      ui.end_row();
      ui.label(RichText::new("4:").size(20.0));
      for i in 8..20 {
        if ui.button(RichText::new(NAMES[i]).size(20.0)).clicked() {
          self.to_play_send.send(ToPlayMsg::Play(i, 1000)).unwrap();
          playing = true;
          id = i;
        }
      }
      ui.end_row();
      ui.label(RichText::new("5:").size(20.0));
      for i in 20..32 {
        if ui.button(RichText::new(NAMES[i]).size(20.0)).clicked() {
          self.to_play_send.send(ToPlayMsg::Play(i, 1000)).unwrap();
          playing = true;
          id = i;
        }
      }
      ui.end_row();
    });
    self.state = AppState::PlayAny { playing, id };
  }

  fn distinguish_interval_ui(
    &mut self,
    ui: &mut egui::Ui,
    mut state: DistinguishIntervalState,
    choices: &[usize],
    name: &str,
  ) {
    if let Ok(_) = self.to_front_recv.try_recv() {}

    ui.heading(RichText::new(name).size(25.0));

    egui::Grid::new("grid").show(ui, |ui| {
      ui.label(RichText::new("Correct: ").size(20.0));
      ui.label(RichText::new(&format!("{}", state.correct)).size(20.0));
      ui.label(RichText::new("Wrong: ").size(20.0));
      ui.label(RichText::new(&format!("{}", state.wrong)).size(20.0));
      ui.label(RichText::new("Rate: ").size(20.0));
      ui.label(
        RichText::new(&format!(
          "{:.2}",
          if state.correct + state.wrong == 0 {
            0.0
          } else {
            state.correct as f32 / (state.correct + state.wrong) as f32
          }
        ))
        .size(20.0),
      );
      if ui
        .button(RichText::new("reset").strong().size(20.0))
        .clicked()
      {
        state.correct = 0;
        state.wrong = 0;
        state.id = None;
        state.last = None;
      }
      ui.end_row();
    });
    ui.separator();

    ui.horizontal(|ui| {
      ui.radio_value(
        &mut state.dir,
        Direction::Up,
        RichText::new("Up").size(20.0),
      );
      ui.radio_value(
        &mut state.dir,
        Direction::Down,
        RichText::new("Down").size(20.0),
      );
      ui.radio_value(
        &mut state.dir,
        Direction::Rand,
        RichText::new("Rand").size(20.0),
      );
    });
    ui.separator();

    ui.horizontal(|ui| {
      let mut should_play = false;
      if state.id.is_some() {
        if ui.button(RichText::new("Replay").size(20.0)).clicked() {
          should_play = true;
        }
      } else {
        if ui.button(RichText::new("Next").size(20.0)).clicked() {
          should_play = true;
          let interval = choices[self.rng.random_range(0..choices.len())];
          let x = self.rng.random_range(0..(NAMES.len() - interval));
          match state.dir {
            Direction::Up => state.id = Some((interval, x, x + interval)),
            Direction::Down => state.id = Some((interval, x + interval, x)),
            Direction::Rand => {
              if self.rng.random_bool(0.5) {
                state.id = Some((interval, x, x + interval))
              } else {
                state.id = Some((interval, x + interval, x))
              }
            }
          }
        }
      }

      if should_play {
        self
          .to_play_send
          .send(ToPlayMsg::Play(state.id.as_ref().unwrap().1, 500))
          .unwrap();
        self
          .to_play_send
          .send(ToPlayMsg::PlayNext(state.id.as_ref().unwrap().2, 500))
          .unwrap();
      }

      let buttons: Vec<_> = choices
        .iter()
        .map(|x| egui::Button::new(RichText::new(interval_name(*x)).size(20.0)))
        .collect();
      if let Some((x, a, b)) = &state.id {
        let mut select = None;
        for (i, b) in choices.iter().zip(buttons) {
          if ui.add(b).clicked() {
            select = Some(*i);
          }
        }
        if let Some(y) = select {
          if *x == y {
            state.correct += 1;
            state.last = Some((true, *x, *a, *b));
          } else {
            state.wrong += 1;
            state.last = Some((false, *x, *a, *b));
          }
          state.id = None;
        }
      } else {
        for b in buttons {
          ui.add_enabled(false, b);
        }
      }
    });

    if let Some((correct, i, x, y)) = &state.last {
      ui.horizontal(|ui| {
        if *correct {
          ui.label(
            RichText::new("Correct:")
              .color(egui::Color32::GREEN)
              .size(20.0),
          );
        } else {
          ui.label(RichText::new("Wrong:").color(egui::Color32::RED).size(20.0));
        }
        ui.label(RichText::new(interval_name(*i)).strong().size(20.0));
        ui.label(RichText::new(NAMES[*x]).size(20.0));
        ui.label(RichText::new(NAMES[*y]).size(20.0));
      });
    }

    match self.state {
      AppState::Distinguish2(_) => self.state = AppState::Distinguish2(state),
      AppState::Distinguish3(_) => self.state = AppState::Distinguish3(state),
      AppState::Distinguish45(_) => self.state = AppState::Distinguish45(state),
      AppState::Distinguish6(_) => self.state = AppState::Distinguish6(state),
      AppState::Distinguish7(_) => self.state = AppState::Distinguish7(state),
      AppState::Distinguish8(_) => self.state = AppState::Distinguish8(state),
      AppState::DistinguishAll(_) => self.state = AppState::DistinguishAll(state),
      _ => panic!(),
    }
  }
}
