//ggez for GUI
use ggez::{Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawParam};
use ggez::input::keyboard::{KeyCode, KeyMods};
use ggez::timer;
use std::time::Instant;
use rand::Rng;
use std::thread;

//constants for sizes, movement speeds, and durations
const CELL_SIZE: f32 = 30.0;
const PACMAN_SIZE: f32 = 25.0;
const DOT_SIZE: f32 = 6.0;
const GHOST_SIZE: f32 = 25.0;
const MOVEMENT_SPEED: f32 = 1.0;
const GHOST_SPEED: f32 = 0.5;     
const THIN_WALL_SIZE: f32 = 30.0;
const POWER_PELLET_SIZE: f32 = 15.0;
const POWER_PELLET_DURATION: f32 = 5.0; 
const VULNERABLE_GHOST_SPEED: f32 = 0.5;  

//W's represent walls, dots represent pellets. G represents Ghosts
const MAP_STR: [&'static str; 20] = [
    "WWWWWWWWWWWWWWWWWWWW",
    "W........W.........W",
    "W.WW.WWW.W.WWW.WW.WW",
    "W..................W",
    "W.WW.W.WWWWW.W.WW.WW",
    "W....W...W...W....WW",
    "WWWW.WWW.W.WWW.WWWWW",
    "   W.W.......W.W   W",
    "WWWW.W.WW WW.W.WWWWW",
    "W....... GG ......W",
    "WWWW.W.WWWWW.W.WWWWW",
    "   W.W.......W.....W",
    "WWWW.W.WWWWW.W.WWWWW",
    "W........W........WW",
    "W.WW.WWW.W.WWW.WW.WW",
    "W..W.....P.....W..WW",
    "WW.W.W.WWWWW.W.W.WWW",
    "W....W...W...W....WW",
    "W.WWWWWW.W.WWWWWW..W",
    "WWWWWWWWWWWWWWWWWWWW",
];

//derive clone, copy, and equality from direction
#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
    None,
}

//position arguments, directions, colors, and timers
#[derive(Clone)]
struct Ghost {
    x: f32,
    y: f32,
    direction: Direction,
    color: Color,
    target_x: f32,
    target_y: f32,
    is_vulnerable: bool,
    respawn_timer: f32,
    spawn_position: (f32, f32),
    confused_timer: f32,
}

impl Ghost {
    //ghost struct with following values
    fn new(x: f32, y: f32, color: Color) -> Self {
        Ghost {
            x,
            y,
            direction: Direction::Left,
            color,
            target_x: x,
            target_y: y,
            is_vulnerable: false,
            respawn_timer: 0.0,
            spawn_position: (x, y),
            confused_timer: 0.0,
        }
    }

    //for updating the graphics
    fn update(&mut self, walls: &[graphics::Rect], pacman_x: f32, pacman_y: f32) {
        let mut rng = rand::thread_rng();
        
        //Confused timer to introduce a bit of rng here. Prevents ghosts from streamlining to you
        if self.confused_timer > 0.0 {
            if rng.gen_bool(0.1) {
                self.target_x = rng.gen_range(0.0..600.0);
                self.target_y = rng.gen_range(0.0..600.0);
            }
        } else {
            if rng.gen_bool(0.05) {
                if rng.gen_bool(0.6) {
                    self.target_x = rng.gen_range(0.0..600.0);
                    self.target_y = rng.gen_range(0.0..600.0);
                } else {
                    self.target_x = pacman_x;
                    self.target_y = pacman_y;
                }
            }
        }

        //Calculate direction to target
        let _dx = self.target_x - self.x;
        let _dy = self.target_y - self.y;
        
        //Choose direction based on target position and available paths
        let possible_directions = vec![Direction::Up, Direction::Down, Direction::Left, Direction::Right];
        let mut valid_directions = Vec::new();

        for &dir in &possible_directions {
            let speed = if self.is_vulnerable {
                VULNERABLE_GHOST_SPEED
            } else {
                GHOST_SPEED
            };

            //potential direction
            let (test_dx, test_dy) = match dir {
                Direction::Up => (0.0, -speed),
                Direction::Down => (0.0, speed),
                Direction::Left => (-speed, 0.0),
                Direction::Right => (speed, 0.0),
                Direction::None => (0.0, 0.0),
            };

            //ghost cage
            let ghost_rect = graphics::Rect::new(
                self.x + test_dx,
                self.y + test_dy,
                GHOST_SIZE,
                GHOST_SIZE,
            );
            
            //pushing direction based on wall
            if !walls.iter().any(|wall| wall.overlaps(&ghost_rect)) {
                valid_directions.push(dir);
            }
        }

        //Update direction selection based on confused state
        if !valid_directions.is_empty() {
            let preferred_direction = if self.confused_timer > 0.0 {
                valid_directions[rng.gen_range(0..valid_directions.len())]
            } else {
                *valid_directions.iter().min_by_key(|&&dir| {
                    let (test_dx, test_dy) = match dir {
                        Direction::Up => (0.0, -1.0),
                        Direction::Down => (0.0, 1.0),
                        Direction::Left => (-1.0, 0.0),
                        Direction::Right => (1.0, 0.0),
                        Direction::None => (0.0, 0.0),
                    };
                    let distance = ((self.x + test_dx - self.target_x).powi(2) +
                                  (self.y + test_dy - self.target_y).powi(2)).sqrt();
                    (distance * 100.0) as i32
                }).unwrap_or(&Direction::None)
            };
            
            self.direction = preferred_direction;
        }

        //move ghost with GHOST_SPEED
        let (dx, dy) = match self.direction {
            Direction::Up => (0.0, -GHOST_SPEED),
            Direction::Down => (0.0, GHOST_SPEED),
            Direction::Left => (-GHOST_SPEED, 0.0),
            Direction::Right => (GHOST_SPEED, 0.0),
            Direction::None => (0.0, 0.0),
        };

        let new_x = self.x + dx;
        let new_y = self.y + dy;
        let ghost_rect = graphics::Rect::new(new_x, new_y, GHOST_SIZE, GHOST_SIZE);

        if !walls.iter().any(|wall| wall.overlaps(&ghost_rect)) {
            self.x = new_x;
            self.y = new_y;
        }
    }

    //for resetting ghosts after eating them
    fn reset_position(&mut self) {
        self.x = self.spawn_position.0;
        self.y = self.spawn_position.1;
        self.is_vulnerable = false;
        self.respawn_timer = 0.0;
        self.direction = Direction::Left;
        self.confused_timer = 3.0;  
    }
}

//state of the game
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
    show_menu: bool,
    power_pellets: Vec<ggez::mint::Point2<f32>>,
    power_pellet_active: bool,
    power_pellet_timer: f32,
    thread_count: usize,
}

impl MainState {
    pub fn new(_ctx: &mut Context) -> GameResult<MainState> {
        let mut walls = Vec::new();
        let mut dots = Vec::new();
        let mut power_pellets = Vec::new();
        let mut ghosts = Vec::new();
        let mut pacman_start_x = 0.0;
        let mut pacman_start_y = 0.0;

        //find center position for ghost spawn
        let center_x = (MAP_STR[0].len() as f32 / 2.0).floor() * CELL_SIZE;
        let center_y = (MAP_STR.len() as f32 / 2.0).floor() * CELL_SIZE;

        //add 'power' pellets in corners
        power_pellets.push(ggez::mint::Point2 { x: CELL_SIZE * 1.5, y: CELL_SIZE * 1.5 });
        power_pellets.push(ggez::mint::Point2 { x: CELL_SIZE * (MAP_STR[0].len() as f32 - 1.5), y: CELL_SIZE * 1.5 });
        power_pellets.push(ggez::mint::Point2 { x: CELL_SIZE * 1.5, y: CELL_SIZE * (MAP_STR.len() as f32 - 1.5) });
        power_pellets.push(ggez::mint::Point2 { x: CELL_SIZE * (MAP_STR[0].len() as f32 - 1.5), y: CELL_SIZE * (MAP_STR.len() as f32 - 1.5) });

        //initialize ghosts in center
        ghosts.push(Ghost::new(center_x, center_y, Color::RED));
        ghosts.push(Ghost::new(center_x, center_y, Color::CYAN));
        ghosts.push(Ghost::new(center_x, center_y, Color::MAGENTA));

        //parse map and create game objects
        for (y, row) in MAP_STR.iter().enumerate() {
            for (x, cell) in row.chars().enumerate() {
                let pos_x = x as f32 * CELL_SIZE;
                let pos_y = y as f32 * CELL_SIZE;

                //walls based on given map above.
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
                    'G' => {
                        ghosts.push(Ghost::new(pos_x, pos_y, Color::RED));
                        if ghosts.len() > 1 {
                            ghosts.push(Ghost::new(pos_x, pos_y, Color::CYAN));
                        }
                        if ghosts.len() > 2 {
                            ghosts.push(Ghost::new(pos_x, pos_y, Color::MAGENTA));
                        }
                    },
                    'P' => {
                        pacman_start_x = pos_x + (CELL_SIZE - PACMAN_SIZE) / 2.0;
                        pacman_start_y = pos_y + (CELL_SIZE - PACMAN_SIZE) / 2.0;
                    },
                    _ => {}
                }
            }
        }
        //if ok, set default values for main state
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
            show_menu: false,
            power_pellets,
            power_pellet_active: false,
            power_pellet_timer: 0.0,
            thread_count: thread::available_parallelism().map_or(1, |p| p.get()),
        })
    }
    //reset game by enumerating over x and y and resetting particles
    fn reset_game(&mut self) {
        //reset Pacman position
        for (y, row) in MAP_STR.iter().enumerate() {
            for (x, cell) in row.chars().enumerate() {
                if cell == 'P' {
                    self.pacman_x = x as f32 * CELL_SIZE + (CELL_SIZE - PACMAN_SIZE) / 2.0;
                    self.pacman_y = y as f32 * CELL_SIZE + (CELL_SIZE - PACMAN_SIZE) / 2.0;
                    break;
                }
            }
        }

        //store ghost spawn positions
        let mut ghost_spawn_positions = Vec::new();
        for (y, row) in MAP_STR.iter().enumerate() {
            for (x, cell) in row.chars().enumerate() {
                if cell == 'G' {
                    ghost_spawn_positions.push((
                        x as f32 * CELL_SIZE + (CELL_SIZE - GHOST_SIZE) / 2.0,
                        y as f32 * CELL_SIZE + (CELL_SIZE - GHOST_SIZE) / 2.0
                    ));
                }
            }
        }

        //reset ghosts by repushing them in their spawn position
        self.ghosts.clear();
        if !ghost_spawn_positions.is_empty() {
            let pos = ghost_spawn_positions[0];
            self.ghosts.push(Ghost::new(pos.0, pos.1, Color::RED));
            self.ghosts.push(Ghost::new(pos.0, pos.1, Color::CYAN));
            self.ghosts.push(Ghost::new(pos.0, pos.1, Color::MAGENTA));
            
            //store spawn positions for each ghost
            for ghost in &mut self.ghosts {
                ghost.spawn_position = pos;
            }
        }

        //reset power pellets
        self.power_pellets.clear();
        self.power_pellets.push(ggez::mint::Point2 { x: CELL_SIZE * 1.5, y: CELL_SIZE * 1.5 });
        self.power_pellets.push(ggez::mint::Point2 { x: CELL_SIZE * (MAP_STR[0].len() as f32 - 1.5), y: CELL_SIZE * 1.5 });
        self.power_pellets.push(ggez::mint::Point2 { x: CELL_SIZE * 1.5, y: CELL_SIZE * (MAP_STR.len() as f32 - 1.5) });
        self.power_pellets.push(ggez::mint::Point2 { x: CELL_SIZE * (MAP_STR[0].len() as f32 - 1.5), y: CELL_SIZE * (MAP_STR.len() as f32 - 1.5) });

        //reset game state
        self.score = 0;
        self.lives = 3;
        self.game_over = false;
        self.show_menu = false;
        self.current_direction = Direction::None;
        self.requested_direction = Direction::None;
        self.power_pellet_active = false;
        self.power_pellet_timer = 0.0;
        
        //recreate dots
        self.dots.clear();
        for (y, row) in MAP_STR.iter().enumerate() {
            for (x, cell) in row.chars().enumerate() {
                if cell == '.' {
                    self.dots.push(ggez::mint::Point2 {
                        x: x as f32 * CELL_SIZE + CELL_SIZE / 2.0,
                        y: y as f32 * CELL_SIZE + CELL_SIZE / 2.0,
                    });
                }
            }
        }
        self.thread_count = thread::available_parallelism().map_or(1, |p| p.get());
    }

    //possibility for movement depends on the cell grid they 'snap' to
    fn can_move(&self, direction: Direction) -> bool {
        let (dx, dy) = match direction {
            Direction::Up => (0.0, -CELL_SIZE),
            Direction::Down => (0.0, CELL_SIZE),
            Direction::Left => (-CELL_SIZE, 0.0),
            Direction::Right => (CELL_SIZE, 0.0),
            Direction::None => (0.0, 0.0),
        };

        //'snap' pacman to a grid cell to allow for smoother grid tracing
        let test_x = (self.pacman_x / CELL_SIZE).round() * CELL_SIZE + dx + (CELL_SIZE - PACMAN_SIZE) / 2.0;
        let test_y = (self.pacman_y / CELL_SIZE).round() * CELL_SIZE + dy + (CELL_SIZE - PACMAN_SIZE) / 2.0;
        
        let pacman_rect = graphics::Rect::new(test_x, test_y, PACMAN_SIZE, PACMAN_SIZE);
        !self.walls.iter().any(|wall| wall.overlaps(&pacman_rect))
    }

    //life counter
    fn check_ghost_collision(&mut self) {
        if self.lives <= 0 {
            return;
        }

        let pacman_center = ggez::mint::Point2 {
            x: self.pacman_x + PACMAN_SIZE / 2.0,
            y: self.pacman_y + PACMAN_SIZE / 2.0,
        };

        for ghost in &mut self.ghosts {
            if ghost.respawn_timer <= 0.0 {
                let ghost_center = ggez::mint::Point2 {
                    x: ghost.x + GHOST_SIZE / 2.0,
                    y: ghost.y + GHOST_SIZE / 2.0,
                };

                let distance = ((ghost_center.x - pacman_center.x).powi(2) +
                              (ghost_center.y - pacman_center.y).powi(2)).sqrt();

                if distance < (PACMAN_SIZE + GHOST_SIZE) / 2.0 {
                    if ghost.is_vulnerable {
                        ghost.reset_position();
                        self.score += 200;
                    } else {
                        self.lives -= 1;
                        if self.lives <= 0 {
                            self.game_over = true;
                            self.show_menu = true;
                            self.lives = 0;
                            return;
                        }
                        //reset positions
                        self.reset_pacman_position();
                        for ghost in &mut self.ghosts {
                            ghost.reset_position();
                        }
                        break;
                    }
                }
            }
        }
    }

    //function to make pacman an entity of the current cell it resides in. Allows for easier movement without getting stuck on edges
    fn snap_to_grid(&mut self) {
        //round to nearest grid position
        self.pacman_x = (self.pacman_x / CELL_SIZE).round() * CELL_SIZE + (CELL_SIZE - PACMAN_SIZE) / 2.0;
        self.pacman_y = (self.pacman_y / CELL_SIZE).round() * CELL_SIZE + (CELL_SIZE - PACMAN_SIZE) / 2.0;
    }

    fn is_at_grid_center(&self) -> bool {
        let grid_x = (self.pacman_x - (CELL_SIZE - PACMAN_SIZE) / 2.0) / CELL_SIZE;
        let grid_y = (self.pacman_y - (CELL_SIZE - PACMAN_SIZE) / 2.0) / CELL_SIZE;
        
        let center_x = grid_x.round() * CELL_SIZE + (CELL_SIZE - PACMAN_SIZE) / 2.0;
        let center_y = grid_y.round() * CELL_SIZE + (CELL_SIZE - PACMAN_SIZE) / 2.0;
        
        (self.pacman_x - center_x).abs() < 1.0 && (self.pacman_y - center_y).abs() < 1.0
    }

    //resetting position and directions
    fn reset_pacman_position(&mut self) {
        //find and reset Pacman's position from the map
        for (y, row) in MAP_STR.iter().enumerate() {
            for (x, cell) in row.chars().enumerate() {
                if cell == 'P' {
                    self.pacman_x = x as f32 * CELL_SIZE + (CELL_SIZE - PACMAN_SIZE) / 2.0;
                    self.pacman_y = y as f32 * CELL_SIZE + (CELL_SIZE - PACMAN_SIZE) / 2.0;
                    break;
                }
            }
        }
        self.current_direction = Direction::None;
        self.requested_direction = Direction::None;
    }
}

impl EventHandler<ggez::GameError> for MainState {
    
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = timer::delta(ctx).as_secs_f32();

        //update power pellet timer
        if self.power_pellet_active {
            self.power_pellet_timer -= dt;
            if self.power_pellet_timer <= 0.0 {
                self.power_pellet_active = false;
                for ghost in &mut self.ghosts {
                    ghost.is_vulnerable = false;
                }
            }
        }

        //update ghost timers
        for ghost in &mut self.ghosts {
            if ghost.confused_timer > 0.0 {
                ghost.confused_timer -= dt;
            }
            if ghost.respawn_timer > 0.0 {
                ghost.respawn_timer -= dt;
            }
        }

        //check power pellet collection
        self.power_pellets.retain(|&pellet| {
            let distance = ((self.pacman_x + PACMAN_SIZE / 2.0 - pellet.x).powi(2) +
                          (self.pacman_y + PACMAN_SIZE / 2.0 - pellet.y).powi(2)).sqrt();
            if distance < PACMAN_SIZE / 2.0 + POWER_PELLET_SIZE / 2.0 {
                self.power_pellet_active = true;
                self.power_pellet_timer = POWER_PELLET_DURATION;
                for ghost in &mut self.ghosts {
                    ghost.is_vulnerable = true;
                }
                false
            } else {
                true
            }
        });

        //check ghost collisions
        for ghost in &mut self.ghosts {
            if ghost.respawn_timer <= 0.0 {
                let distance = ((self.pacman_x + PACMAN_SIZE / 2.0 - ghost.x - GHOST_SIZE / 2.0).powi(2) +
                              (self.pacman_y + PACMAN_SIZE / 2.0 - ghost.y - GHOST_SIZE / 2.0).powi(2)).sqrt();
                
                if distance < (PACMAN_SIZE + GHOST_SIZE) / 2.0 {
                    if ghost.is_vulnerable {
                        ghost.reset_position();
                        self.score += 200;
                    } else if !ghost.is_vulnerable {
                        self.lives -= 1;
                        if self.lives <= 0 {
                            self.game_over = true;
                            self.show_menu = true;
                        }
                    }
                }
            }
        }

        if self.game_over {
            return Ok(());
        }

        //update mouth animation
        if self.animation_start.elapsed().as_millis() > 200 {
            self.mouth_open = !self.mouth_open;
            self.animation_start = Instant::now();
        }

        //if at grid center, allow direction change if the new direction is valid
        if self.is_at_grid_center() {
            if self.can_move(self.requested_direction) {
                self.current_direction = self.requested_direction;
            }
        }

        //move in current direction
        let (dx, dy) = match self.current_direction {
            Direction::Up => (0.0, -MOVEMENT_SPEED),
            Direction::Down => (0.0, MOVEMENT_SPEED),
            Direction::Left => (-MOVEMENT_SPEED, 0.0),
            Direction::Right => (MOVEMENT_SPEED, 0.0),
            Direction::None => (0.0, 0.0),
        };

        //update movement
        let new_x = self.pacman_x + dx;
        let new_y = self.pacman_y + dy;
        let pacman_rect = graphics::Rect::new(new_x, new_y, PACMAN_SIZE, PACMAN_SIZE);

        if !self.walls.iter().any(|wall| wall.overlaps(&pacman_rect)) {
            self.pacman_x = new_x;
            self.pacman_y = new_y;
        } else {
            //if we hit a wall, snap to grid
            self.snap_to_grid();
            self.current_direction = Direction::None;
        }

        //update ghosts with Pac-Man's position
        for ghost in &mut self.ghosts {
            ghost.update(&self.walls, self.pacman_x, self.pacman_y);
        }

        //check collisions
        self.check_ghost_collision();

        //collect dots
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

        //draw walls
        for wall in &self.walls {
            let wall_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                *wall,
                Color::new(0.0, 0.0, 1.0, 1.0),
            )?;
            graphics::draw(ctx, &wall_mesh, DrawParam::default())?;
        }

        //draw dots
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

        //draw Pac-Man
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

        //draw ghosts
        for ghost in &self.ghosts {
            if ghost.respawn_timer <= 0.0 {
                let color = if ghost.is_vulnerable {
                    Color::BLUE
                } else {
                    ghost.color
                };

                let ghost_mesh = graphics::Mesh::new_circle(
                    ctx,
                    graphics::DrawMode::fill(),
                    ggez::mint::Point2 {
                        x: ghost.x + GHOST_SIZE/2.0,
                        y: ghost.y + GHOST_SIZE/2.0,
                    },
                    GHOST_SIZE/2.0,
                    0.1,
                    color,
                )?;
                graphics::draw(ctx, &ghost_mesh, DrawParam::default())?;
            }
        }

        //draw score
        let score_text = graphics::Text::new(format!("Score: {}", self.score));
        graphics::draw(
            ctx,
            &score_text,
            DrawParam::default()
                .dest(ggez::mint::Point2 { x: 10.0, y: 10.0 })
                .color(Color::WHITE),
        )?;

        //draw lives
        let lives_text = graphics::Text::new(format!("Lives: {}", self.lives));
        graphics::draw(
            ctx,
            &lives_text,
            DrawParam::default()
                .dest(ggez::mint::Point2 { x: 10.0, y: 30.0 })
                .color(Color::WHITE),
        )?;

        //draw thread count
        let thread_text = graphics::Text::new(format!("Threads: {}", self.thread_count));
        graphics::draw(
            ctx,
            &thread_text,
            DrawParam::default()
                .dest(ggez::mint::Point2 { x: 10.0, y: 50.0 })
                .color(Color::WHITE),
        )?;

        //draw game over text if applicable
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

        //draw game over menu
        if self.show_menu {
            let (w, h) = graphics::drawable_size(ctx);
            
            //draw semi-transparent background
            let background = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(0.0, 0.0, w, h),
                Color::new(0.0, 0.0, 0.0, 0.7),
            )?;
            graphics::draw(ctx, &background, DrawParam::default())?;

            //draw menu box
            let menu_width = 300.0;
            let menu_height = 200.0;
            let menu_x = (w - menu_width) / 2.0;
            let menu_y = (h - menu_height) / 2.0;

            let menu_bg = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(menu_x, menu_y, menu_width, menu_height),
                Color::new(0.2, 0.2, 0.2, 1.0),
            )?;
            graphics::draw(ctx, &menu_bg, DrawParam::default())?;

            //draw game over text
            let game_over_text = graphics::Text::new("GAME OVER!");
            let text_dims = game_over_text.dimensions(ctx);
            graphics::draw(
                ctx,
                &game_over_text,
                DrawParam::default()
                    .dest(ggez::mint::Point2 {
                        x: menu_x + (menu_width - text_dims.w) / 2.0,
                        y: menu_y + 30.0,
                    })
                    .color(Color::RED)
                    .scale([2.0, 2.0]),
            )?;

            //draw final score
            let score_text = graphics::Text::new(format!("Final Score: {}", self.score));
            let score_dims = score_text.dimensions(ctx);
            graphics::draw(
                ctx,
                &score_text,
                DrawParam::default()
                    .dest(ggez::mint::Point2 {
                        x: menu_x + (menu_width - score_dims.w) / 2.0,
                        y: menu_y + 80.0,
                    })
                    .color(Color::WHITE),
            )?;

            //draw buttons
            let button_width = 120.0;
            let button_height = 40.0;
            
            //play Again button
            let play_button = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    menu_x + 30.0,
                    menu_y + 120.0,
                    button_width,
                    button_height,
                ),
                Color::GREEN,
            )?;
            graphics::draw(ctx, &play_button, DrawParam::default())?;
            
            let play_text = graphics::Text::new("Play Again");
            let play_dims = play_text.dimensions(ctx);
            graphics::draw(
                ctx,
                &play_text,
                DrawParam::default()
                    .dest(ggez::mint::Point2 {
                        x: menu_x + 30.0 + (button_width - play_dims.w) / 2.0,
                        y: menu_y + 130.0,
                    })
                    .color(Color::BLACK),
            )?;

            //exit button
            let exit_button = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(
                    menu_x + menu_width - button_width - 30.0,
                    menu_y + 120.0,
                    button_width,
                    button_height,
                ),
                Color::RED,
            )?;
            graphics::draw(ctx, &exit_button, DrawParam::default())?;
            
            let exit_text = graphics::Text::new("Exit");
            let exit_dims = exit_text.dimensions(ctx);
            graphics::draw(
                ctx,
                &exit_text,
                DrawParam::default()
                    .dest(ggez::mint::Point2 {
                        x: menu_x + menu_width - button_width - 30.0 + (button_width - exit_dims.w) / 2.0,
                        y: menu_y + 130.0,
                    })
                    .color(Color::WHITE),
            )?;
        }

        //draw power pellets
        for pellet in &self.power_pellets {
            let pellet_mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                *pellet,
                POWER_PELLET_SIZE/2.0,
                0.1,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &pellet_mesh, DrawParam::default())?;
        }

        graphics::present(ctx)?;
        timer::yield_now();
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        ctx: &mut Context,
        button: event::MouseButton,
        x: f32,
        y: f32,
    ) {
        if self.show_menu && button == event::MouseButton::Left {
            let (w, h) = graphics::drawable_size(ctx);
            let menu_width = 300.0;
            let menu_height = 200.0;
            let menu_x = (w - menu_width) / 2.0;
            let menu_y = (h - menu_height) / 2.0;
            let button_width = 120.0;
            let button_height = 40.0;

            //check Play Again button
            if x >= menu_x + 30.0
                && x <= menu_x + 30.0 + button_width
                && y >= menu_y + 120.0
                && y <= menu_y + 120.0 + button_height
            {
                self.reset_game();
            }

            //check Exit button
            if x >= menu_x + menu_width - button_width - 30.0
                && x <= menu_x + menu_width - button_width - 30.0 + button_width
                && y >= menu_y + 120.0
                && y <= menu_y + 120.0 + button_height
            {
                event::quit(ctx);
            }
        }
    }
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        if !self.game_over {
            let new_direction = match keycode {
                KeyCode::Up => Direction::Up,
                KeyCode::Down => Direction::Down,
                KeyCode::Left => Direction::Left,
                KeyCode::Right => Direction::Right,
                _ => self.requested_direction,
            };

            //update requested direction immediately
            self.requested_direction = new_direction;

            //if we're at a grid center and the new direction is valid, change immediately
            if self.is_at_grid_center() && self.can_move(new_direction) {
                self.current_direction = new_direction;
            }
        }
    }
}

//main function to call window setup and run event given context and state
fn main() -> GameResult {
    let cb = ContextBuilder::new("pacman", "Your Name")
        .window_setup(ggez::conf::WindowSetup::default().title("Pac-Man"))
        .window_mode(ggez::conf::WindowMode::default()
            .dimensions(CELL_SIZE * MAP_STR[0].len() as f32, CELL_SIZE * MAP_STR.len() as f32)
            .resizable(false));

    let (mut ctx, event_loop) = cb.build()?;
    let state = MainState::new(&mut ctx)?;
    event::run(ctx, event_loop, state)
}