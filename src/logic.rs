use olc_pge as olc;
use olc::Vf2d;
use crate::constants::*;

#[derive(PartialEq, Clone, Copy)]
pub enum Mode
{
    Hand,
    Cut,
    Place,
    Force,
}

#[derive(Clone, Copy)]
pub struct Point
{
    pub pos: Vf2d,
    pub prev: Vf2d,
    connection_count: u16,
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
    pub fn simulate(
        &mut self,
        delta: f32,
        iterations: u32,
        gravity: f32,
        pge: &mut olc::PixelGameEngine,
    )
    {
        //moves points one time step further
        for i in 0..self.points.len()
        {
            if !self.points[i].is_static
            {
                let p = &mut self.points[i];
                let temp = p.pos;
                p.pos += p.pos - p.prev;
                p.pos += Vf2d::new(0.0, 1.0) * gravity * delta * delta;
                p.prev = temp;
                p.prev = p.pos + (p.prev - p.pos) * 0.999;
            }
        }

        for _ in 0..iterations
        {
            for i in 0..self.sticks.len()
            {
                let s = &mut self.sticks[i];
                let scopy = &mut self.stickscopy[i];
                let stickcentre = (self.points[s.start].pos + self.points[s.end].pos) / 2.0;
                let stickdir = (self.points[s.start].pos - self.points[s.end].pos).norm();

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
                let nearmouse = (self.points[closest].pos
                    - Vf2d::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32))
                .mag2()
                    < CHANGE_STATE_RADIUS;
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
                let nearmouse = (self.points[closest].pos
                    - Vf2d::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32))
                .mag2()
                    < CHANGE_STATE_RADIUS;
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
            self.generate_grid(64, 5.0);
        }

        if pge.get_key(olc::Key::Q).pressed
        {
            println!("DEBUG INFORMATION:");
            println!("amount of points: {}", self.points.len());
            println!("amount of sticks: {}", self.sticks.len());
            println!("value of newstickstart: {:?}", self.newstickstart);
            println!("value of newstickend: {:?}", self.newstickend);
            println!("value of highlight: {:?}", self.closest_point);
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
                    let stick = Stick{start: i, end: j, target_length: (self.points[i].pos - self.points[j].pos).mag()};
                    self.add_stick(stick)
                }
            }    
        }
    }

    //generates a grid that acts like cloth
    pub fn generate_grid(&mut self, dim: u32, stepsize: f32) // 20.0
    {
        let initial_size = self.points.len();
        for i in 0..dim
        {
            for j in 0..dim
            {
                let temp = Vf2d::new(20.0 + j as f32 * stepsize, 20.0 + i as f32 * stepsize);
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
                    target_length: (self.points[s].pos - self.points[e1].pos).mag(),
                });
                let e2 = initial_size + ((i + 1) * dim + j) as usize;
                self.add_stick(Stick {
                    start: s,
                    end: e2,
                    target_length: (self.points[s].pos - self.points[e2].pos).mag(),
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
                target_length: (self.points[s].pos - self.points[e].pos).mag(),
            });
            s = initial_size + (dim * (dim - 1) + i) as usize;
            e = initial_size + (dim * (dim - 1) + i + 1) as usize;
            self.add_stick(Stick {
                start: s,
                end: e,
                target_length: (self.points[s].pos - self.points[e].pos).mag(),
            });
        }
    }

    //snaps all sticks that become too long
    pub fn snap_apart_too_long_sticks(&mut self)
    {
        let mut potentially_lone_dots = Vec::new();
        let mut i = 0;
        while i != self.sticks.len()
        {
            let current_length =
                (self.points[self.sticks[i].start].pos - self.points[self.sticks[i].end].pos).mag();
            if current_length / self.sticks[i].target_length > SNAP_RATIO
            {
                let dot_indices = self.remove_stick(i);
                potentially_lone_dots.push(dot_indices.0);
                potentially_lone_dots.push(dot_indices.1);
                potentially_lone_dots.push(dot_indices.2);
                potentially_lone_dots.push(dot_indices.3);
                i -= 1;
                self.counter += 1;
            }
            i += 1;
        }
        self.delete_orphan_points(&mut potentially_lone_dots);
    }

    pub fn delete_orphan_points(&mut self, potential_orphans: &mut[usize])
    {
        let mut i = 0;
        while i != potential_orphans.len()
        {
            let point = potential_orphans[i];
            let remove = self.points[point].connection_count == 0;
            if remove
            {
                if self.points.len() == 0
                {
                    return;
                }
                self.points.remove(point);
                self.closest_point = None;
                if let Some(p) = self.closest_point
                {
                    if p > point
                    {
                        self.closest_point = Some(p - 1);
                    }
                    if p == point
                    {
                        self.closest_point = None;
                    }
                }
                for stick in self.sticks.iter_mut().chain(self.stickscopy.iter_mut())
                {
                    for idx in [&mut stick.start, &mut stick.end]
                    {
                        if *idx >= point
                        {
                            *idx = (*idx).max(1) - 1;
                        }
                    }
                }
                let mut j = 0;
                while j != potential_orphans.len()
                {
                    if potential_orphans[j] >= point
                    {
                        potential_orphans[j] = potential_orphans[j].max(1) - 1;
                    }
                    j += 1;
                }
            }
            i += 1;
        }
    }

    //bounces the points off the window borders
    pub fn constrain_points(&mut self, pge: &mut olc::PixelGameEngine)
    {
        for i in 0..self.points.len()
        {
            let mut p = self.points[i];
            let vel = p.pos - p.prev;
            let sw = pge.screen_width() as f32;
            let sh = pge.screen_height() as f32;
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
            self.points[i] = p;
        }
    }

    //gets the distance between a line segment (aka Stick), and a point
    pub fn distance(&self, st: Stick, pt: Vf2d) -> f32
    {
        let a = self.points[st.start].pos;
        let b = self.points[st.end].pos;

        //length squared
        let l2 = (a - b).mag2();
        if l2 < 0.001
        {
            return (a - pt).mag();
        }
        let t = 0.0f32.max(1.0f32.min((pt - a).dot(&(b - a)) / l2));
        let projection = a + (b - a) * t;
        (pt - projection).mag()
    }

    //returns whether two line segments intersect or not
    pub fn intersects(start1: Vf2d, end1: Vf2d, start2: Vf2d, end2: Vf2d) -> bool
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

        let mag = (start2 - end2).mag();

        let x12 = a.x - b.x;
        let y12 = a.y - b.y;
        let x13 = a.x - c.x;
        let y13 = a.y - c.y;
        let y34 = c.y - d.y;
        let x34 = c.x - d.x;

        let mut t = x13 * y34 - y13 * x34;
        t /= x12 * y34 - y12 * x34;
        let range = -0.0 - 1.0/mag..=1.0 + 1.0 / mag;
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

    //-1 when no points exist
    pub fn get_closest_point(&mut self, x: f32, y: f32) -> Option<usize>
    {
        if self.points.is_empty()
        {
            return None;
        }
        let mut out = 0;
        let pos = Vf2d { x, y };
        for i in 0..self.points.len()
        {
            if (pos - self.points[i].pos).mag2() < (pos - self.points[out].pos).mag2()
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
                    Vf2d::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            }
        }
    }
    //can leave orphan points that bounce around
    pub fn cut_sticks(&mut self, pge: &mut olc::PixelGameEngine)
    {
        let mut potentially_lone_dots = Vec::new();
        if pge.get_mouse(0).held
        {
            let current_mouse_pos = Vf2d::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            let mouse_delta = current_mouse_pos - self.previous_mouse_pos;
            let mut i = 0;
            while i != self.sticks.len()
            {
                //switches between cutting by removing nearby sticks (when mouse moves slowly), 
                //and cutting by intersecting the mouse's movement vector with the stick (when mouse moves quickly)
                if mouse_delta.mag() < CUT_RADIUS * 2.0
                {
                    let mousepos = Vf2d::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
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
                let vel =
                    self.points[self.sticks[i].end].pos - self.points[self.sticks[i].end].prev;
                let relative_vel = mouse_delta - vel;
                self.points[self.sticks[i].start].pos +=
                    relative_vel.norm() / relative_vel.mag2().max(1.0).min(30.0) * 10.0;
                self.points[self.sticks[i].end].pos +=
                    relative_vel.norm() / relative_vel.mag2().max(1.0).min(30.0) * 10.0;
                if relative_vel.mag() > 3.0
                {
                    let dot_indices = self.remove_stick(i);
                    potentially_lone_dots.push(dot_indices.0);
                    potentially_lone_dots.push(dot_indices.1);
                    potentially_lone_dots.push(dot_indices.2);
                    potentially_lone_dots.push(dot_indices.3);
                    self.counter += 1;
                }
                else
                {
                    i += 1;
                }
            }
        }
        
        self.delete_orphan_points(&mut potentially_lone_dots);
    }
    pub fn spawn_point(&mut self, pge: &mut olc::PixelGameEngine)
    {
        if pge.get_mouse(0).pressed
        {
            let temp = Vf2d::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            self.points.push(Point {
                pos: temp,
                prev: temp,
                is_static: false,
                connection_count: 0,
            });
        }
    }
    pub fn apply_force(&mut self, pge: &mut olc::PixelGameEngine)
    {
        if pge.get_mouse(0).held
        {
            let mp = Vf2d::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);
            for i in 0..self.points.len()
            {
                if !self.points[i].is_static
                {
                    let p = &mut self.points[i];
                    let dir = mp - p.pos;
                    p.pos += dir.norm() / (dir.mag() + 0.5) * FORCE;
                }
            }
        }
    }

    //there is a function for doing this in olc::PGE,
    //but it takes in integers for position,
    //which means that it's not actually a circle rendered in low res,
    //but more of a "texture", as there is only one shape the circle can have.
    //replace fillCircle() with FillCircle() calls,
    //and you might see the difference yourself
    pub fn fill_circle(x: f32, y: f32, radius: f32, colour: olc::Pixel, pge: &mut olc::PixelGameEngine)
    {
        let startx = (x - radius) as i32;
        let starty = (y - radius) as i32;
        let endx = (x + radius).ceil() as i32;
        let endy = (y + radius).ceil() as i32;
        for i in starty..endy
        {
            for j in startx..endx
            {
                let deltax = j - x as i32;
                let deltay = i - y as i32;
                let dist2 = deltax * deltax + deltay * deltay;
                if dist2 as f32 <= radius * radius
                {
                    pge.draw(j, i, colour);
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
                    let len = (self.points[newstart].pos - self.points[newend].pos).mag();
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

    pub fn remove_stick(&mut self, index: usize) -> (usize, usize, usize, usize)
    {
        let stick = self.sticks.remove(index);
        let stick2 = self.stickscopy.remove(index);
        self.points[stick.start].connection_count -= 1;
        self.points[stick.end].connection_count -= 1;
        (stick.start, stick.end, stick2.start, stick2.end)
    }
    pub fn add_stick(&mut self, st: Stick)
    {
        self.sticks.push(st);
        self.stickscopy.push(st);
        self.points[st.start].connection_count+=1;
        self.points[st.end].connection_count+=1;
    }
}