pub use web_audio_api::context::{AudioContext, BaseAudioContext};
pub use web_audio_api::node::{AudioNode, AudioScheduledSourceNode};


//TODO: Change everything about this.
impl crate::Window
{
    pub fn play_sfx(&mut self)
    {
        let mut buffer = self.context.create_buffer(
            1,
            (self.context.sample_rate() / 60.0) as usize,
            self.context.sample_rate(),
        );

        generate_ticks(
            buffer.get_channel_data_mut(0),
            self.counter,
            fastrand::usize(80..100),
        );
        self.counter = 0;

        // play the buffer at given volume
        let volume = self.context.create_gain();
        volume.connect(&self.context.destination());
        volume.gain().set_value(0.5);

        let buffer_source = self.context.create_buffer_source();
        buffer_source.connect(&volume);
        buffer_source.set_buffer(buffer);

        // start the sources
        buffer_source.start();
    }
}

pub fn generate_ticks(data: &mut [f32], frequency: usize, length: usize)
{
    for sample in data.iter_mut()
    {
        *sample = 0.0;
    }
    let len = length;
    for f in 0..frequency
    {
        for l in 0..len
        {
            let offset = f * (data.len() / frequency);
            data[l.min(data.len() - offset - 1) + offset] = 0.7;
        }
    }
}