use std::ops::Sub;

use rand::Rng;
use raylib::prelude::*;

const GENE_LEN: usize = 400;
const MUTATION_RATE: f32 = 0.03;
const DEGREE_CHANGE: f32 = 10.0;

const SCREEN_WIDTH: i32 = 1000;
const SCREEN_HEIGHT: i32 = 650;
const BACKGROUND_COLOR: Color = Color::new(24, 24, 24, 255);

const ROCKET_COUNT: usize = 80;
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
        Self {
            genes: [0.0; GENE_LEN],
            curr_gene: 0,
            fitness: 0.0,
        }
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

    fn crossover(parent_a: &Self, parent_b: &Self) -> Self {
        let mut rng = rand::thread_rng();
        let rand_split_point = rng.gen_range(0..GENE_LEN);
        let mut child = Self::new();
        for i in 0..GENE_LEN {
            if i < rand_split_point {
                child.genes[i] = parent_a.genes[i];
            } else {
                child.genes[i] = parent_b.genes[i];
            }
        }
        child
    }

    fn mutate(dna: &mut DNA) {
        for i in 0..GENE_LEN {
            let rand_num = rand::random::<f32>();
            if rand_num < MUTATION_RATE {
                dna.genes[i] = rand_f32(-DEGREE_CHANGE, DEGREE_CHANGE);
            }
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum RocketState {
    Alive,
    Dead,
    Successful,
}

#[derive(Clone, Copy)]
struct Rocket {
    pub dna: DNA,
    pub pos: Vector2,
    pub state: RocketState,
    pub angle: f32,
    pub dist_from_target: f32,
}

impl Rocket {
    fn new(pos: Vector2) -> Self {
        Self {
            dna: DNA::new(),
            pos,
            angle: -90.0,
            state: RocketState::Alive,
            dist_from_target: 0.0,
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
    pub alive_count: i32,
    pub walls: [Rectangle; WALL_COUNT],
    pub target: Vector2,
    pub frame_counter: u32,
    pub timer_rect: Rectangle,
    pub generation: u32,
    mating_pool: Vec<usize>,
}

impl World {
    fn new() -> Self {
        let mut instance = Self {
            rockets: [Rocket::new(Vector2::new(
                (SCREEN_WIDTH / 2) as f32,
                (SCREEN_HEIGHT - 75) as f32,
            )); ROCKET_COUNT],
            alive_count: ROCKET_COUNT as i32,
            walls: [
                Rectangle::new(300.0, 250.0, WALL_SIZE.x, WALL_SIZE.y),
                Rectangle::new(150.0, 300.0, WALL_SIZE.x, WALL_SIZE.y),
            ],
            target: Vector2::new(100.0, 100.0),
            frame_counter: 0,
            timer_rect: Rectangle::new(
                0.0,
                (SCREEN_HEIGHT - TIMER_RECT_HEIGHT) as f32,
                SCREEN_WIDTH as f32,
                TIMER_RECT_HEIGHT as f32,
            ),
            generation: 0,
            mating_pool: vec![],
        };
        for rocket in &mut instance.rockets {
            rocket.dna.randomize();
        }
        instance
    }

    fn restart(&mut self) {
        self.calc_fitness();
        self.gen_mating_pool();
        let mut instance = Self::new();
        self.selection(&mut instance.rockets);
        instance.generation = self.generation + 1;

        *self = instance;
    }

    fn calc_dist_from_target(&mut self) {
        for rocket in &mut self.rockets {
            let pos_diff = self.target.sub(rocket.pos);
            let hyp = (pos_diff.x.powi(2)) + (pos_diff.y.powi(2));
            rocket.dist_from_target = hyp.sqrt();
        }
    }

    fn calc_fitness(&mut self) {
        self.calc_dist_from_target();
        let dist_from_target_sum: f32 = self.rockets.iter().map(|el| el.dist_from_target).sum();

        for rocket in &mut self.rockets {
            rocket.dna.fitness = 1.0 - (rocket.dist_from_target / dist_from_target_sum);
        }
    }

    fn gen_mating_pool(&mut self) {
        self.mating_pool.clear();

        for (ind, rocket) in self.rockets.iter().enumerate() {
            let n = rocket.dna.fitness * 100.0;
            let n = match rocket.state {
                RocketState::Dead => n * (0.6),
                RocketState::Alive => n,
                RocketState::Successful => n * 2.0,
            };
            for _ in 0..(n.floor() as usize) {
                self.mating_pool.push(ind);
            }
        }
    }

    fn selection(&self, rockets: &mut [Rocket]) {
        for rocket in rockets.iter_mut() {
            let mut rocket_inst = Rocket::new(Vector2::new(
                (SCREEN_WIDTH / 2) as f32,
                (SCREEN_HEIGHT - 75) as f32,
            ));

            let mut rng = rand::thread_rng();
            let rand_a = rng.gen_range(0..self.mating_pool.len());
            let rand_b = rng.gen_range(0..self.mating_pool.len());
            let parent_a_ind = self.mating_pool[rand_a];
            let parent_b_ind = self.mating_pool[rand_b];
            rocket_inst.dna = DNA::crossover(
                &self.rockets[parent_a_ind].dna,
                &self.rockets[parent_b_ind].dna,
            );
            DNA::mutate(&mut rocket_inst.dna);

            *rocket = rocket_inst;
        }
    }

    fn collision_rocket(&self, ind: usize) -> bool {
        self.collision_world(&self.rockets[ind].pos) || self.collision_wall(&self.rockets[ind].pos)
    }

    fn collision_world(&self, pos: &Vector2) -> bool {
        pos.x < 0.0 || pos.x > SCREEN_WIDTH as f32 || pos.y < 0.0 || pos.y > SCREEN_HEIGHT as f32
    }

    fn collision_wall(&self, pos: &Vector2) -> bool {
        for wall in &self.walls {
            if pos.x > wall.x
                && pos.x < wall.x + wall.width
                && pos.y > wall.y
                && pos.y < wall.y + wall.height
            {
                return true;
            }
        }
        false
    }

    fn collision_target(&self, ind: usize) -> bool {
        let pos = self.rockets[ind].pos;
        let diff = self.target.sub(pos);
        let dist_from_center = (diff.x.powi(2) + diff.y.powi(2)).sqrt();
        dist_from_center < TARGET_RADIUS
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
    if world.frame_counter == GENE_LEN as u32 {
        world.restart();
        return;
    }
    let mut dead_inds: Vec<usize> = vec![];
    let mut succ_inds: Vec<usize> = vec![];
    for ind in 0..world.rockets.len() {
        if world.collision_rocket(ind) {
            dead_inds.push(ind);
            continue;
        }
        if world.collision_target(ind) {
            succ_inds.push(ind);
            continue;
        }
    }

    for (ind, rocket) in world.rockets.iter_mut().enumerate() {
        if dead_inds.contains(&ind) {
            if rocket.state == RocketState::Alive {
                world.alive_count -= 1;
            }
            rocket.state = RocketState::Dead;
            continue;
        }
        if succ_inds.contains(&ind) {
            rocket.state = RocketState::Successful;
            continue;
        }
        rocket.angle += rocket.dna.next_angle();

        let pos_offset = rocket.calc_offset();
        rocket.pos.x += pos_offset.x;
        rocket.pos.y += pos_offset.y;
    }
    world.frame_counter += 1;
    world.timer_rect.width -= SCREEN_WIDTH as f32 / GENE_LEN as f32;
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
            rocket_color,
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

    ctx.draw_text(
        format!("Generation {}", world.generation).as_str(),
        20,
        SCREEN_HEIGHT - 40,
        20,
        Color::RAYWHITE,
    );
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Smart Rockets")
        .msaa_4x()
        .vsync()
        .build();

    let mut world = World::new();
    let mut pause = false;
    while !rl.window_should_close() {
        // Handle input phase
        match handle_input(&rl) {
            Actions::Pause => pause = !pause,
            Actions::Reset => {
                world.restart();
                println!("Restarted")
            }
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
