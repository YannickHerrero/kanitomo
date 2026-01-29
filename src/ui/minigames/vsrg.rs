use rand::Rng;

const VSRG_LANES: usize = 4;
const VSRG_DURATION_SECS: f32 = 20.0;
const VSRG_BASE_INTERVAL: f32 = 0.55;
const VSRG_INTERVAL_JITTER: f32 = 0.15;
const VSRG_NOTE_SPEED: f32 = 12.0;
const VSRG_NOTE_LENGTH: u16 = 2;
const VSRG_HIT_WINDOW: f32 = 2.6;
const VSRG_PERFECT_WINDOW: f32 = 0.9;
const VSRG_GREAT_WINDOW: f32 = 1.8;
const VSRG_HIT_ZONE_HEIGHT: f32 = 2.0;
const VSRG_HIT_GRACE: f32 = 0.7;
const VSRG_FEEDBACK_TIME: f32 = 0.6;

#[derive(Debug, Clone)]
pub struct VsrgNote {
    pub lane: usize,
    pub y: f32,
    pub length: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsrgJudgment {
    Perfect,
    Great,
    Ok,
    Miss,
}

#[derive(Debug, Clone, Copy)]
pub struct VsrgFeedback {
    pub judgment: VsrgJudgment,
    pub timer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VsrgLaneFlashKind {
    Hit,
    Miss,
}

#[derive(Debug, Clone, Copy)]
pub struct VsrgLaneFlash {
    pub kind: VsrgLaneFlashKind,
    pub timer: f32,
}

#[derive(Debug)]
pub struct VsrgGame {
    pub score: u32,
    pub combo: u32,
    pub max_combo: u32,
    pub bounds: (u16, u16),
    pub elapsed: f32,
    pub duration: f32,
    pub total_points: u32,
    pub total_judged: u32,
    pub notes: Vec<VsrgNote>,
    pub last_judgment: Option<VsrgFeedback>,
    pub lane_flashes: [Option<VsrgLaneFlash>; VSRG_LANES],
    spawn_timer: f32,
    rng: rand::rngs::ThreadRng,
}

impl VsrgGame {
    pub fn new(bounds: (u16, u16)) -> Self {
        let mut rng = rand::thread_rng();
        let spawn_timer = Self::next_spawn_timer(&mut rng);

        Self {
            score: 0,
            combo: 0,
            max_combo: 0,
            bounds,
            elapsed: 0.0,
            duration: VSRG_DURATION_SECS,
            total_points: 0,
            total_judged: 0,
            notes: Vec::new(),
            last_judgment: None,
            lane_flashes: [None; VSRG_LANES],
            spawn_timer,
            rng,
        }
    }

    pub fn update_bounds(&mut self, bounds: (u16, u16)) {
        self.bounds = bounds;
    }

    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;

        if self.elapsed < self.duration {
            self.spawn_timer -= dt;
            if self.spawn_timer <= 0.0 {
                self.spawn_notes();
                self.spawn_timer = Self::next_spawn_timer(&mut self.rng);
            }
        }

        let speed = self.note_speed();
        for note in &mut self.notes {
            note.y += speed * dt;
        }

        let (_hit_top, hit_bottom) = self.hit_zone_bounds();
        let mut remaining = Vec::with_capacity(self.notes.len());
        let mut misses = 0u32;
        for note in self.notes.drain(..) {
            if note.y > hit_bottom + VSRG_HIT_WINDOW {
                misses += 1;
            } else {
                remaining.push(note);
            }
        }
        self.notes = remaining;
        if misses > 0 {
            self.register_miss(misses, true);
        }

        if let Some(mut feedback) = self.last_judgment {
            feedback.timer = (feedback.timer - dt).max(0.0);
            if feedback.timer <= 0.0 {
                self.last_judgment = None;
            } else {
                self.last_judgment = Some(feedback);
            }
        }

        for flash in &mut self.lane_flashes {
            if let Some(mut lane_flash) = *flash {
                lane_flash.timer = (lane_flash.timer - dt).max(0.0);
                if lane_flash.timer <= 0.0 {
                    *flash = None;
                } else {
                    *flash = Some(lane_flash);
                }
            }
        }
    }

    pub fn hit(&mut self, lane: usize) {
        if lane >= VSRG_LANES {
            return;
        }

        let (hit_top, hit_bottom) = self.hit_zone_bounds();
        let hit_center = (hit_top + hit_bottom) / 2.0;
        let mut best_index = None;
        let mut best_delta = f32::MAX;

        for (index, note) in self.notes.iter().enumerate() {
            if note.lane != lane {
                continue;
            }
            let note_center = note.y + (note.length as f32 - 1.0) / 2.0;
            let delta = (note_center - hit_center).abs();
            if delta < best_delta {
                best_delta = delta;
                best_index = Some(index);
            }
        }

        let Some(index) = best_index else {
            self.register_miss(1, true);
            self.flash_lane(lane, VsrgLaneFlashKind::Miss);
            return;
        };
        let note_center = self.notes[index].y + (self.notes[index].length as f32 - 1.0) / 2.0;
        if note_center < hit_top - VSRG_HIT_GRACE || note_center > hit_bottom + VSRG_HIT_GRACE {
            self.register_miss(1, true);
            self.flash_lane(lane, VsrgLaneFlashKind::Miss);
            return;
        }

        let note = self.notes.remove(index);
        let (points, judgment) = if best_delta <= VSRG_PERFECT_WINDOW {
            (300, VsrgJudgment::Perfect)
        } else if best_delta <= VSRG_GREAT_WINDOW {
            (200, VsrgJudgment::Great)
        } else {
            (100, VsrgJudgment::Ok)
        };

        self.total_points += points;
        self.total_judged += 1;
        self.combo += 1;
        self.max_combo = self.max_combo.max(self.combo);
        self.score = self
            .score
            .saturating_add(points + self.combo * 2 + note.length as u32);
        self.last_judgment = Some(VsrgFeedback {
            judgment,
            timer: VSRG_FEEDBACK_TIME,
        });
        self.flash_lane(lane, VsrgLaneFlashKind::Hit);
    }

    pub fn accuracy(&self) -> f32 {
        if self.total_judged == 0 {
            return 0.0;
        }
        let max_points = self.total_judged as f32 * 300.0;
        (self.total_points as f32 / max_points) * 100.0
    }

    pub fn remaining_time(&self) -> f32 {
        (self.duration - self.elapsed).max(0.0)
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration && self.notes.is_empty()
    }

    pub fn hit_line_y(&self) -> f32 {
        let height = self.bounds.1.saturating_sub(2).max(4) as f32;
        height - 1.0
    }

    pub fn hit_zone_bounds(&self) -> (f32, f32) {
        let bottom = self.hit_line_y();
        let top = (bottom - (VSRG_HIT_ZONE_HEIGHT - 1.0)).max(0.0);
        (top, bottom)
    }

    fn register_miss(&mut self, count: u32, set_feedback: bool) {
        self.combo = 0;
        self.total_judged += count;
        if set_feedback {
            self.last_judgment = Some(VsrgFeedback {
                judgment: VsrgJudgment::Miss,
                timer: VSRG_FEEDBACK_TIME,
            });
        }
    }

    fn flash_lane(&mut self, lane: usize, kind: VsrgLaneFlashKind) {
        if lane >= VSRG_LANES {
            return;
        }
        self.lane_flashes[lane] = Some(VsrgLaneFlash {
            kind,
            timer: VSRG_FEEDBACK_TIME,
        });
    }

    fn spawn_notes(&mut self) {
        let lane = self.rng.gen_range(0..VSRG_LANES);
        self.notes.push(VsrgNote {
            lane,
            y: 0.0,
            length: VSRG_NOTE_LENGTH,
        });

        let chord_chance: f32 = self.rng.gen_range(0.0..1.0);
        if chord_chance > 0.92 {
            let mut other_lane = self.rng.gen_range(0..VSRG_LANES);
            while other_lane == lane {
                other_lane = self.rng.gen_range(0..VSRG_LANES);
            }
            self.notes.push(VsrgNote {
                lane: other_lane,
                y: 0.0,
                length: VSRG_NOTE_LENGTH,
            });
        }
    }

    fn next_spawn_timer(rng: &mut rand::rngs::ThreadRng) -> f32 {
        let jitter = rng.gen_range(-VSRG_INTERVAL_JITTER..VSRG_INTERVAL_JITTER);
        (VSRG_BASE_INTERVAL + jitter).max(0.28)
    }

    fn note_speed(&self) -> f32 {
        let height_scale = (self.bounds.1 as f32 / 24.0).clamp(0.6, 1.1);
        VSRG_NOTE_SPEED * height_scale
    }
}

pub fn vsrg_lane_count() -> usize {
    VSRG_LANES
}
