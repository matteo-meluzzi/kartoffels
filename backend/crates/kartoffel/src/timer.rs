use crate::{rdi, MEM_TIMER};

/// Returns a pseudorandom number that can be used as a source of randomness
/// for hashmaps and the like.
///
/// Note that this doesn't return a *new* random number each time it's called -
/// rather the number is randomized once, when the bot is being (re)started.
#[inline(always)]
pub fn timer_seed() -> u32 {
    rdi(MEM_TIMER, 0)
}

/// Returns the number of ticks that have passed since the bot's been spawned.
#[inline(always)]
pub fn timer_ticks() -> u32 {
    rdi(MEM_TIMER, 1)
}
