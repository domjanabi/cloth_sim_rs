use olc::Vf2d;
use olc_pge as olc;
pub use web_audio_api::context::{AudioContext, BaseAudioContext};
pub use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};

pub mod audio;
pub mod constants;
pub mod rendering;
pub mod logic;
use logic::*;

fn main()
{
    let context = AudioContext::default();
    let window = Window::new(context);
    olc::PixelGameEngine::construct(window, 800, 450, 2, 2).start();
}

impl olc::PGEApplication for Window
{
    const APP_NAME: &'static str = "lmao";

    fn on_user_create(&mut self, pge: &mut olc::PixelGameEngine) -> bool
    {
        pge.clear(olc::Pixel::rgb(40, 45, 35));
        self.points.reserve(400);
        self.sticks.reserve(400);
        self.stickscopy.reserve(400);

        true
    }
    fn on_user_update(&mut self, pge: &mut olc::PixelGameEngine, elapsed_time: f32) -> bool
    {
        let fps = 60.0;
        self.handle_fps(fps, elapsed_time);
        pge.clear(olc::Pixel::rgb(33, 36, 30));

        self.handle_input(pge);
        match self.currentmode
        {
            Mode::Hand => self.move_point(pge),
            Mode::Cut => self.cut_sticks(pge),
            Mode::Place => self.spawn_point(pge),
            Mode::Force => self.apply_force(pge),
        }

        //space used to "stop time"
        if !pge.get_key(olc::Key::Space).held
        {
            self.simulate(1.0 / 100.0, 8, 200.0, pge);
        }

        self.snap_apart_too_long_sticks();
        self.render(pge);
        self.play_sfx();

        self.previous_mouse_pos = Vf2d::new(pge.get_mouse_x() as f32, pge.get_mouse_y() as f32);

        true
    }
}

impl Window
{
    fn new(audio_context: AudioContext) -> Window
    {
        Window
        {
            smoothdelta: 0.1,
            previous_mouse_pos: Vf2d::new(0.0, 0.0),
            //indices for creating new sticks
            newstickstart: None,
            newstickend: None,
            //index to the point closest to the mouse
            closest_point: None,
    
            points: vec![],
            //write buffer
            sticks: vec![],
            // ^
            // | these two get swapped each iteration(!= frame)
            // v
            //read buffer
            stickscopy: vec![],
            currentmode: Mode::Place,
            context: audio_context,
            avg_frame_time: 0.0,
            counter: 0,
        }
    }
}

struct Window
{
    previous_mouse_pos: Vf2d,
    smoothdelta: f32,
    //indices for creating new sticks
    newstickstart: Option<usize>,
    newstickend: Option<usize>,
    //index to the point closest to the mouse
    closest_point: Option<usize>,

    points: Vec<Point>,
    //write buffer
    sticks: Vec<Stick>,
    // ^
    // | these two get swapped each iteration(!= frame)
    // v
    //read buffer
    stickscopy: Vec<Stick>,
    currentmode: Mode,
    context: AudioContext,
    avg_frame_time: f32,
    counter: usize,
}
