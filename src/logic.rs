use crate::constants::*;
use olc_pge as olc;
use rayon::{slice::ParallelSliceMut, prelude::ParallelIterator};
use vek::vec2::Vec2;

#[derive(PartialEq, Clone, Copy)]
pub enum Mode
{
    Hand,
    Cut,
    Place,
    Force,
}

#[derive(Clone, Copy, Debug)]
pub struct Point
{
    pub pos: Vec2<f32>,
    pub prev: Vec2<f32>,
    connection_count: u32,
    pub is_static: bool,
}

#[derive(Clone, Copy)]
pub struct Stick
{
    pub start: usize,
    pub end: usize,
    pub target_length: f32,
}

impl crate::Window
{
    //used verlet integration for this
    //https://youtu.be/3HjO_RGIjCU
    // ^ very useful link
    #[inline(never)]
    pub fn simulate(
        &mut self,
        delta: f32,
        iterations: u32,
        gravity: f32,
        pge: &mut olc::PixelGameEngine,
    )
    {
        //moves points one time step further
        self.points.par_chunks_mut(200).for_each(|points| 
                points.iter_mut().for_each(|p|
                if !p.is_static
                {
                    let temp = p.pos;
                    p.pos += p.pos - p.prev;
                    p.pos += Vec2::new(0.0, 1.0) * gravity * delta * delta;
                    p.prev = temp;
                    p.prev = p.pos + (p.prev - p.pos) * 0.999;
                }
            )
        );

        for _ in 0..iterations
        {
            self.sticks.iter().zip(self.stickscopy.iter())
            .for_each(|(s, scopy)|
                {
                    let stickcentre = (self.points[s.start].pos + self.points[s.end].pos) / 2.0;
                    let stickdir = (self.points[s.start].pos - self.points[s.end].pos).normalized();

                    //keeps points at constant distance from eachother,
                    //at least in theory. with less iterations it's bouncier
                    //with more iterations it becomes rigid
                    if !self.points[s.start].is_static
                    {
                        self.points[scopy.start].pos = stickcentre + stickdir * s.target_length / 2.0;
                    }

                    if !self.points[s.end].is_static
                    {
                        self.points[scopy.end].pos = stickcentre - stickdir * s.target_length / 2.0;
                    }    
                }
            );
            //swaps read and write buffers
            std::mem::swap(&mut self.sticks, &mut self.stickscopy);
            self.constrain_points(pge);
        }
    }

    pub fn handle_fps(&mut self, fps: f32, elapsed_time: f32)
    {
        self.avg_frame_time = (self.avg_frame_time * (fps - 1.0) + elapsed_time) / fps;
        std::thread::sleep(std::time::Duration::from_secs_f32(
            (1.0 / fps - self.avg_frame_time).max(0.0),
        ));
        self.smoothdelta = (self.smoothdelta + elapsed_time * 0.1) / 1.1;
    }

    #[inline(never)]
    pub fn handle_input(&mut self, pge: &mut olc::PixelGameEngine)
    {
        if !((self.currentmode == Mode::Hand) && pge.get_mouse(0).held)
        {
            self.closest_point =
                self.get_closest_point(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
        }

        if pge.get_key(olc::Key::S).pressed
        {
            if let Some(closest) = self.closest_point
            {
                let dist2 = 
                (self.points[closest].pos - Vec2::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32))
                .magnitude_squared();
                let nearmouse = dist2 < CHANGE_STATE_RADIUS;
                
                self.points[closest].is_static = if nearmouse
                {
                    true
                }
                else
                {
                    self.points[closest].is_static
                };
            }
        }
        else if pge.get_key(olc::Key::D).pressed
        {
            if let Some(closest) = self.closest_point
            {
                let dist2 = 
                (self.points[closest].pos - Vec2::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32))
                .magnitude_squared();
                let nearmouse = dist2 < CHANGE_STATE_RADIUS;

                self.points[closest].is_static = if nearmouse
                {
                    false
                }
                else
                {
                    self.points[closest].is_static
                };
            }
        }
        if pge.get_key(olc::Key::H).pressed
        {
            self.currentmode = Mode::Hand;
        }
        else if pge.get_key(olc::Key::C).pressed
        {
            self.currentmode = Mode::Cut;
        }
        else if pge.get_key(olc::Key::P).pressed
        {
            self.currentmode = Mode::Place;
        }
        else if pge.get_key(olc::Key::F).pressed
        {
            self.currentmode = Mode::Force;
        }
        if pge.get_key(olc::Key::G).pressed
        {
            self.generate_grid(64, 6.0);
        }

        if pge.get_key(olc::Key::Q).pressed
        {
            println!("DEBUG INFORMATION:");
            println!("amount of points: {}", self.points.len());
            println!("amount of sticks: {}", self.sticks.len());
            println!("value of newstickstart: {:?}", self.newstickstart);
            println!("value of newstickend: {:?}", self.newstickend);
            println!("value of highlight: {:?}", self.closest_point);
            if let Some(i) = self.closest_point
            {
                println!("{:?}", self.points[i]);
            }
        }
        if pge.get_key(olc::Key::A).pressed
        {
            //disconnects all points
            println!("deleted all sticks.");
            self.sticks.clear();
            self.stickscopy.clear();
        }
        if pge.get_key(olc::Key::W).pressed
        {
            //erases everything.
            println!("deleted all points.");
            self.sticks.clear();
            self.stickscopy.clear();
            self.points.clear();
        }

        if pge.get_key(olc::Key::T).pressed
        {
            self.generate_connections();
        }

        if pge.get_mouse(1).pressed
        {
            self.start_connection(pge);
        }
        else
        {
            if pge.get_mouse(1).released
            {
                self.end_connection(pge);
            }
            if !pge.get_mouse(1).held
            {
                //resets
                self.newstickstart = None;
                self.newstickend = None;
            }
        }
        if let Some(newstart) = self.newstickstart
        {
            let temp = self.get_closest_point(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            if let Some(tempend) = temp
            {
                if tempend != newstart
                {
                    let x0 = self.points[newstart].pos.x as i32;
                    let y0 = self.points[newstart].pos.y as i32;
                    let x1 = self.points[tempend].pos.x as i32;
                    let y1 = self.points[tempend].pos.y as i32;
                    pge.draw_line(x0, y0, x1, y1, olc::Pixel::rgb(255, 200, 100));
                }
            }
        }
    }

    pub fn generate_connections(&mut self)
    {
        for i in 0..self.points.len()
        {
            for j in 0..self.points.len()
            {
                if i != j
                {
                    let stick = Stick {
                        start: i,
                        end: j,
                        target_length: (self.points[i].pos - self.points[j].pos).magnitude(),
                    };
                    self.add_stick(stick)
                }
            }
        }
    }

    #[inline(never)]
    //generates a grid that acts like cloth
    pub fn generate_grid(&mut self, dim: u32, stepsize: f32) // 20.0
    {
        let initial_size = self.points.len();
        for i in 0..dim
        {
            for j in 0..dim
            {
                let temp = Vec2::new(20.0 + j as f32 * stepsize, 20.0 + i as f32 * stepsize);
                let should_be_static = i == 0 && (j % 4 == 0 || j == dim - 1);
                self.points.push(Point {
                    pos: temp,
                    prev: temp,
                    is_static: should_be_static,
                    connection_count: 0,
                });
            }
        }
        for i in 0..dim - 1
        {
            for j in 0..dim - 1
            {
                let s = initial_size + (i * dim + j) as usize;
                let e1 = initial_size + ((i) * dim + j + 1) as usize;

                self.add_stick(Stick {
                    start: s,
                    end: e1,
                    target_length: (self.points[s].pos - self.points[e1].pos).magnitude(),
                });
                let e2 = initial_size + ((i + 1) * dim + j) as usize;
                self.add_stick(Stick {
                    start: s,
                    end: e2,
                    target_length: (self.points[s].pos - self.points[e2].pos).magnitude(),
                });
            }
        }
        for i in 0..dim - 1
        {
            let mut s = initial_size + ((i + 1) * dim + dim - 1) as usize;
            let mut e = initial_size + (i * dim + dim - 1) as usize;
            self.add_stick(Stick {
                start: s,
                end: e,
                target_length: (self.points[s].pos - self.points[e].pos).magnitude(),
            });
            s = initial_size + (dim * (dim - 1) + i) as usize;
            e = initial_size + (dim * (dim - 1) + i + 1) as usize;
            self.add_stick(Stick {
                start: s,
                end: e,
                target_length: (self.points[s].pos - self.points[e].pos).magnitude(),
            });
        }
    }

    //snaps all sticks that become too long
    #[inline(never)]
    pub fn snap_apart_too_long_sticks(&mut self)
    {
        assert!(self.orphans.is_empty());
        let mut i = 0;
        
        while i != self.sticks.len()
        {
            let current_length =
                (self.points[self.sticks[i].start].pos - self.points[self.sticks[i].end].pos).magnitude();
            if current_length / self.sticks[i].target_length > SNAP_RATIO
            {
                let dot_indices = self.remove_stick(i);
                self.orphans.push(dot_indices.0);
                self.orphans.push(dot_indices.1);
                self.counter += 1;
            }
            else
            {
                i += 1;
            }
        }
        self.delete_orphan_points();
    }

    #[inline(never)]
    pub fn delete_orphan_points(&mut self)
    {
        let mut i = 0;
        self.orphans.sort();
        self.orphans.dedup();
        while i != self.orphans.len()
        {
            let point = self.orphans[i];
            let remove = match self.points.get(point)
            {
                Some(p) => p.connection_count == 0,
                None => false,
            };
            if remove
            {
                if self.points.len() == 0
                {
                    return;
                }
                self.points.swap_remove(point);
                self.closest_point = None;
                if let Some(p) = self.closest_point
                {
                    if p == self.points.len() && !self.points.is_empty()
                    {
                        self.closest_point = Some(point);
                    }
                    if p == point
                    {
                        self.closest_point = None;
                    }
                }
                self.sticks.par_chunks_mut(200).chain(self.stickscopy.par_chunks_mut(200)).for_each(|sticks|
                    {
                        sticks.iter_mut().for_each(|stick|

                            for idx in [&mut stick.start, &mut stick.end]
                            {
                                if *idx == self.points.len()
                                {
                                    *idx = point;
                                }
                            }
                        )
                    }
                );
                let mut j = 0;
                while j != self.orphans.len()
                {
                    if self.orphans[j] == self.points.len()
                    {
                        self.orphans[j] = point;
                    }
                    j += 1;
                }
            }
            i += 1;
        }
        self.orphans.clear();
    }

    #[inline(never)]
    //bounces the points off the window borders
    pub fn constrain_points(&mut self, pge: &mut olc::PixelGameEngine)
    {
        let sw = pge.screen_width() as f32;
        let sh = pge.screen_height() as f32;
        self.points.par_chunks_mut(200).for_each(|points|
            points.iter_mut().for_each(|point|
                {
                    let mut p = *point;
                    let vel = p.pos - p.prev;
                    
                    if p.pos.x > sw - POINT_RADIUS
                    {
                        p.pos.x = sw - POINT_RADIUS;
                        p.prev.x = p.pos.x + vel.x.abs();
                    }
                    else if p.pos.x < POINT_RADIUS
                    {
                        p.pos.x = POINT_RADIUS;
                        p.prev.x = p.pos.x - vel.x.abs();
                    }
                    if p.pos.y > sh - POINT_RADIUS
                    {
                        p.pos.y = sh - POINT_RADIUS;
                        p.prev.y = p.pos.y + vel.y.abs();
                    }
                    else if p.pos.y < POINT_RADIUS
                    {
                        p.pos.y = POINT_RADIUS;
                        p.prev.y = p.pos.y - vel.y.abs();
                    }
                    *point = p;
                }
            )
        );
    }

    #[inline(never)]
    //gets the distance between a line segment (aka Stick), and a point
    pub fn distance(&self, st: Stick, pt: Vec2<f32>) -> f32
    {
        let a = self.points[st.start].pos;
        let b = self.points[st.end].pos;

        //length squared
        let l2 = (a - b).magnitude_squared();
        if l2 < 0.001
        {
            return (a - pt).magnitude_squared();
        }
        let t = 0.0f32.max(1.0f32.min((pt - a).dot(b - a) / l2));
        let projection = a + (b - a) * t;
        (pt - projection).magnitude()
    }

    #[inline(never)]
    //returns whether two line segments intersect or not
    pub fn intersects(start1: Vec2<f32>, end1: Vec2<f32>, start2: Vec2<f32>, end2: Vec2<f32>) -> bool
    {
        //t=
        //(x1-x3)(y3-y4) - (y1-y3)(x3-x4)
        //______________________________
        //(x1-x2)(y3-y4) - (y1-y2)(x3-x4)

        //u=
        //(x1-x3)(y1-y2) - (y1-y3)(x1-x2)
        //______________________________
        //(x1-x2)(y3-y4) - (y1-y2)(x3-x4)

        let a = start1;
        let b = end1;
        let c = start2;
        let d = end2;

        let mag = (start2 - end2).magnitude();

        let x12 = a.x - b.x;
        let y12 = a.y - b.y;
        let x13 = a.x - c.x;
        let y13 = a.y - c.y;
        let y34 = c.y - d.y;
        let x34 = c.x - d.x;

        let mut t = x13 * y34 - y13 * x34;
        t /= x12 * y34 - y12 * x34;
        let range = -0.0 - 1.0 / mag..=1.0 + 1.0 / mag;
        if !range.contains(&t)
        {
            return false;
        }
        let mut u = x13 * y12 - y13 * x12;
        u /= x12 * y34 - y12 * x34;
        if !range.contains(&u)
        {
            return false;
        }
        true
    }

    #[inline(never)]
    //-1 when no points exist
    pub fn get_closest_point(&mut self, x: f32, y: f32) -> Option<usize>
    {
        if self.points.is_empty()
        {
            return None;
        }
        let mut out = 0;
        let pos = Vec2 { x, y };
        for i in 0..self.points.len()
        {
            if (pos - self.points[i].pos).magnitude_squared() < (pos - self.points[out].pos).magnitude_squared()
            {
                out = i;
            }
        }
        Some(out)
    }


    pub fn move_point(&mut self, pge: &mut olc::PixelGameEngine)
    {
        if pge.get_mouse(0).held
        {
            if let Some(closest) = self.closest_point
            {
                self.points[closest].pos =
                    Vec2::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            }
        }
    }

    #[inline(never)]
    //can leave orphan points that bounce around
    pub fn cut_sticks(&mut self, pge: &mut olc::PixelGameEngine)
    {
        assert!(self.orphans.is_empty());
        if pge.get_mouse(0).held
        {
            let current_mouse_pos = Vec2::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            let mouse_delta = current_mouse_pos - self.previous_mouse_pos;
            let mut i = 0;
            while i != self.sticks.len()
            {
                //switches between cutting by removing nearby sticks (when mouse moves slowly),
                //and cutting by intersecting the mouse's movement vector with the stick (when mouse moves quickly)
                if mouse_delta.magnitude() < CUT_RADIUS * 2.0
                {
                    let mousepos = Vec2::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
                    if self.distance(self.sticks[i], mousepos) >= CUT_RADIUS
                    {
                        i += 1;
                        continue;
                    }
                }
                else if !(Self::intersects(
                    current_mouse_pos,
                    self.previous_mouse_pos,
                    self.points[self.sticks[i].start].pos,
                    self.points[self.sticks[i].end].pos,
                ))
                {
                    i += 1;
                    continue;
                }
                //moves points a bit when cutting them apart
                let vel = self.points[self.sticks[i].end].pos - self.points[self.sticks[i].end].prev;
                let relative_vel = mouse_delta - vel;
                self.points[self.sticks[i].start].pos += relative_vel.normalized() / relative_vel.magnitude_squared().max(1.0).min(30.0) * 10.0;
                self.points[self.sticks[i].end].pos += relative_vel.normalized() / relative_vel.magnitude_squared().max(1.0).min(30.0) * 10.0;
                if relative_vel.magnitude_squared() > 9.0
                {
                    let dot_indices = self.remove_stick(i);
                    self.orphans.push(dot_indices.0);
                    self.orphans.push(dot_indices.1);
                    self.counter += 1;
                }
                else
                {
                    i += 1;
                }
            }
        }

        self.delete_orphan_points();
    }

    pub fn spawn_point(&mut self, pge: &mut olc::PixelGameEngine)
    {
        if pge.get_mouse(0).pressed
        {
            let temp = Vec2::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            self.points.push(Point {
                pos: temp,
                prev: temp,
                is_static: false,
                connection_count: 0,
            });
        }
    }

    #[inline(never)]
    pub fn apply_force(&mut self, pge: &mut olc::PixelGameEngine)
    {
        if pge.get_mouse(0).held
        {
            let mp = Vec2::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            for i in 0..self.points.len()
            {
                if !self.points[i].is_static
                {
                    let p = &mut self.points[i];
                    let dir = mp - p.pos;
                    let (norm, mag) = dir.normalized_and_get_magnitude();
                    p.pos += norm / (mag + 0.5) * FORCE;
                }
            }
        }
    }

    //for connecting points using sticks
    pub fn start_connection(&mut self, pge: &mut olc::PixelGameEngine)
    {
        self.newstickstart =
            self.get_closest_point(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
    }

    pub fn end_connection(&mut self, pge: &mut olc::PixelGameEngine)
    {
        if let Some(newstart) = self.newstickstart
        {
            self.newstickend =
                self.get_closest_point(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            if let Some(newend) = self.newstickend
            {
                if self.newstickstart != self.newstickend
                {
                    let len = (self.points[newstart].pos - self.points[newend].pos).magnitude();
                    self.add_stick(Stick {
                        start: newstart,
                        end: newend,
                        target_length: len,
                    });
                    self.newstickstart = None;
                    self.newstickend = None;
                }
            }
        }
    }

    #[inline(never)]
    pub fn remove_stick(&mut self, index: usize) -> (usize, usize)
    {
        let stick = self.sticks.swap_remove(index);
        let stick2 = self.stickscopy.swap_remove(index);
        self.points[stick.start].connection_count -= 1;
        self.points[stick.end].connection_count -= 1;
        (stick.start, stick.end)
    }
    pub fn add_stick(&mut self, st: Stick)
    {
        self.sticks.push(st);
        self.stickscopy.push(st);
        self.points[st.start].connection_count += 1;
        self.points[st.end].connection_count += 1;
    }
}
