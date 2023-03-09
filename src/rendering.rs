use crate::constants::*;
use crate::Mode;
use olc_pge as olc;

impl crate::Window
{
    
    pub fn render(&mut self, pge: &mut olc::PixelGameEngine)
    {
        pge.clear(olc::Pixel::rgb(33, 36, 30));
        self.render_sticks(pge);
        self.render_points(pge);

        self.render_mode_symbol(pge);
    }


    pub fn render_mode_symbol(&mut self, pge: &mut olc::PixelGameEngine)
    {
        for y in 0..SYMBOL_HEIGHT
        {
            for x in 0..SYMBOL_WIDTH
            {
                let index = y * SYMBOL_WIDTH + x;
                let should_draw = match self.currentmode
                {
                    Mode::Hand => HAND_MODE_SYMBOL,
                    Mode::Cut => CUT_MODE_SYMBOL,
                    Mode::Place => PLACE_MODE_SYMBOL,
                    Mode::Force => FORCE_MODE_SYMBOL,
                }[index];
                if should_draw
                {
                    pge.draw(x as i32 + 5, y as i32 + 5, olc::WHITE);
                }
            }
        }
        for y in 0..SYMBOL_HEIGHT + 6
        {
            for x in 0..SYMBOL_WIDTH + 6
            {
                if x.min(y) <= 1 || (x >= SYMBOL_WIDTH + 4 || y >= SYMBOL_HEIGHT + 4)
                {
                    let colour = if x * SYMBOL_HEIGHT >= y * SYMBOL_WIDTH
                    {
                        olc::Pixel::rgb(255, 220, 170)
                    }
                    else
                    {
                        olc::Pixel::rgb(255, 150, 80)
                    };
                    pge.draw(x as i32 + 2, y as i32 + 2, colour);
                }
            }
        }
    }

    
    pub fn render_points(&mut self, pge: &mut olc::PixelGameEngine)
    {
        for i in 0..self.points.len()
        {
            let pos = self.points[i].pos;
            if Some(i) != self.closest_point
            {
                pge.draw(pos.x as i32, pos.y as i32, POINT_COLOUR);
            }
            else
            {
                Self::fill_circle(pos.x, pos.y, 2.0, HIGHLIGHT_COLOUR, pge);
            }
        }
    }

    
    pub fn render_sticks(&self, pge: &mut olc::PixelGameEngine)
    {
        for stick in self.sticks.iter()
        {
            pge.draw_line(
                self.points[stick.start].pos.x as i32,
                self.points[stick.start].pos.y as i32,
                self.points[stick.end].pos.x as i32,
                self.points[stick.end].pos.y as i32,
                STICK_COLOUR,
            );
        }
    }
    
    pub fn fill_circle(
        x: f32,
        y: f32,
        radius: f32,
        colour: olc::Pixel,
        pge: &mut olc::PixelGameEngine,
    )
    {
        let startx = (x - radius).floor() as i32 - 1;
        let starty = (y - radius).floor() as i32 - 1;
        let endx = (x + radius).ceil() as i32 + 1;
        let endy = (y + radius).ceil() as i32 + 1;
        for i in starty..endy
        {
            for j in startx..endx
            {
                let deltax = j as f32 - x;
                let deltay = i  as f32 - y;
                let dist2 = deltax * deltax + deltay * deltay;
                if dist2.sqrt() <= radius
                {
                    pge.draw(j, i, colour);
                }
            }
        }
    }
}
