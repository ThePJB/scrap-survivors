# Round Survival

todo:
  sound... will be interesting, software synthesis, audio callback or whatever
    behaviour set in frame
  wasm....... and leaderboard lol
  'Design and code of round survival' vid, or 'how i learned to restrict scope and love monolithic architecture'
    auto flow state / difficulty
    constraints: dont kill yourself with fireball, like liability of king in chess
    recognition, progression
    randomness, heck it works
  subtle death animation


How is audio engine gonna work
* chunk is a pcm buf of floats
* chunks synthed however
* music is a pattern of chunks
* mixer gets told "play this immediately" and also aware of music: hopefully treated the same
  * but maybe not the same, want to play whole sound asap, would some ever get chopped off? probably not tbh
  * so some number of chunks are playing with some amount of gain, even compression or something or post processing could occur
  * lol when you're hurt music goes into dub mode
  * cpal calls the callback to get the data, how much do I provide at a time?
  * what format are my playing chunks in? ideally we can talk in samples


  * I guess mixer just does state machine stuff, im going to let per sample be the dictator
  * so like whatever sample we are up to gets to be the current sound effect
    * do we ever get interrupted by the audio thread?

  * i guess we store a list of (&buf, start_sample, gain) and when we want the next 1000 samples we compute maybe window sizes of where we do the same shit, mix them in