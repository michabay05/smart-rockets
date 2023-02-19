use rand::Rng;
use raylib::prelude::*;

const GENE_LEN: usize = 400;
const DEGREE_CHANGE: f32 = 10.0;

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 700;
const BACKGROUND_COLOR: Color = Color::RAYWHITE;

const ROCKET_COUNT: usize = 5;
const ROCKET_SPEED: f32 = 3.0;
const ROCKET_SIZE: Vector2 = Vector2::new(15.0, 45.0);
const ALIVE_ROCKET_COLOR: Color = Color::new(230, 230, 230, 255);
const DEAD_ROCKET_COLOR: Color = Color::new(230, 230, 230, 180); // 180 alpha value = 70.59% opacity
const SUCCESSFUL_ROCKET_COLOR: Color = Color::new(230, 138, 80, 255);

const TARGET_OUTER_COLOR: Color = Color::RAYWHITE;
const TARGET_INNER_COLOR: Color = Color::new(199, 111, 40, 255);
const TARGET_RADIUS: f32 = 30.0;

const WALL_SIZE: Vector2 = Vector2::new(200.0, 20.0);
const WALL_COUNT: usize = 2;
const WALL_COLOR: Color = Color::new(171, 171, 171, 255);

const TIMER_RECT_COLOR: Color = Color::LIME;
const TIMER_RECT_HEIGHT: i32 = 15;

// ================================== UTIL functions
fn rand_f32(min: f32, max: f32) -> f32 {
    let mut rng = rand::thread_rng();
    rng.gen_range(min..max) as f32
}

#[derive(Copy, Clone, Debug)]
struct DNA {
    pub genes: [f32; GENE_LEN],
    pub curr_gene: usize,
    pub fitness: f32,
}

impl DNA {
    fn new() -> Self {
        let mut instance = Self {
            genes: [0.0; GENE_LEN],
            curr_gene: 0,
            fitness: 0.0,
        };
        instance.randomize();
        instance
    }

    fn randomize(&mut self) {
        for el in &mut self.genes {
            *el = rand_f32(-DEGREE_CHANGE, DEGREE_CHANGE);
        }
    }

    fn next_angle(&mut self) -> f32 {
        if self.curr_gene >= GENE_LEN {
            return self.genes[GENE_LEN - 1];
        }
        let next_angle = self.genes[self.curr_gene];
        self.curr_gene += 1;
        next_angle
    }
}

#[derive(Clone, Copy)]
enum RocketState {
    Alive,
    Dead,
    Successful,
}

#[derive(Clone, Copy)]
struct Rocket {
    pub dna: DNA,

    pub pos: Vector2,
    pub color: Color,

    pub angle: f32,
    pub state: RocketState,
}

impl Rocket {
    fn new(pos: Vector2) -> Self {
        Self {
            dna: DNA::new(),
            pos,
            color: ALIVE_ROCKET_COLOR,
            angle: -90.0,
            state: RocketState::Alive,
        }
    }

    fn calc_offset(&self) -> Vector2 {
        Vector2::new(
            ROCKET_SPEED * self.angle.to_radians().cos(),
            ROCKET_SPEED * self.angle.to_radians().sin(),
        )
    }
}

struct World {
    pub rockets: [Rocket; ROCKET_COUNT],
    pub walls: [Rectangle; WALL_COUNT],
    pub target: Vector2,
    pub frame_counter: i32,
    pub timer_rect: Rectangle,
    mating_pool: Vec<Rocket>
}

impl World {
    fn new() -> Self {
        let mut instance = Self {
            rockets: [Rocket::new(Vector2::new(
                (SCREEN_WIDTH / 2) as f32,
                (SCREEN_HEIGHT - 75) as f32,
            )); ROCKET_COUNT],
            walls: [Rectangle::new(200.0, 100.0, WALL_SIZE.x, WALL_SIZE.y); WALL_COUNT],
            target: Vector2::new(100.0, 100.0),
            frame_counter: 0,
            timer_rect: Rectangle::new(
                0.0,
                (SCREEN_HEIGHT - TIMER_RECT_HEIGHT) as f32,
                SCREEN_WIDTH as f32,
                TIMER_RECT_HEIGHT as f32,
            ),
            mating_pool: vec![],
        };
        for rocket in &mut instance.rockets {
            rocket.dna.randomize();
        }
        instance
    }

    fn collision_rocket(&self, ind: usize) -> bool {
        self.collision_world(&self.rockets[ind].pos) || self.collision_wall(&self.rockets[ind].pos)
    }

    fn collision_world(&self, pos: &Vector2) -> bool {
        pos.x < 0.0 || pos.x > SCREEN_WIDTH as f32 || pos.y < 0.0 || pos.y > SCREEN_HEIGHT as f32
    }

    fn collision_wall(&self, pos: &Vector2) -> bool {
        for wall in &self.walls {
            if pos.x > wall.x && pos.x < wall.x + wall.width && pos.y > wall.y && pos.y < wall.y + wall.height {
                return true;
            }
        }
        false
    }
}

enum Actions {
    Pause,
    Reset,
    Nothing,
}

fn handle_input(rl: &RaylibHandle) -> Actions {
    if rl.is_key_pressed(KeyboardKey::KEY_SPACE) {
        return Actions::Pause;
    }
    if rl.is_key_pressed(KeyboardKey::KEY_R) {
        return Actions::Reset;
    }
    Actions::Nothing
}

fn update(world: &mut World) {
    let mut inds: Vec<usize> = vec![];
    for ind in 0..world.rockets.len() {
        if world.collision_rocket(ind) {
            inds.push(ind);
        }
    }

    for (ind, rocket) in world.rockets.iter_mut().enumerate() {
        if inds.contains(&ind) {
            rocket.state = RocketState::Dead;
            inds.remove(inds.iter().position(|x| *x == ind).unwrap());
            continue;
        }
        rocket.angle += rocket.dna.next_angle();

        let pos_offset = rocket.calc_offset();
        rocket.pos.x += pos_offset.x;
        rocket.pos.y += pos_offset.y;
    }
    world.frame_counter += 1;
    world.timer_rect.x -= (SCREEN_WIDTH as usize / GENE_LEN) as f32;
}

fn render(mut ctx: RaylibDrawHandle, world: &World) {
    ctx.clear_background(BACKGROUND_COLOR);
    ctx.draw_fps(15, 15);

    // Draw rockets
    for rocket in &world.rockets {
        let rocket_color = match rocket.state {
            RocketState::Dead => DEAD_ROCKET_COLOR,
            RocketState::Alive => ALIVE_ROCKET_COLOR,
            RocketState::Successful => SUCCESSFUL_ROCKET_COLOR,
        };
        ctx.draw_rectangle_pro(
            Rectangle::new(rocket.pos.x, rocket.pos.y, ROCKET_SIZE.x, ROCKET_SIZE.y),
            Vector2::new(ROCKET_SIZE.x / 2.0, ROCKET_SIZE.y / 2.0),
            rocket.angle + 90.0,
            rocket_color
        );
    }

    // Draw walls
    for wall in &world.walls {
        ctx.draw_rectangle_rec(wall, WALL_COLOR);
    }

    // Draw target
    ctx.draw_circle_v(world.target, TARGET_RADIUS, TARGET_OUTER_COLOR);
    ctx.draw_circle_v(world.target, TARGET_RADIUS / 2.0, TARGET_INNER_COLOR);
    ctx.draw_rectangle_rec(world.timer_rect, TIMER_RECT_COLOR);
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Hello, World")
        .msaa_4x()
        .vsync()
        .build();

    let mut world = World::new();
    let mut pause = false;
    while !rl.window_should_close() {
        // Handle input phase
        match handle_input(&rl) {
            Actions::Pause => pause = !pause,
            Actions::Reset => {}
            _ => {}
        };

        // Update phase
        if !pause {
            update(&mut world);
        }

        // Render phase
        let ctx = rl.begin_drawing(&thread);
        render(ctx, &world);
    }
}
