use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::timer;
use std::time::Instant;
use rand::Rng;

const CELL_SIZE: f32 = 30.0;
const PACMAN_SIZE: f32 = 25.0;
const DOT_SIZE: f32 = 6.0;
const GHOST_SIZE: f32 = 25.0;
const MOVEMENT_SPEED: f32 = 3.0;
const THIN_WALL_SIZE: f32 = 30.0;

// Define the ghost cage boundaries
const GHOST_CAGE: (f32, f32, f32, f32) = (CELL_SIZE * 5.0, CELL_SIZE * 5.0, CELL_SIZE * 10.0, CELL_SIZE * 10.0); // (x1, y1, x2, y2)

const MAP_STR: [&'static str; 31] = [
    "WWWWWWWWWWWWWWWWWWWWWWWWWWWW",
    "W............##............W",
    "W.WW.W.WWWWWWW.W.W.W.WWW.W.W",
    "W.W..W.W.....W.W.W.W.W.W.W",
    "W.W..W.W.WWW.W.W.W.W.W.W.W",
    "W....W...W.W...W...W...W.W",
    "W.WW.WWW.W.W.WWWWW.W.W.W.W",
    "W.W..W...W.W.....W.W.W.W.W",
    "W.W.WW.WWW.WWWWW.W.W.W.W.W",
    "W.W....W...W.....W.W.W.W.W",
    "W.WWWW.W.W.W.WWW.W.W.W.W.W",
    "W......W.W.....W...W.W.W.W",
    "WWWWWWWWWWWWWWWWWWWWWWWWWWWW",
    "W............##............W",
    "W.WW.W.WWWWWWW.W.W.W.WWW.W.W",
    "W.W..W.W.....W.W.W.W.W.W.W",
    "W.W..W.W.WWW.W.W.W.W.W.W.W",
    "W....W...W.W...W...W...W.W",
    "W.WW.WWW.W.W.WWWWW.W.W.W.W",
    "W.W..W...W.W.....W.W.W.W.W",
    "W.W.WW.WWW.WWWWW.W.W.W.W.W",
    "W.W....W...W.....W.W.W.W.W",
    "W.WWWW.W.W.W.WWW.W.W.W.W.W",
    "W......W.W.....W...W.W.W.W",
    "WWWWWWWWWWWWWWWWWWWWWWWWWWWW",
    "W............##............W",
    "W.WW.W.WWWWWWW.W.W.W.WWW.W.W",
    "W.W..W.W.....W.W.W.W.W.W.W",
    "W.W..W.W.WWW.W.W.W.W.W.W.W",
    "W....W...W.W...W...W...W.W",
    "WWWWWWWWWWWWWWWWWWWWWWWWWWWW",
];

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

#[derive(Clone)]
struct Ghost {
    x: f32,
    y: f32,
    direction: Direction,
    color: Color,
}

impl Ghost {
    fn new(x: f32, y: f32, color: Color) -> Self {
        Ghost {
            x,
            y,
            direction: Direction::Left,
            color,
        }
    }

    fn update(&mut self, walls: &[graphics::Rect]) {
        let mut rng = rand::thread_rng();
        
        // Ensure ghosts stay within the cage
        if self.x < GHOST_CAGE.0 || self.x > GHOST_CAGE.2 || self.y < GHOST_CAGE.1 || self.y > GHOST_CAGE.3 {
            self.direction = match rng.gen_range(0..4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                _ => Direction::Right,
            };
        }

        let (dx, dy) = match self.direction {
            Direction::Up => (0.0, -MOVEMENT_SPEED),
            Direction::Down => (0.0, MOVEMENT_SPEED),
            Direction::Left => (-MOVEMENT_SPEED, 0.0),
            Direction::Right => (MOVEMENT_SPEED, 0.0),
            Direction::None => (0.0, 0.0),
        };

        let new_x = self.x + dx;
        let new_y = self.y + dy;
        let ghost_rect = graphics::Rect::new(new_x, new_y, GHOST_SIZE, GHOST_SIZE);

        if walls.iter().any(|wall| wall.overlaps(&ghost_rect)) {
            // Change direction randomly when hitting a wall
            self.direction = match rng.gen_range(0..4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                _ => Direction::Right,
            };
        } else {
            self.x = new_x;
            self.y = new_y;
        }

        // Random direction change occasionally
        if rng.gen_bool(0.02) {
            self.direction = match rng.gen_range(0..4) {
                0 => Direction::Up,
                1 => Direction::Down,
                2 => Direction::Left,
                _ => Direction::Right,
            };
        }
    }
}

struct MainState {
    pacman_x: f32,
    pacman_y: f32,
    current_direction: Direction,
    requested_direction: Direction,
    walls: Vec<graphics::Rect>,
    dots: Vec<ggez::mint::Point2<f32>>,
    ghosts: Vec<Ghost>,
    score: u32,
    lives: i32,
    animation_start: Instant,
    mouth_open: bool,
    game_over: bool,
}

impl MainState {
    pub fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut walls = Vec::new();
        let mut dots = Vec::new();
        let mut ghosts = Vec::new();

        for (y, row) in MAP_STR.iter().enumerate() {
            for (x, cell) in row.chars().enumerate() {
                let pos_x = x as f32 * CELL_SIZE;
                let pos_y = y as f32 * CELL_SIZE;

                match cell {
                    'W' => walls.push(graphics::Rect::new(
                        pos_x,
                        pos_y,
                        THIN_WALL_SIZE,
                        THIN_WALL_SIZE,
                    )),
                    '.' => dots.push(ggez::mint::Point2 {
                        x: pos_x + CELL_SIZE / 2.0,
                        y: pos_y + CELL_SIZE / 2.0,
                    }),
                    'X' => {
                        // Handle special cases for 'X' if needed
                    },
                    _ => {}
                }
            }
        }

        // Ensure Pac-Man starts in a position not inside a wall
        let pacman_start_x = CELL_SIZE * 1.5;
        let pacman_start_y = CELL_SIZE * 1.5;

        // Create ghosts with different colors within the cage
        ghosts.push(Ghost::new(CELL_SIZE * 6.0, CELL_SIZE * 6.0, Color::RED));
        ghosts.push(Ghost::new(CELL_SIZE * 8.0, CELL_SIZE * 6.0, Color::CYAN));
        ghosts.push(Ghost::new(CELL_SIZE * 6.0, CELL_SIZE * 8.0, Color::MAGENTA));
        ghosts.push(Ghost::new(CELL_SIZE * 8.0, CELL_SIZE * 8.0, Color::from_rgb(255, 182, 255)));

        Ok(MainState {
            pacman_x: pacman_start_x,
            pacman_y: pacman_start_y,
            current_direction: Direction::None,
            requested_direction: Direction::None,
            walls,
            dots,
            ghosts,
            score: 0,
            lives: 3,
            animation_start: Instant::now(),
            mouth_open: true,
            game_over: false,
        })
    }

    fn can_move(&self, x: f32, y: f32) -> bool {
        let pacman_rect = graphics::Rect::new(x, y, PACMAN_SIZE, PACMAN_SIZE);
        !self.walls.iter().any(|wall| wall.overlaps(&pacman_rect))
    }

    fn check_ghost_collision(&mut self) {
        let pacman_center = ggez::mint::Point2 {
            x: self.pacman_x + PACMAN_SIZE / 2.0,
            y: self.pacman_y + PACMAN_SIZE / 2.0,
        };

        for ghost in &self.ghosts {
            let ghost_center = ggez::mint::Point2 {
                x: ghost.x + GHOST_SIZE / 2.0,
                y: ghost.y + GHOST_SIZE / 2.0,
            };

            let distance = ((ghost_center.x - pacman_center.x).powi(2) +
                            (ghost_center.y - pacman_center.y).powi(2)).sqrt();

            if distance < (PACMAN_SIZE + GHOST_SIZE) / 2.0 {
                self.game_over = true; // End the game on collision
            }
        }
    }
}

impl EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.game_over {
            return Ok(());
        }

        // Update mouth animation
        if self.animation_start.elapsed().as_millis() > 200 {
            self.mouth_open = !self.mouth_open;
            self.animation_start = Instant::now();
        }

        // Try movement in requested direction
        let (dx, dy) = match self.requested_direction {
            Direction::Up => (0.0, -MOVEMENT_SPEED),
            Direction::Down => (0.0, MOVEMENT_SPEED),
            Direction::Left => (-MOVEMENT_SPEED, 0.0),
            Direction::Right => (MOVEMENT_SPEED, 0.0),
            Direction::None => (0.0, 0.0),
        };

        if self.can_move(self.pacman_x + dx, self.pacman_y + dy) {
            self.pacman_x += dx;
            self.pacman_y += dy;
            self.current_direction = self.requested_direction;
        }

        // Update ghosts
        for ghost in &mut self.ghosts {
            ghost.update(&self.walls);
        }

        // Check collisions
        self.check_ghost_collision();

        // Collect dots
        self.dots.retain(|&dot| {
            let distance = ((self.pacman_x + PACMAN_SIZE / 2.0 - dot.x).powi(2) +
                             (self.pacman_y + PACMAN_SIZE / 2.0 - dot.y).powi(2)).sqrt();
            if distance < PACMAN_SIZE / 2.0 + DOT_SIZE / 2.0 {
                self.score += 10;
                false
            } else {
                true
            }
        });

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, Color::BLACK);

        // Draw walls
        for wall in &self.walls {
            let wall_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                *wall,
                Color::new(0.0, 0.0, 1.0, 1.0),
            )?;
            graphics::draw(ctx, &wall_mesh, DrawParam::default())?;
        }

        // Draw dots
        for dot in &self.dots {
            let dot_mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                *dot,
                DOT_SIZE/2.0,
                0.1,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &dot_mesh, DrawParam::default())?;
        }

        // Draw Pac-Man
        let pacman_mesh = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            ggez::mint::Point2 {
                x: self.pacman_x + PACMAN_SIZE/2.0,
                y: self.pacman_y + PACMAN_SIZE/2.0,
            },
            PACMAN_SIZE/2.0,
            0.1,
            Color::YELLOW,
        )?;
        graphics::draw(ctx, &pacman_mesh, DrawParam::default())?;

        // Draw ghosts
        for ghost in &self.ghosts {
            let ghost_mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                ggez::mint::Point2 {
                    x: ghost.x + GHOST_SIZE/2.0,
                    y: ghost.y + GHOST_SIZE/2.0,
                },
                GHOST_SIZE/2.0,
                0.1,
                ghost.color,
            )?;
            graphics::draw(ctx, &ghost_mesh, DrawParam::default())?;
        }

        // Draw score
        let score_text = graphics::Text::new(format!("Score: {}", self.score));
        graphics::draw(
            ctx,
            &score_text,
            DrawParam::default()
                .dest(ggez::mint::Point2 { x: 10.0, y: 10.0 })
                .color(Color::WHITE),
        )?;

        // Draw lives
        let lives_text = graphics::Text::new(format!("Lives: {}", self.lives));
        graphics::draw(
            ctx,
            &lives_text,
            DrawParam::default()
                .dest(ggez::mint::Point2 { x: 10.0, y: 30.0 })
                .color(Color::WHITE),
        )?;

        // Draw game over text if applicable
        if self.game_over {
            let game_over_text = graphics::Text::new("GAME OVER!");
            let text_dims = game_over_text.dimensions(ctx);
            let (w, h) = graphics::drawable_size(ctx);
            graphics::draw(
                ctx,
                &game_over_text,
                DrawParam::default()
                    .dest(ggez::mint::Point2 {
                        x: (w - text_dims.w) / 2.0,
                        y: (h - text_dims.h) / 2.0,
                    })
                    .color(Color::RED)
                    .scale([2.0, 2.0]),
            )?;
        }

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
        }
    

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        if !self.game_over {
            self.requested_direction = match keycode {
                KeyCode::Up => Direction::Up,
                KeyCode::Down => Direction::Down,
                KeyCode::Left => Direction::Left,
                KeyCode::Right => Direction::Right,
                _ => self.requested_direction,
            };
        }
    }
}
fn main() -> GameResult {
    let cb = ContextBuilder::new("pacman", "Your Name")
        .window_setup(ggez::conf::WindowSetup::default().title("Pac-Man"))
        .window_mode(ggez::conf::WindowMode::default()
            .dimensions(CELL_SIZE * 21.0, CELL_SIZE * 13.0)
            .resizable(false));

    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}