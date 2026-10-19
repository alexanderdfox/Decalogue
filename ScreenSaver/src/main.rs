use macroquad::prelude::*;

const COLS: usize = 128;
const ROWS: usize = 72;

struct DecalogueGrid {
    cells: Vec<Vec<u32>>,
    generation: u64,
    sabbath: bool,
    rest_frames: i32,
    running: bool,
}

impl DecalogueGrid {
    fn new() -> Self {
        let mut grid = Self {
            cells: vec![vec![0; COLS]; ROWS],
            generation: 0,
            sabbath: false,
            rest_frames: 0,
            running: true,
        };
        grid.randomize();
        grid
    }

    fn clear(&mut self) {
        self.cells = vec![vec![0; COLS]; ROWS];
        self.generation = 0;
        self.sabbath = false;
    }

    fn randomize(&mut self) {
        for y in 0..ROWS {
            for x in 0..COLS {
                self.cells[y][x] = if rand::gen_range(0.0, 1.0) < 0.2 {
                    rand::gen_range(1, 6)
                } else {
                    0
                };
            }
        }
        self.generation = 0;
        self.sabbath = false;
    }

    fn seed_cross(&mut self) {
        self.clear();
        let cx = COLS / 2;
        let cy = ROWS / 2;
        let shape = [
            (0, 1), (0, 2), (0, 3),
            (1, 0), (1, 4),
            (2, 0), (2, 2), (2, 4),
            (3, 0), (3, 4),
            (4, 1), (4, 2), (4, 3)
        ];
        for &(dx, dy) in &shape {
            let px = (cx as i32 + dx - 2) as usize;
            let py = (cy as i32 + dy - 2) as usize;
            if px < COLS && py < ROWS {
                self.cells[py][px] = 1;
            }
            let px2 = (cx as i32 + dx + 12) as usize;
            let py2 = (cy as i32 + dy - 2) as usize;
            if px2 < COLS && py2 < ROWS {
                self.cells[py2][px2] = 1;
            }
        }
    }

    fn count_neighbors(&self, x: usize, y: usize) -> u32 {
        let mut n = 0;
        for dy in -1..=1 {
            for dx in -1..=1 {
                if dx == 0 && dy == 0 { continue; }
                let nx = (x as i32 + dx + COLS as i32) % COLS as i32;
                let ny = (y as i32 + dy + ROWS as i32) % ROWS as i32;
                if self.cells[ny as usize][nx as usize] > 0 {
                    n += 1;
                }
            }
        }
        n
    }

    fn step(&mut self) {
        self.generation += 1;

        if self.generation % 7 == 0 {
            self.sabbath = true;
            self.rest_frames = 25;
            for row in &mut self.cells {
                for cell in row {
                    if *cell > 0 { *cell += 1; }
                }
            }
            return;
        }

        let mut next = self.cells.clone();

        for y in 0..ROWS {
            for x in 0..COLS {
                let age = self.cells[y][x];
                let n = self.count_neighbors(x, y);

                if age == 0 {
                    if n == 2 || n == 4 {
                        next[y][x] = 1;
                    }
                } else {
                    if n >= 6 {
                        next[y][x] = 0;
                        continue;
                    }

                    let survives = n == 1 || n == 3 || n == 5;
                    if !survives {
                        next[y][x] = 0;
                        continue;
                    }

                    let mut new_age = age + 1;

                    if new_age > 12 && n == 3 {
                        new_age = 4;
                    }

                    if new_age > 24 {
                        next[y][x] = 0;
                    } else {
                        next[y][x] = new_age;
                    }
                }
            }
        }
        self.cells = next;
    }
}

fn get_cell_color(age: u32) -> Color {
    if age <= 2 {
        Color::new(0.0, 0.94, 1.0, 1.0)
    } else if age <= 6 {
        Color::new(0.0, 1.0, 0.4, 1.0)
    } else if age <= 10 {
        Color::new(0.8, 1.0, 0.0, 1.0)
    } else if age <= 15 {
        Color::new(1.0, 0.8, 0.0, 1.0)
    } else if age <= 20 {
        Color::new(1.0, 0.2, 0.4, 1.0)
    } else {
        Color::new(0.58, 0.2, 0.92, 1.0)
    }
}

#[macroquad::main("Decalogue Screensaver")]
async fn main() {
    set_fullscreen(true);

    let mut sim = DecalogueGrid::new();
    let mut last_update = get_time();
    let update_interval = 0.06;

    loop {
        clear_background(Color::new(0.01, 0.01, 0.03, 1.0));

        // Keyboard Controls
        if is_key_pressed(KeyCode::Space) {
            if !sim.sabbath { sim.running = !sim.running; }
        }
        if is_key_pressed(KeyCode::C) {
            sim.clear();
        }
        if is_key_pressed(KeyCode::R) {
            sim.randomize();
        }
        if is_key_pressed(KeyCode::S) {
            sim.seed_cross();
        }
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        // Mouse painting support
        if is_mouse_button_down(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let cell_w = screen_width() / COLS as f32;
            let cell_h = screen_height() / ROWS as f32;
            let gx = (mx / cell_w) as usize;
            let gy = (my / cell_h) as usize;
            if gx < COLS && gy < ROWS {
                sim.cells[gy][gx] = if sim.cells[gy][gx] > 0 { 0 } else { 1 };
            }
        }

        let now = get_time();
        if now - last_update >= update_interval {
            last_update = now;
            if sim.sabbath {
                sim.rest_frames -= 1;
                if sim.rest_frames <= 0 {
                    sim.sabbath = false;
                }
            } else if sim.running {
                sim.step();
            }
        }

        // Render Grid Cells
        let cell_w = screen_width() / COLS as f32;
        let cell_h = screen_height() / ROWS as f32;

        for y in 0..ROWS {
            for x in 0..COLS {
                let age = sim.cells[y][x];
                if age > 0 {
                    draw_rectangle(
                        x as f32 * cell_w + 0.5,
                        y as f32 * cell_h + 0.5,
                        cell_w - 1.0,
                        cell_h - 1.0,
                        get_cell_color(age),
                    );
                }
            }
        }

        // Clean Bottom Status Bar (Legend removed)
        let status_str = if sim.sabbath {
            format!("SABBATH · Gen {} · The Grid Rests", sim.generation)
        } else {
            format!("Generation {} · Spectrum Active (Space: Pause | R: Random | S: Seed | Esc: Exit)", sim.generation)
        };
        draw_rectangle(0.0, screen_height() - 35.0, screen_width(), 35.0, Color::new(0.01, 0.01, 0.03, 0.9));
        draw_text(&status_str, 20.0, screen_height() - 12.0, 16.0, if sim.sabbath { Color::new(0.85, 0.27, 0.94, 1.0) } else { Color::new(0.0, 0.94, 1.0, 1.0) });

        next_frame().await;
    }
}