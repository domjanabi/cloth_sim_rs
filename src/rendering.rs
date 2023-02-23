use olc_pge as olc;
use crate::Mode;
use crate::constants::*;

impl crate::Window
{
    pub fn render(&mut self, pge: &mut olc::PixelGameEngine)
    {
        if EXPENSIVE_POINT_RENDERING
        {
            self.render_points(pge);
        }
        self.render_sticks(pge);
        if !EXPENSIVE_POINT_RENDERING
        {
            self.render_points(pge);
        }

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
        if EXPENSIVE_POINT_RENDERING
        {
            for i in 0..self.points.len()
            {
                let pos = self.points[i].pos;
                if Some(i) != self.closest_point
                {
                    Self::fill_circle(pos.x, pos.y, POINT_RADIUS, POINT_COLOUR, pge);
                }
                else
                {
                    //renders closest point to mouse highlighted
                    Self::fill_circle(pos.x, pos.y, HIGHLIGHT_RADIUS, HIGHLIGHT_COLOUR, pge);
                    Self::fill_circle(pos.x, pos.y, POINT_RADIUS, POINT_COLOUR, pge);
                }
            }
        }
        else
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
                    pge.fill_circle(pos.x as i32, pos.y as i32, 2, HIGHLIGHT_COLOUR);
                }
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
}