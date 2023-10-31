// mod ai;

// use ai;
use ggez;
use ggez::glam::vec2;
use ggez::graphics;
use ggez::input::mouse::MouseButton;
use ggez::{event, Context};
// use ggez::input::keyboard::{KeyCode, KeyMods, KeyInput};
use std::f32::consts::PI;
// use std::time::Duration;
use mint::Point2;
// use rand::Rng;

pub const WINDOW_WIDTH: f32 = 1920.0;
pub const WINDOW_HEIGHT: f32 = 1080.0;
const BORDER: f32 = 40.0;
const FIELD_WIDTH: f32 = WINDOW_WIDTH - BORDER;
const FIELD_HEIGHT: f32 = WINDOW_HEIGHT - BORDER;
// const CIRCLE_V: f32 = 4.5;
const SIGMA: f32 = 1.0;
const HOLES_RADIUS: f32 = 36.0;
const HOLES_POINTS: [Point2<f32>; 6]  = [
    Point2{x: 0.0 + HOLES_RADIUS, y: 0.0 + HOLES_RADIUS},
    Point2{x: WINDOW_WIDTH - HOLES_RADIUS, y: 0.0 + HOLES_RADIUS}, // DO not work
    Point2{x: WINDOW_WIDTH  - HOLES_RADIUS, y: WINDOW_HEIGHT - HOLES_RADIUS},
    Point2{x: 0.0 + HOLES_RADIUS, y: WINDOW_HEIGHT - HOLES_RADIUS},
    Point2{x: WINDOW_WIDTH/2.0 , y: WINDOW_HEIGHT - HOLES_RADIUS},
    Point2{x: WINDOW_WIDTH/2.0 , y: 0.0 + HOLES_RADIUS},
];
const BALL_RADIUS: f32 = 18.0;
const DECELERATION_FACTOR: f32 = 0.98;

struct Ball {
    position: Point2<f32>,
    radius: f32,
    color: graphics::Color,
    velocity: Point2<f32>,
    number: usize,
    mass: f32,
    // momentum: f32,
}
impl Ball {
    fn new(x:f32, y:f32 ,radius: f32, color: graphics::Color, velocity: Point2<f32>, number: usize) -> Ball {
        // Calculate the position based on the radius
        let position: Point2<f32>= Point2{
            x, 
            y};
        let mass: f32 = SIGMA*PI*radius.powf(2.0);
        
        Ball {
            position,
            radius,
            color,
            velocity,
            // direction,
            number,
            mass,
            // momentum: mass*CIRCLE_V,
        }
    }
}

struct Balls{
    balls_red: Vec<Ball>,
    ball_white: Ball,
}
impl Balls {
    fn new() -> Balls {
        let mut x: f32 = 1500.0;
        let mut y: f32 = 540.0;
        let mut aux_index: usize = 0;
        let aux_array: [usize;5] = [1,3,6,10,15];
        // let mut count: usize = 0;

        let mut balls_red = Vec::new();
        for i in 1..=15{
            let ball :Ball = Ball::new(
                x, 
                y, 
                BALL_RADIUS, 
                graphics::Color::RED, 
                Point2 { 
                    x: 0.0, 
                    y: 0.0,
                },
                i);
            if i == aux_array[aux_index]{
                aux_index += 1;
                let aux_index_f32: f32 = aux_index as f32;
                x += BALL_RADIUS * 2.0;
                y = 540.0 + aux_index_f32*BALL_RADIUS;
            } else {
                y -= BALL_RADIUS*2.0;
            }
            balls_red.push(ball);
        }
        let ball_white :Ball = Ball::new(
            200.0, 
            540.0, 
            BALL_RADIUS, 
            graphics::Color::WHITE, 
            Point2 { 
                x: 0.0, 
                y: 0.0,
            },
            0);

        Balls { balls_red, ball_white }
    }
}

struct Hole{
    position: Point2<f32>,
    radius: f32,
}
struct Holes{
    holes: Vec<Hole>,
}
impl Holes {
    fn new () -> Holes{
        let mut holes:Vec<Hole> = Vec::new();
        for i in 0..HOLES_POINTS.len(){
            holes.push(Hole { 
                position: HOLES_POINTS[i], 
                radius: HOLES_RADIUS 
            })
        }
        Holes{holes}
    }
}

struct Player{
    points: i32,
}

pub struct MainState {
    balls: Balls,
    holes: Holes,
    player_1: Player,
    player_2: Player,
    turn: usize,
    player_scores: bool,
}

impl MainState {
    pub fn new() -> Self {
        // let (screen_w, screen_h) = (FIELD_WIDTH, FIELD_HEIGHT);
  
        let balls = Balls::new();
        let holes = Holes::new();
        let player1 = Player{
            points: 0,
        };
        let player2 = Player { 
            points: 0,
        };

        let state: MainState =  MainState { 
            balls: balls,
            holes: holes,
            player_1: player1,
            player_2: player2,
            turn: 1,
            player_scores: true,
        };
        state
    }
    fn score(&mut self) {
        if self.turn == 1 {
            self.player_1.points += 1; // Player 1 scores
        } else {
            self.player_2.points += 1; // Player 2 scores
        }
        self.player_scores = true; // Set the flag to indicate scoring
    }
}

// Functions
fn clamp(ball: &mut Ball) {
    let (min_x, min_y, max_x, max_y) = (
        ball.radius + BORDER,
        ball.radius + BORDER,
        FIELD_WIDTH - ball.radius,
        FIELD_HEIGHT - ball.radius,
    );

    if ball.position.x < min_x {
        ball.position.x = min_x;
        ball.velocity.x *= -1.0;
    } else if ball.position.x > max_x {
        ball.position.x = max_x;
        ball.velocity.x *= -1.0;
    }

    if ball.position.y < min_y {
        ball.position.y = min_y;
        ball.velocity.y *= -1.0;
    } else if ball.position.y > max_y {
        ball.position.y = max_y;
        ball.velocity.y *= -1.0;
    }
}

fn collision(balls: &Balls) -> (bool, Vec<(usize, usize)>) {
    let mut collided_pairs: Vec<(usize, usize)> = Vec::new();
    let mut collision_detected = false;

    let white_ball = &balls.ball_white;
    let red_balls = &balls.balls_red;


    for i in 0..red_balls.len() {
        let ball = &red_balls[i];
        let dx = white_ball.position.x - ball.position.x;
        let dy = white_ball.position.y - ball.position.y;
        let distance_squared = dx * dx + dy * dy;
        let min_distance = white_ball.radius + ball.radius;
        if distance_squared < min_distance * min_distance {
            // Add the white ball index and the red ball index to the list.
            // Assuming that white ball has index -1 (you can choose a unique index)
            collided_pairs.push((69, i)); // Collision detected with white ball.
            collision_detected = true;
        }
    }

    for i in 0..red_balls.len() - 1 {
        for j in i + 1..red_balls.len() {
            let ball1 = &red_balls[i];
            let ball2 = &red_balls[j];
            let dx = ball1.position.x - ball2.position.x;
            let dy = ball1.position.y - ball2.position.y;
            let distance_squared = dx * dx + dy * dy;
            let min_distance = ball1.radius + ball2.radius;
            if distance_squared < min_distance * min_distance {
                collided_pairs.push((i, j)); // Collision detected, add the pair to the list.
                collision_detected = true;
            }
        }
    }
    (collision_detected, collided_pairs)
}

fn momentum_conservation(collided_balls: &[(usize, usize)], balls: &mut Balls) {
    let red_balls = &mut balls.balls_red;
    let white_ball = &mut balls.ball_white;
    for i in 0..collided_balls.len() {
        let (index1, index2) = collided_balls[i];
        if index1 != 69{
            // Calculate the new velocities separately for x and y components.
            let delta_x = red_balls[index2].position.x - red_balls[index1].position.x;
            let delta_y = red_balls[index2].position.y - red_balls[index1].position.y;
            let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

            let normal_x = delta_x / distance;
            let normal_y = delta_y / distance;

            let relative_velocity_x = red_balls[index2].velocity.x - red_balls[index1].velocity.x;
            let relative_velocity_y = red_balls[index2].velocity.y - red_balls[index1].velocity.y;

            let dot_product = normal_x * relative_velocity_x + normal_y * relative_velocity_y;

            if dot_product < 0.0 {
                let impulse = (2.0 * dot_product)
                    / (1.0 / red_balls[index1].mass + 1.0 / red_balls[index2].mass);
                let impulse_x = impulse * normal_x;
                let impulse_y = impulse * normal_y;

                // Update velocities for both balls.
                red_balls[index1].velocity.x += impulse_x / red_balls[index1].mass;
                red_balls[index1].velocity.y += impulse_y / red_balls[index1].mass;
                red_balls[index2].velocity.x -= impulse_x / red_balls[index2].mass;
                red_balls[index2].velocity.y -= impulse_y / red_balls[index2].mass;
            }
        }else {
            // Calculate the new velocities separately for x and y components.
            let delta_x = red_balls[index2].position.x - white_ball.position.x;
            let delta_y = red_balls[index2].position.y - white_ball.position.y;
            let distance = (delta_x * delta_x + delta_y * delta_y).sqrt();

            let normal_x = delta_x / distance;
            let normal_y = delta_y / distance;

            let relative_velocity_x = red_balls[index2].velocity.x - white_ball.velocity.x;
            let relative_velocity_y = red_balls[index2].velocity.y - white_ball.velocity.y;

            let dot_product = normal_x * relative_velocity_x + normal_y * relative_velocity_y;

            if dot_product < 0.0 {
                let impulse = (2.0 * dot_product)
                    / (1.0 / white_ball.mass + 1.0 / red_balls[index2].mass);
                let impulse_x = impulse * normal_x;
                let impulse_y = impulse * normal_y;

                // Update velocities for both balls.
                white_ball.velocity.x += impulse_x / white_ball.mass;
                white_ball.velocity.y += impulse_y / white_ball.mass;
                red_balls[index2].velocity.x -= impulse_x / red_balls[index2].mass;
                red_balls[index2].velocity.y -= impulse_y / red_balls[index2].mass;
            }
        }
    }
} 

fn pool_movement(ctx: &Context, white_ball: &Ball) -> (f32, f32) {
    let max_distance: f32 = 40.0;

    let mouse_position = ctx.mouse.position();
    let dx: f32 = mouse_position.x - white_ball.position.x;
    let dy: f32 = mouse_position.y - white_ball.position.y;
    let distance: f32 = (dx * dx + dy * dy).sqrt();
    
    let power: f32 = f32::min(max_distance, distance);
    let direc_x: f32 = if distance > 0.0 { dx / distance } else { 0.0 };
    let direc_y: f32 = if distance > 0.0 { dy / distance } else { 0.0 };

    // Adjust the factor (e.g., 50.0) to control the velocity magnitude
    let velocity_x = -power * direc_x;
    let velocity_y = -power * direc_y;

    (velocity_x * 1.1, velocity_y* 1.1)
}

fn in_hole(holes: &Holes, balls: &mut Balls) -> (Vec<usize>, bool){
    let red_balls: &Vec<Ball> = &balls.balls_red;
    let white_ball: &Ball = &balls.ball_white;
    let holes: &Vec<Hole> = &holes.holes;
    let mut balls_in_the_hole: Vec<usize> = Vec::new();
    let mut ball_in_detected: bool = false;

    for j in 0..holes.len(){
        let hole: &Hole = &holes[j];
        let dx: f32 = white_ball.position.x - hole.position.x;
        let dy: f32 = white_ball.position.y - hole.position.y;
        let distance_squared: f32 = dx * dx + dy * dy;
        let min_distance: f32 = white_ball.radius + hole.radius;
        if distance_squared < min_distance * min_distance {
            balls_in_the_hole.push(69); // Collision detected, add the pair to the list.
            ball_in_detected = true;
        }
    }

    for i in 0..red_balls.len() {
        for j in 0..holes.len()  {
            let ball: &Ball = &red_balls[i];
            let hole: &Hole = &holes[j];
            let dx: f32 = ball.position.x - hole.position.x;
            let dy: f32 = ball.position.y - hole.position.y;
            let distance_squared: f32 = dx * dx + dy * dy;
            let min_distance: f32 = ball.radius + hole.radius;
            if distance_squared < min_distance * min_distance {
                balls_in_the_hole.push(i); // Collision detected, add the pair to the list.
                ball_in_detected = true;
            }
        }
    }
    (balls_in_the_hole, ball_in_detected)
}


impl event::EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {

        // let (white_power_x, white_power_y) = pool_movement(ctx, &self.balls.ball_white);
        // println!("{white_power}");
        let m_ctx: &ggez::input::mouse::MouseContext = &ctx.mouse;
        let mut ready_falg : bool = false;
        
        if self.balls.ball_white.velocity.x.abs() < 0.05 
            && self.balls.ball_white.velocity.y.abs() < 0.05
            && !ready_falg{
            ready_falg = true;
            self.balls.ball_white.velocity.x = 0.0;
            self.balls.ball_white.velocity.y = 0.0;
            self.balls.ball_white.color = ggez::graphics::Color::WHITE;
        }
        
        if m_ctx.button_just_pressed(MouseButton::Left) && ready_falg {
            self.player_scores = false;
            self.balls.ball_white.color = ggez::graphics::Color::from_rgb(183, 183, 183);
            let (white_power_x, white_power_y) = pool_movement(ctx, &self.balls.ball_white);
            self.balls.ball_white.velocity.x = white_power_x;
            self.balls.ball_white.velocity.y = white_power_y;


        }

        clamp(&mut self.balls.ball_white);
        // Apply deceleration to the white ball
        self.balls.ball_white.velocity.x *= DECELERATION_FACTOR;
        self.balls.ball_white.velocity.y *= DECELERATION_FACTOR;
        self.balls.ball_white.position.x += self.balls.ball_white.velocity.x;
        self.balls.ball_white.position.y += self.balls.ball_white.velocity.y;

        for ball in &mut self.balls.balls_red{
            
            ball.velocity.x *= DECELERATION_FACTOR;
            ball.velocity.y *= DECELERATION_FACTOR;
            ball.position.x += ball.velocity.x;
            ball.position.y += ball.velocity.y;
            clamp(ball)
        }

        // Check for colisions
        let (collision_bool, collided_balls) = collision(&mut self.balls);
        if collision_bool{
            momentum_conservation(&collided_balls, &mut self.balls);
        }
        
        // Check to see if the ball its the hole
        let (balls_in_the_hole, ball_hole_flag) = in_hole(&mut self.holes, &mut self.balls);
        if ball_hole_flag{
            for index in balls_in_the_hole{
                if index != 69{
                    self.balls.balls_red.remove(index);
                    self.player_scores = true;
                    self.score();
                }else {
                    self.balls.ball_white.position = Point2{
                        x: 200.0, 
                        y: 540.0
                    };
                    self.balls.ball_white.velocity = Point2{
                        x:0.0,
                        y:0.0
                    };
                }
            }
        }

        if self.balls.ball_white.color == ggez::graphics::Color::WHITE && !self.player_scores{
            self.player_scores = true;
            println!("Branco");
            if self.turn == 1 {
                self.turn = 2;
            }else{self.turn = 1}
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> Result<(), ggez::GameError> {

        let mut canvas = graphics::Canvas::from_frame(ctx, graphics::Color::GREEN);
        
        // DRAW BORDER
        let border = graphics::Rect::new(
            0.0, 
            0.0, 
            WINDOW_WIDTH, 
            WINDOW_HEIGHT,
        );
        
        let rect = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::stroke(BORDER*2.0),
            border,
            graphics::Color::from_rgb(165, 42, 42),
        )?;
        canvas.draw(&rect, graphics::DrawParam::default());

        // DRAW HOLES
        for i in 0..HOLES_POINTS.len(){
            let ball_mesh: graphics::Mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                HOLES_POINTS[i],
                HOLES_RADIUS,
                0.1,
                graphics::Color::BLACK,
            )?;
            canvas.draw(&ball_mesh, graphics::DrawParam::default());
        }

        // DRAW BALLS
        for ball in &mut self.balls.balls_red{
            let ball_mesh: graphics::Mesh = graphics::Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                ball.position,
                ball.radius,
                0.1,
                ball.color,
            )?;
            canvas.draw(&ball_mesh, graphics::DrawParam::default());

            let mut text = graphics::Text::new(format!("{}", ball.number));
            // let mut number_position: Point2<f32> = Point2 { x: 0.0, y: 0.0 };

            let number_position: Point2<f32> = if ball.number < 10 {
                Point2 { 
                    x: ball.position.x - ball.radius/3.0, 
                    y: ball.position.y - ball.radius/2.0, 
                }
            } else {
                Point2 { 
                    x: ball.position.x - ball.radius/1.7, 
                    y: ball.position.y - ball.radius/2.0, 
                }
            };

            let text_dest = graphics::DrawParam::new()
                .dest(number_position)
                .color(graphics::Color::WHITE);
            canvas.draw(
                text
                    .set_scale(20.)
                    .set_bounds(vec2(16.0, 16.0)),
                    text_dest,
            );
        }

        let ball_mesh_white: graphics::Mesh = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            self.balls.ball_white.position,
            self.balls.ball_white.radius,
            0.1,
            self.balls.ball_white.color,
        )?;
        canvas.draw(&ball_mesh_white, graphics::DrawParam::default());

        let mut player1_points_txt = graphics::Text::new(format!("Player 1 Points: {}", self.player_1.points));
        let player1_points = graphics::DrawParam::new()
        .dest(Point2{
            x:270.0,
            y:0.0,
        })
        .color(graphics::Color::WHITE);

        canvas.draw(
        player1_points_txt
            .set_scale(40.),
            player1_points,
        );


        let mut player2_points_txt = graphics::Text::new(format!("Player 2 Points: {}", self.player_2.points));
        let player2_points = graphics::DrawParam::new()
        .dest(Point2{
            x:1270.0,
            y:0.0,
        })
        .color(graphics::Color::WHITE);

        canvas.draw(
        player2_points_txt
            .set_scale(40.),
            player2_points,
        );

        let mut player_turn_txt = graphics::Text::new(format!("Player {} Turn", self.turn));
        let player_turn = graphics::DrawParam::new()
        .dest(Point2{
            x:300.0,
            y:1040.0,
        })
        .color(graphics::Color::WHITE);

        canvas.draw(
            player_turn_txt
            .set_scale(40.),
            player_turn,
        );
        canvas.finish(ctx)?;
        Ok(())
    }
}



// fn main() -> GameResult {
//     let (ctx, event_loop) = ggez::ContextBuilder::new("Rusty Pong", "M@ano")
//         .window_setup(conf::WindowSetup::default().title("Rusty Pong"))
//         .window_mode(conf::WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
//         .build()
//         .unwrap();

//     let state = MainState::new();
//     event::run(ctx, event_loop, state);
//     // Ok(())
// }
