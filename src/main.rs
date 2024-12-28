//using ggez crate for GUI and event handlers
use ggez::{event, graphics, Context, ContextBuilder, GameResult, GameError};
use ggez::input::keyboard::{KeyCode, KeyMods};

//storing current location of pacman
struct MainState {
    pacman_x: f32,
    pacman_y: f32,
}

//storing initial coordinates at (100, 100)
impl MainState {
    pub fn new() -> MainState {
        MainState {
            pacman_x:100.0,
            pacman_y:100.0,
        }
    }
}

//implement EventHandler without generic parameters
impl event::EventHandler<GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        // update movement here
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        //black background
        graphics::clear(ctx, graphics::Color::from_rgb(0, 0, 0));

        //circular design of pacman
        let pacman = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            ggez::mint::Point2{ x: self.pacman_x, y: self.pacman_y},
            20.0, //radius
            0.1,
            graphics::Color::from_rgb(255, 255, 0), //yellow pacman
        )?;
        graphics::draw(ctx, &pacman, (ggez::mint::Point2 {x: 0.0, y: 0.0},))?;

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
       //matching user input to pacmans movement
        match keycode {
            KeyCode::Up => self.pacman_y -= 5.0,
            KeyCode::Down => self.pacman_y += 5.0,
            KeyCode::Left => self.pacman_x -= 5.0,
            KeyCode::Right => self.pacman_x += 5.0,

            //default ignore here
            _ => {}
        }
    }
}


fn main() -> GameResult<()> {
    //run as player 1
    let (ctx, event_loop) = ContextBuilder::new("rust_pacman", "Player 1")
        .build()?;
    let state = MainState::new();
    //run the game
    event::run(ctx, event_loop, state)
}
