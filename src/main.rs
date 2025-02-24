use eframe::egui;
use rand::{rngs::SmallRng, Rng, SeedableRng};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        window_builder: Some(Box::new(|builder| {
            builder.with_inner_size([700.0, 800.0]) // ここでウィンドウサイズを指定
        })),
        ..Default::default()
    };

    eframe::run_native(
        "Minesweepers",
        options,
        Box::new(|_cc| Ok(Box::new(Minesweepers::new(10, 15)))),
    )
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Cell {
    Mine,
    Number(u8),
    Empty,
}

struct Minesweepers {
    grid_size: usize,
    cells: Vec<Vec<Cell>>,
    opened: Vec<Vec<bool>>, // true: open, false: close
    flagged: Vec<Vec<bool>>,
    game_over: bool,
    game_won: bool,
    start_time: Option<Instant>,
    elapsed_time: f32,
}

impl Minesweepers {
    fn new(size: usize, mine_count: usize) -> Self {
        let mut cells = vec![vec![Cell::Empty; size]; size];
        let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
        let mut rng = SmallRng::seed_from_u64(seed);

        // 地雷を配置
        let mut placed = 0;
        while placed < mine_count {
            let x = rng.random_range(0..size);
            let y = rng.random_range(0..size);
            if cells[y][x] == Cell::Empty {
                cells[y][x] = Cell::Mine;
                placed += 1;
            }
        }

        // 地雷数を計算
        for y in 0..size {
            for x in 0..size {
                if cells[y][x] == Cell::Mine {
                    continue;
                }

                let mut count = 0;
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dy == 0 && dx == 0 {
                            continue;
                        }
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        if nx >= 0 && nx < size as isize && ny >= 0 && ny < size as isize {
                            if cells[ny as usize][nx as usize] == Cell::Mine {
                                count += 1;
                            }
                        }
                    }
                }

                if count > 0 {
                    cells[y][x] = Cell::Number(count);
                }
            }
        }
        Self {
            grid_size: size,
            cells,
            opened: vec![vec![false; size]; size],
            flagged: vec![vec![false; size]; size],
            game_over: false,
            game_won: false,
            start_time: None,
            elapsed_time: 0.0,
        }
    }

    fn open_cell_inner(&mut self, x: usize, y:usize) {
        if self.opened[y][x] {
            return;
        }

        self.opened[y][x] = true;

        if self.cells[y][x] == Cell::Mine {
            self.game_over = true;
        }
        if self.cells[y][x] == Cell::Empty {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && nx < self.grid_size as isize && ny >= 0 && ny < self.grid_size as isize {
                        self.open_cell_inner(nx as usize, ny as usize);
                    }
                }
            }
        }

        self.check_win();
    }

    fn open_cell(&mut self, x: usize, y:usize) {
        if self.flagged[y][x] || self.game_over || self.game_won {
            return;
        }
        if self.opened[y][x] && matches!(self.cells[y][x], Cell::Number(_)) {
            let mut count = 0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if nx >= 0 && nx < self.grid_size as isize && ny >= 0 && ny < self.grid_size as isize {
                        if self.flagged[ny as usize][nx as usize] {
                            count += 1;
                        }
                    }
                }
            }
            if self.cells[y][x] == Cell::Number(count) {
                for dy in -1..=1 {
                    for dx in -1..=1 {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = x as isize + dx;
                        let ny = y as isize + dy;
                        if nx >= 0 && nx < self.grid_size as isize && ny >= 0 && ny < self.grid_size as isize {
                            if !self.flagged[ny as usize][nx as usize] && !self.opened[ny as usize][nx as usize] {
                                self.open_cell_inner(nx as usize, ny as usize);
                            }
                        }
                    }
                }
            }
        }
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }
        self.open_cell_inner(x, y);
    }

    fn toggle_flag(&mut self, x: usize, y: usize) {
        if self.game_over || self.game_won || self.opened[y][x] {
            return;
        }
        self.flagged[y][x] = !self.flagged[y][x];
    }

    fn check_win(&mut self) {
        for y in 0..self.grid_size {
            for x in 0..self.grid_size {
                if self.cells[y][x] != Cell::Mine && !self.opened[y][x] {
                    return;
                }
            }
        }
        self.game_won = true;
    }

    fn reset(&mut self) {
        *self = Self::new(self.grid_size, self.grid_size * self.grid_size / 6)
    }

    fn update_timer(&mut self, ctx: &egui::Context) {
        if let Some(start) = self.start_time {
            if !self.game_over && !self.game_won {
                self.elapsed_time = start.elapsed().as_secs_f32();
                ctx.request_repaint();
            }
        }
    }
}

impl eframe::App for Minesweepers {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.update_timer(ctx);

        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "custom_font".to_owned(),
            egui::FontData::from_static(include_bytes!("../font/NotoSansJP-VariableFont_wght.ttf")).into(),
        );
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "custom_font".to_owned());
        fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "custom_font".to_owned());
        ctx.set_fonts(fonts);

        // let cell_size = 30.0;
        // let padding = 20.0;

        // let new_width = self.grid_size as f32 * cell_size + padding;
        // let new_height = self.grid_size as f32 * cell_size + padding + 100.0;
        // frame.set_window_size(egui::Vec2::new(new_width, new_height));


        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Minesweepers");
            ui.horizontal(|ui| {
                ui.label("サイズ：");
                if ui.add(egui::Slider::new(&mut self.grid_size, 5..=20).text("マス"))
                    .changed() {
                        self.reset();
                        ctx.request_repaint();
                    }
            });

            ui.label(format!("⏱ 経過時間: {:.1} 秒", self.elapsed_time));

            if self.game_over {
                ui.label("💥 ゲームオーバー！💥");
            } else if self.game_won {
                ui.label("🎉 クリア！🎉");
            }

            let spacing = ui.style_mut().spacing.item_spacing;
            ui.style_mut().spacing.item_spacing = egui::vec2(1.0, 1.0);

            for y in 0..self.grid_size {
                ui.horizontal(|ui| {
                    for x in 0..self.grid_size {
                        let mut button = egui::Button::new(if self.opened[y][x] || (self.game_over && self.cells[y][x] == Cell::Mine) {
                            match self.cells[y][x] {
                                Cell::Mine => "💣".to_string(),
                                Cell::Number(n) => n.to_string(),
                                Cell::Empty => " ".to_string(),
                            }
                        } else if self.flagged[y][x] {
                            "🚩".to_string()
                        } else {
                            "".to_string()
                        });
                        if self.opened[y][x] {
                            button = button.fill(egui::Color32::GRAY);
                        }

                        let response = ui.add_sized([30.0, 30.0], button);
                        if response.clicked() {
                            self.open_cell(x, y);
                        }
                        if response.secondary_clicked() {
                            self.toggle_flag(x, y);
                        }
                    }
                });
            }

            ui.style_mut().spacing.item_spacing = spacing;

            if ui.button("🔄 リスタート").clicked() {
                self.reset();
            }
        });

        ctx.request_repaint();
    }
}
