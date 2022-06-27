use crate::lib::kinput::*;
use crate::lib::kmath::*;
use crate::krenderer::*;

use glutin::event::VirtualKeyCode;

pub struct Game {
    enemy_positions: Vec<Vec2>,
    enemy_vels: Vec<Vec2>,

    is_clear: bool,

    is_player: bool,
    player_pos: Vec2,
    player_vel: Vec2,
    player_health: f32,

    is_fireball: bool,
    fireball_pos: Vec2,
    fireball_vel: Vec2,

    explosion_end: f64,

    t: f64,
    t_next_spawn: f64,

    difficulty_level: i32,
}

impl Game {
    pub fn new() -> Game {
        Game {
            enemy_positions: Vec::new(),
            enemy_vels: Vec::new(),

            is_player: true,
            is_clear: true,
            player_pos: Vec2::new(0.0, 0.0),
            player_vel: Vec2::new(0.0, 0.0),
            player_health: 1.0,

            fireball_pos: Vec2::new(0.0, 0.0),
            fireball_vel: Vec2::new(0.0, 0.0),
            is_fireball: false,

            explosion_end: -0.2,

            t: 0.0,
            t_next_spawn: 0.0,

            difficulty_level: 0,
        }
    }

    pub fn frame(&mut self, inputs: &FrameInputState, kc: &mut KRCanvas) {
        self.t += inputs.dt;

        let player_speed = 0.55;
        let enemy_speed = 0.45;
        let enemy_steer_amount = 9.0;

        let spawn_interval = 0.1;
        let spawn_chance = 0.02 + 0.01 * self.difficulty_level as f32;
        let player_radius = 0.05;
        let enemy_radius = 0.025;
        let fireball_radius = 0.025;
        let fireball_explode_radius = 0.35;
        let fireball_self_damage = 0.4;
        let enemy_damage = 0.2;
        let game_rect = Rect::new(-1.0, -1.0, 2.0, 2.0);


        let fgs = vec![
            Vec4::new(0.9, 0.0, 0.0, 1.0),
            Vec4::new(0.0, 0.9, 0.0, 1.0),
            Vec4::new(0.0, 0.0, 0.9, 1.0),
            Vec4::new(0.9, 0.9, 0.0, 1.0),
            Vec4::new(0.0, 0.9, 0.9, 1.0),
            Vec4::new(0.9, 0.0, 0.9, 1.0),
        ];
        let bgs = vec![
            Vec4::new(1.0, 0.9, 0.8, 1.0),
            Vec4::new(0.9, 1.0, 0.8, 1.0),
            Vec4::new(0.8, 0.9, 1.0, 1.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        ];

        let foreground_max = fgs[self.difficulty_level as usize % fgs.len()];
        let foreground_min = Vec4::new(0.0, 0.0, 0.0, 1.0);
        let background_max = bgs[self.difficulty_level as usize % bgs.len()];
        let background_min = Vec4::new(0.0, 0.0, 0.0, 1.0);

        let lerp_t = self.player_health.max(0.0);
        let lerp_t = lerp_t * lerp_t;

        let fg = foreground_min.lerp(foreground_max, lerp_t);
        let bg = background_min.lerp(background_max, lerp_t);
     
        // spawn
        if self.t > self.t_next_spawn {
            self.t_next_spawn += spawn_interval;
            let spawn_points = [
                Vec2::new(-1.0, 0.0),
                Vec2::new(1.0, 0.0),
                Vec2::new(0.0, 1.0),
                Vec2::new(0.0, -1.0),
            ];
            for (i, sp) in spawn_points.iter().enumerate() {
                if chance(inputs.seed + i as u32, spawn_chance) {
                    self.enemy_positions.push(*sp);
                    self.enemy_vels.push(Vec2::new(0.0, 0.0));
                    self.is_clear = false;
                }
            }
        }

        let old_enemy_positions = self.enemy_positions.clone();
        let old_player_pos = self.player_pos;

        if inputs.just_pressed(VirtualKeyCode::Escape) {
            std::process::exit(0);
        }

        if inputs.just_pressed(VirtualKeyCode::R) && self.is_player == false {
            *self = Game::new();
        }

        let mut dead_enemies: Vec<usize> = Vec::new();

        let walls = vec![
            Rect::new(-1.0, -1.0, 0.9, 0.1),
            Rect::new(0.1, -1.0, 0.9, 0.1),
            Rect::new(-1.0, 0.9, 0.9, 0.1),
            Rect::new(0.1, 0.9, 0.9, 0.1),
            
            Rect::new(-1.0, -1.0, 0.1, 0.9),
            Rect::new(0.9, -1.0, 0.1, 0.9),
            Rect::new(-1.0, 0.1, 0.1, 0.9),
            Rect::new(0.9, 0.1, 0.1, 0.9),
        ];

        if self.is_player && self.is_fireball == false && self.t > self.explosion_end && inputs.lmb == KeyStatus::JustPressed {
            // spawn fireball
            let mouse_world = inputs.mouse_pos.transform(inputs.screen_rect, game_rect);
            self.is_fireball = true;
            self.fireball_pos = self.player_pos;
            self.fireball_vel = (mouse_world - self.player_pos).normalize() * 2.0;
        }
        
        let player_vel = {
            // let mut player_vel = Vec2::new(0.0, 0.0);
            let mut player_vel = self.player_vel;
            if inputs.pressed(VirtualKeyCode::W) {
                player_vel.y = (player_vel.y - 1.0).max(-1.0);
            }
            if inputs.pressed(VirtualKeyCode::S) {
                player_vel.y = (player_vel.y + 1.0).min(1.0);
            }
            if inputs.pressed(VirtualKeyCode::A) {
                player_vel.x = (player_vel.x -  1.0).max(-1.0);
            }
            if inputs.pressed(VirtualKeyCode::D) {
                player_vel.x = (player_vel.x + 1.0).min(1.0);
            }
            player_vel.normalize();
            player_vel * player_speed
        };
        self.player_vel = player_vel;

        self.player_pos = self.player_pos + player_vel * inputs.dt as f32;

        if self.is_player {
            self.player_health = (self.player_health + 0.1 * inputs.dt as f32).min(1.0);
        }

        if self.is_fireball {
            self.fireball_pos = self.fireball_pos + self.fireball_vel * inputs.dt as f32; 
        }

        // fireball explodes
        let should_explode = self.is_fireball && (
            self.enemy_positions.iter()
            .filter(|p| (**p - self.fireball_pos).magnitude() < enemy_radius + fireball_radius)
            .nth(0)
            .is_some() ||
            walls.iter().filter(|w| w.dilate(fireball_radius).contains(self.fireball_pos))
            .nth(0)
            .is_some());

        if should_explode {
            for i in 0..self.enemy_positions.len() {
                if (self.enemy_positions[i] - self.fireball_pos).magnitude() < enemy_radius + fireball_explode_radius {
                    dead_enemies.push(i);
                }
            }
            if (self.player_pos - self.fireball_pos).magnitude() < fireball_explode_radius {
                self.player_health -= fireball_self_damage;
            }
            self.is_fireball = false;
            self.explosion_end = self.t + 0.05;
        }
        
        // enemy steering
        for i in 0..self.enemy_positions.len() {
            let current_dir = self.enemy_vels[i].normalize();
            let steer_dir = (self.player_pos - self.enemy_positions[i]).normalize();
            let new_dir = current_dir.lerp(steer_dir, enemy_steer_amount * inputs.dt as f32).normalize();
            self.enemy_vels[i] = new_dir * enemy_speed;
        }

        // enemy movement
        for i in 0..self.enemy_positions.len() {
            self.enemy_positions[i] = self.enemy_positions[i] + self.enemy_vels[i] * inputs.dt as f32;
        }

        // calculate enemy collisions
        let mut enemy_collisions:Vec<(usize, usize, Vec2)> = Vec::new();
        for i in 0..self.enemy_positions.len() {
            for j in 0..self.enemy_positions.len() {
                if i == j {continue};
                let penetration = 2.0 * enemy_radius - (self.enemy_positions[i] - self.enemy_positions[j]).magnitude();
                if penetration > 0.0 {
                    let pvec = penetration *  (self.enemy_positions[i] - self.enemy_positions[j]).normalize();
                    enemy_collisions.push((i, j, pvec));                    
                }
            }
        }

        // apply enemy collisions
        for (subject, object, pen) in enemy_collisions {
            self.enemy_positions[subject] = self.enemy_positions[subject] + 0.5 * pen;
        }

        // player collides with walls
        // how to get axis of penetration from rect-circle collision?
        for wall in walls.iter() {
            let closest_point = wall.snap(self.player_pos);
            let penetration = player_radius - (closest_point - self.player_pos).magnitude();
            if penetration > 0.0 {
                let pen_vec = penetration * (closest_point - self.player_pos).normalize();
                self.player_pos = self.player_pos - pen_vec;
            }
        }

        // enemies collide with walls
        for wall in walls.iter() {
            for i in 0..self.enemy_positions.len() {
                let closest_point = wall.snap(self.enemy_positions[i]);
                let penetration = enemy_radius - (closest_point - self.enemy_positions[i]).magnitude();
                if penetration > 0.0 {
                    let pen_vec = penetration * (closest_point - self.enemy_positions[i]).normalize();
                    self.enemy_positions[i] = self.enemy_positions[i] - pen_vec;
                }
            }
        }

        // player collides with enemies (and takes damage)
        let mut enemy_col_with_player: Vec<(usize, Vec2)> = Vec::new();
        for i in 0..self.enemy_positions.len() {
            if player_radius + enemy_radius > (self.enemy_positions[i] - self.player_pos).magnitude() {
                enemy_col_with_player.push((i, (self.enemy_positions[i] - self.player_pos)))
            }
        }

        for (enemy_id, pen) in enemy_col_with_player {
            self.player_pos = self.player_pos - 0.5 * pen;
            self.enemy_positions[enemy_id] = self.enemy_positions[enemy_id] + 0.5 * pen;
            self.player_health -= enemy_damage;
        }

        // player dies
        if self.player_health <= 0.0 {
            self.is_player = false;
            println!("rip, press r to reset");
        }

        // velocity fix
        for i in 0..self.enemy_positions.len() {
            self.enemy_vels[i] = (self.enemy_positions[i] - old_enemy_positions[i]) / inputs.dt as f32;
        }
        self.player_vel = (self.player_pos - old_player_pos) / inputs.dt as f32;

        // kill dead enemies
        for de in dead_enemies.iter().rev() {
            self.enemy_positions.swap_remove(*de);
            self.enemy_vels.swap_remove(*de);
        }

        if !self.is_clear && self.enemy_positions.len() == 0 {
            self.is_clear = true;
            self.difficulty_level += 1;
        }

        // clamp player to arena
        self.player_pos = game_rect.snap(self.player_pos);

        // kill fireball if it leaves arena
        if !game_rect.contains(self.fireball_pos) {
            self.is_fireball = false;
        }

        kc.set_colour(bg);
        kc.set_depth(1.0);
        kc.rect(game_rect);

        kc.set_colour(fg);
        kc.set_depth(2.0);
        for wall in walls {
            kc.rect(wall);
        }
        
        if self.t < self.explosion_end {
            kc.circle(self.fireball_pos, fireball_explode_radius);
        }
        for i in 0..self.enemy_positions.len() {
            kc.circle(self.enemy_positions[i], enemy_radius);
        }
        if self.is_player {
            kc.circle(self.player_pos, player_radius);
        }
        if self.is_fireball {
            kc.circle(self.fireball_pos, fireball_radius);
        }
    }
}