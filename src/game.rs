use std::f32::consts::PI;

use crate::lib::kinput::*;
use crate::lib::kmath::*;
use crate::krenderer::*;

use glutin::event::VirtualKeyCode;

pub struct Game {
    init: bool,

    world_seed: u32,

    zoom_level: f32,

    player_pos: Vec2,
    player_vel: Vec2,
    player_hp: f32,
    player_scrap: i32,
    player_next_t: f32,
    player_draw_arc_t: f32,

    player_place_building_kind: Option<i32>,

    projectile_pos: Vec<Vec2>,
    projectile_vel: Vec<Vec2>,
    projectile_kind: Vec<i32>,

    scrap_pos: Vec<Vec2>,
    scrap_vel: Vec<Vec2>,

    enemy_hp: Vec<f32>,
    enemy_vel: Vec<Vec2>,
    enemy_pos: Vec<Vec2>,
    enemy_scrap: Vec<i32>, // coordinators can be big ones, maybe scrap spawns randomly? but coordinators at night good
                        // maybe enemies die naturally causing random scrap. solar radiation
                        // good to have closed form solutions, if we chunk we can load in

    building_pos: Vec<(i32, i32)>,
    building_hp: Vec<f32>,
    building_next_t: Vec<f32>,  // cooldown to shoot, or next generated scrap
    building_kind: Vec<i32>,

    pub paused: bool,
    t: f32,
}

// todo text rendering

impl Game {
    pub fn new() -> Game {
        Game {
            init: true,
            world_seed: 0,

            zoom_level: 1.0,

            player_pos: Vec2::new(0.0, 0.0),
            player_vel: Vec2::new(0.0, 0.0),
            player_hp: 1.0,
            player_scrap: 0,
            player_next_t: 0.0,
            player_draw_arc_t: 0.0,

            player_place_building_kind: None,

            projectile_pos: Vec::new(),
            projectile_vel: Vec::new(),
            projectile_kind: Vec::new(),

            scrap_pos: vec![Vec2::new(1.0, 1.0), Vec2::new(1.5, 1.5)],
            scrap_vel: vec![Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0)],

            enemy_hp: Vec::new(),
            enemy_vel: Vec::new(),
            enemy_pos: Vec::new(),
            enemy_scrap: Vec::new(),

            building_pos: Vec::new(),
            building_hp: Vec::new(),
            building_next_t: Vec::new(),
            building_kind: Vec::new(),

            t: 0.0,
            paused: false,
        }
    }

    pub fn frame(&mut self, inputs: &FrameInputState, kc: &mut KRCanvas) {

        if inputs.scroll_up {
            self.zoom_level /= 1.1;
        } else if inputs.scroll_down {
            self.zoom_level *= 1.1;
        }
        self.zoom_level = self.zoom_level.min(5.0).max(0.4);

        let cam_w = self.zoom_level * inputs.screen_rect.w/inputs.screen_rect.h;
        let cam_h = self.zoom_level;
        let camera_rect = Rect::new_centered(self.player_pos.x, self.player_pos.y, cam_w, cam_h);
        let mouse_pos = inputs.mouse_pos.transform(inputs.screen_rect, camera_rect);
        let new_camera_center = camera_rect.centroid().lerp(mouse_pos, 0.2);
        let camera_rect = Rect::new_centered(new_camera_center.x, new_camera_center.y, cam_w, cam_h);
        let mouse_pos = inputs.mouse_pos.transform(inputs.screen_rect, camera_rect);
        
        let player_speed = 1.0;
        let player_radius = 0.06;
        let player_melee_radius = 0.3;
        let player_melee_arc = PI/4.;
        let arc_duration = 0.06;
        let player_melee_cooldown = 0.5;
        let player_melee_damage = 0.5;
        let player_suck_pickup_radius = 0.5;
        let suck_force = 10.0;

        let scrap_radius = 0.02;

        let enemy_radius = 0.03;
        let enemy_radius_per_scrap = 0.02;
        let enemy_steer_amount = 5.0;
        let enemy_speed = 0.4;

        let building_s = 0.2;

        
        
        self.t += inputs.dt as f32;
        
        // 1.0 / -1.0 is nightfall
        // 0.0 is dawn of a new day
        let day_t = ((self.t / 200.0) % 1.0) * 2.0 - 1.0;

        let day = day_t > 0.0;
        let enemy_count = if day {
            200
        } else {
            400
        };

        let enemy_acquisition_range = if day {
            1.0
        } else {
            1.0
        };


        // building
        if inputs.just_pressed(VirtualKeyCode::Q) {
            if self.player_place_building_kind == Some(0) {
                self.player_place_building_kind = None;
            } else {
                self.player_place_building_kind = Some(0);
            }
        }


        let player_steer = {
            let mut steer = Vec2::new(0.0, 0.0);
            if inputs.pressed(VirtualKeyCode::W) {
                steer.y = (steer.y - 1.0).max(-1.0);
            }
            if inputs.pressed(VirtualKeyCode::S) {
                steer.y = (steer.y + 1.0).min(1.0);
            }
            if inputs.pressed(VirtualKeyCode::A) {
                steer.x = (steer.x -  1.0).max(-1.0);
            }
            if inputs.pressed(VirtualKeyCode::D) {
                steer.x = (steer.x + 1.0).min(1.0);
            }
            steer.normalize()
        };
        self.player_vel = 0.5 * self.player_vel;

        let frame_vmag = (player_speed * player_steer + self.player_vel).magnitude();
        let frame_vdir = (player_speed * player_steer + self.player_vel).normalize();
        let frame_v = frame_vmag.min(player_speed) * frame_vdir;
        self.player_pos = self.player_pos + frame_v * inputs.dt as f32;


        // move scrap towards player
        for (i, p) in self.scrap_pos.iter().enumerate() {
            let vsp = self.player_pos - *p;
            if vsp.magnitude() < (player_radius + scrap_radius + player_suck_pickup_radius) {
                self.scrap_vel[i] = suck_force * inputs.dt as f32 * vsp.normalize() / vsp.magnitude();
            } else {
                self.scrap_vel[i] = Vec2::new(0.0, 0.0);
            }
        }
        
        for i in 0..self.scrap_pos.len() {
            self.scrap_pos[i] = self.scrap_pos[i] + self.scrap_vel[i] * inputs.dt as f32;
        }

        let mut dead_scrap = Vec::new(); // is this the best way to remove stuff from an array lol.
        // could do a while loop and mutate it manually i guess and make sure to bail approriately
        // because we might double spend if player and entities both pick up the scrap

        // player picks up scrap
        for (i, p) in self.scrap_pos.iter().enumerate() {
            if (self.player_pos - *p).magnitude() < (player_radius + scrap_radius) {
                self.player_scrap += 1;
                dead_scrap.push(i);
            }
        }

        for idx in dead_scrap.iter().rev() {
            self.scrap_pos.swap_remove(*idx);
            self.scrap_vel.swap_remove(*idx);
        }

        // cull enemies
        let mut idx = self.enemy_pos.len() as i32 - 1;
        while idx > 0 {
            if self.enemy_pos[idx as usize].dist(self.player_pos) > 4.05 {
                self.enemy_pos.swap_remove(idx as usize);
                self.enemy_hp.swap_remove(idx as usize);
                self.enemy_scrap.swap_remove(idx as usize);
                self.enemy_vel.swap_remove(idx as usize);
            }
            idx -= 1;
        }

        // spawn enemies
        println!("there  are {} enemies", self.enemy_pos.len());
        let mut seed = inputs.seed;

        while self.enemy_pos.len() < enemy_count {
            self.enemy_pos.push(self.player_pos.offset_r_theta(4.0, kuniform(seed, 0., 2. * PI)));
            self.enemy_hp.push(1.0);
            self.enemy_scrap.push(0);
            self.enemy_vel.push(Vec2::new(0.0, 0.0));
            seed = khash(seed);
        }

        let old_enemy_positions = self.enemy_pos.clone();

        
        // enemy steering
        // yea this will need some work, will = 0 it will still keep its velocity
        // have a think about enemy behaviour
        for i in 0..self.enemy_pos.len() {
            let current_dir = self.enemy_vel[i].normalize();
            let steer_dir = if self.player_pos.dist(self.enemy_pos[i]) > enemy_acquisition_range {
                Vec2::new(0.0, 0.0)
            } else {
                (self.player_pos - self.enemy_pos[i]).normalize()
            };
            let new_dir = current_dir.lerp(steer_dir, enemy_steer_amount * inputs.dt as f32).normalize();
            self.enemy_vel[i] = new_dir * enemy_speed;
        }

        // enemy movement
        for i in 0..self.enemy_pos.len() {
            self.enemy_pos[i] = self.enemy_pos[i] + self.enemy_vel[i] * inputs.dt as f32;
        }

        // calculate enemy collisions
        let mut enemy_collisions:Vec<(usize, usize, Vec2)> = Vec::new();
        for i in 0..self.enemy_pos.len() {
            for j in 0..self.enemy_pos.len() {
                if i == j {continue};
                let penetration = 2.0 * enemy_radius - (self.enemy_pos[i] - self.enemy_pos[j]).magnitude();
                if penetration > 0.0 {
                    let pvec = penetration *  (self.enemy_pos[i] - self.enemy_pos[j]).normalize();
                    enemy_collisions.push((i, j, pvec));                    
                }
            }
        }

        // apply enemy collisions
        for (subject, object, pen) in enemy_collisions {
            self.enemy_pos[subject] = self.enemy_pos[subject] + 0.5 * pen;
        }


        // and spawn a certain distance and roam
        // enemies: i guess cull a certain distamce from player
        // and drop scrap

        // and player needs weapons 

        // player melee enemies
        let melee_attack = inputs.lmb == KeyStatus::Pressed && self.player_place_building_kind == None && self.t > self.player_next_t;

        if melee_attack {
            self.player_next_t = self.t + player_melee_cooldown;
            self.player_draw_arc_t = self.t + arc_duration;
            for idx in 0..self.enemy_pos.len() {
                // circle collision and satisfy dot product for angle
                let v_aim = mouse_pos - self.player_pos;
                let v_enemy = self.enemy_pos[idx] - self.player_pos;
                let theta = v_aim.normalize().dot(v_enemy.normalize()).acos();

                if self.player_pos.dist(self.enemy_pos[idx]) < (player_melee_radius + enemy_radius) && theta.abs() < player_melee_arc {
                    self.enemy_hp[idx] -= player_melee_damage;
                    self.enemy_pos[idx] = self.enemy_pos[idx] + v_enemy.normalize() * 0.03;
                }
            }
        }

        if inputs.lmb == KeyStatus::Pressed && self.player_place_building_kind.is_some() {
            let i = (mouse_pos.x / building_s).floor() as i32;
            let j = (mouse_pos.y / building_s).floor() as i32;
            let bk = self.player_place_building_kind.unwrap();
            if bk == 0 {
                // wall
                if self.player_scrap >= 3 { // check if already occupied too
                    self.player_scrap -= 3;
                    self.building_pos.push((i, j));
                    self.building_kind.push(bk);
                    self.building_hp.push(4.0);
                    self.building_next_t.push(0.);
                }
            }
        }

        // flesh out all the collision types

        // kill enemies with < 0 hp and drop pickups
        let mut idx = self.enemy_pos.len() - 1;
        while idx > 0 {
            if self.enemy_hp[idx] <= 0.0 {
                self.scrap_pos.push(self.enemy_pos[idx]);
                self.scrap_vel.push(Vec2::new(0.0, 0.0));

                self.enemy_pos.swap_remove(idx);
                self.enemy_hp.swap_remove(idx);
                self.enemy_scrap.swap_remove(idx);
                self.enemy_vel.swap_remove(idx);
            }
            idx -= 1;
        }

        // enemies collide with walls
        for i in 0..self.building_pos.len() {
            let building_rect = Rect::new(self.building_pos[i].0 as f32 * building_s, self.building_pos[i].1 as f32 * building_s, building_s, building_s);
            for j in 0..self.enemy_pos.len() {
                let closest_point = building_rect.snap(self.enemy_pos[j]);
                let penetration = enemy_radius - (closest_point - self.enemy_pos[j]).magnitude();
                if penetration > 0.0 {
                    let pen_vec = penetration * (closest_point - self.enemy_pos[j]).normalize();
                    self.enemy_pos[j] = self.enemy_pos[j] - pen_vec;
                }
            }
        }

        // velocity fix
        for i in 0..self.enemy_pos.len() {
            self.enemy_vel[i] = (self.enemy_pos[i] - old_enemy_positions[i]) / inputs.dt as f32;
        }


        kc.set_colour(Vec4::new(0.2, 0.6, 0.2, 1.0));
        kc.set_depth(1.0);
        kc.rect(inputs.screen_rect);

        kc.set_camera(camera_rect);
        
        kc.set_colour(Vec4::new(0.6, 0.0, 0.0, 1.0));
        kc.set_depth(1.5);
        kc.circle(self.player_pos, player_radius);

        // render melee arc
        if self.t < self.player_draw_arc_t {
            kc.set_depth(1.4);
            let arc_alpha = (self.player_draw_arc_t - self.t)/arc_duration;
            kc.set_colour(Vec4::new(1.0, 1.0, 1.0, arc_alpha));

            // for collision dot product of player facing and bearing to enemy

            let dfacing = (mouse_pos - self.player_pos);
            let facing_angle = dfacing.y.atan2(dfacing.x);// probably atan2 or something
            kc.poly_part(self.player_pos, player_melee_radius, facing_angle - PI/4., facing_angle + PI/4., 20);
        }

        // render pickups
        // maybe make it so the actual pickup is the pickup and not the shadow
        for p in self.scrap_pos.iter() {
            kc.set_depth(1.1);
            kc.set_colour(Vec4::new(0.0, 0.0, 0.0, 0.4));
            kc.circle(*p, 0.02);
            kc.set_depth(1.5);
            kc.set_colour(Vec4::new(0.0, 0.0, 1.0, 1.0));
            kc.circle(*p + Vec2::new(0.0, -0.05 + (self.t * 3.0).sin() * 0.015), 0.02);
        }

        // render enemies
        for idx in 0..self.enemy_pos.len() {
            kc.set_depth(1.5);
            kc.set_colour(Vec4::new(0.0, 0.0, 0.0, 1.0));
            kc.circle(self.enemy_pos[idx], enemy_radius + enemy_radius_per_scrap * self.enemy_scrap[idx] as f32);
        }

        // render buildings
        for i in 0..self.building_pos.len() {
            let building_rect = Rect::new(self.building_pos[i].0 as f32 * building_s, self.building_pos[i].1 as f32 * building_s, building_s, building_s);
            if self.building_kind[i] == 0 {
                // wall
                kc.set_depth(1.5);
                kc.set_colour(Vec4::new(0.5, 0.5, 0.5, 1.0));
                kc.rect(building_rect);
                kc.set_depth(1.6);
                kc.set_colour(Vec4::new(0.3, 0.3, 0.3, 1.0));
                kc.rect(building_rect.dilate_pc(-0.05));
            }
        }
        
        // render hover building
        if let Some(bk) = self.player_place_building_kind {
            let i = (mouse_pos.x / building_s).floor() as i32;
            let j = (mouse_pos.y / building_s).floor() as i32;
            let building_rect = Rect::new(i as f32 * building_s, j as f32 * building_s, building_s, building_s);
            if bk == 0 {
                // wall
                kc.set_depth(1.5);
                kc.set_colour(Vec4::new(0.5, 0.5, 0.5, 0.5));
                kc.rect(building_rect);
                kc.set_depth(1.6);
                kc.set_colour(Vec4::new(0.3, 0.3, 0.3, 0.5));
                kc.rect(building_rect.dilate_pc(-0.05));
            }
        }

        let grass_spacing = 0.15;
        let grass_max_offset = 0.03;
        let mut grass_x = kround(camera_rect.left() - 0.1, grass_spacing) - grass_spacing/2.; // almost fixes it hey
        // let mut grass_x = camera_rect.left() - 0.1;
        while grass_x < camera_rect.right() + 0.1 {
            let mut grass_y = kround(camera_rect.top() - 0.1, grass_spacing) - grass_spacing/2.;
            // let mut grass_y = camera_rect.top() - 0.1;
            while grass_y < camera_rect.bot() + 0.1 {
                let site_seed = seed_grid(69, grass_x, grass_y, grass_spacing);
                if chance(site_seed, 0.2) {
                    let xo = kuniform(site_seed * 1231513, -grass_max_offset, grass_max_offset);
                    let yo = kuniform(site_seed * 1238987, -grass_max_offset, grass_max_offset);
                    
                    // just draw rect for now but i will draw a shadow layer then a grass layer
                    // what does a grass look like
                    kc.set_depth(1.1);
                    kc.set_colour(Vec4::new(0.0, 0.0, 0.0, 0.4));
                    let x = grass_x + xo;
                    let y = grass_y + yo;
                    kc.triangle(Vec2::new(x, y), Vec2::new(x + 0.01, y), Vec2::new(x - 0.02, y - 0.03));
                    kc.triangle(Vec2::new(x, y), Vec2::new(x + 0.01, y), Vec2::new(x - 0.01 - 0.02, y - 0.025));
                    kc.triangle(Vec2::new(x, y), Vec2::new(x + 0.01, y), Vec2::new(x + 0.01 - 0.02, y - 0.025));
                    kc.set_depth(1.2);
                    kc.set_colour(Vec4::new(0.2, 0.8, 0.0, 1.0));
                    kc.triangle(Vec2::new(x, y), Vec2::new(x + 0.01, y), Vec2::new(x, y - 0.03));
                    kc.triangle(Vec2::new(x, y), Vec2::new(x + 0.01, y), Vec2::new(x - 0.01, y - 0.025));
                    kc.triangle(Vec2::new(x, y), Vec2::new(x + 0.01, y), Vec2::new(x + 0.01, y - 0.025));

                }
                grass_y += grass_spacing;
            }
            grass_x += grass_spacing;
        }
        
        kc.set_camera(inputs.screen_rect);
        kc.set_depth(10.0);
        let darkness = (day_t * 2.0 * PI).sin().max(0.0).min(0.8);
        kc.set_colour(Vec4::new(0.0, 0.0, 0.0, darkness));
        kc.rect(inputs.screen_rect);

        self.init = false;
    }
}