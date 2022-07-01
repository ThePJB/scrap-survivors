// pub struct Sound {
//     data: Vec<f32>,
// }

// pub struct Mixer {
//     sample_count: u64,
//     playing_chunks: Vec<(&'static Sound, u64)>,
// }

// impl Mixer {
//     pub fn play_immediately(&mut self, s: &'static Sound) {
//         self.playing_chunks.push((s, self.sample_count));
//     }

//     pub fn next(n: u64) -> Vec<f32> {

//     }
// }

// // I guess cpal synth example is pretty good and can do what I want probably
// // a struct and a function called per sample