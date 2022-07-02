# seeded-random

This library encapsulates some of the boilerplate in making a seeded RNG easy to use and pass around a system.

## Why use a seeded RNG?

Most people who want a random number generator simply want randomly generated numbers. That's great until you need to write tests, troubleshoot your system, or realize you just wanted a purely random start point and not truly random decisions throughout an entire platform. Tasks like generating new maps for video games or noise for images and audio are good examples where all you want is a new, random start but you want what stems from there to be predictable, repeatable, and deterministic. Then, you're in a situation where you need reliably reproducible random numbers. That's when you need a seeded RNG.

## How to use

```rust
use seeded_random::Random;

fn main() {
  let rng = Random::new();

  let random_u32 = rng.u32();

  let random_string = rng.string(10);

  println!("{}, {}", random_u32, random_string);
}
```

## Seeding your RNG

You can generate new `Seed` values from any rng. This allows you to generate new, independent RNGs from a central, seeded RNG.

```rust
use seeded_random::{Random,Seed};

fn main() {
  let rng = Random::new();

  let new_seed = rng.seed();

  let rng2 = Random::from_seed(new_seed);

  assert_ne!(rng.u32(), rng2.u32());
  assert_ne!(rng.string(10), rng2.string(10));
}
```

## Manually seeding your RNG

You can create new seeds from any `u64` value with `Seed::unsafe_new()`. This is `unsafe` not because of memory reasons, but because `u64` values implement `Copy` and you can easily reuse seeds in multiple spots without recognizing what you're doing.

Using the same seed for multiple RNGs will get you two instances that generate the exact same values in the exact same order. That's not a bad thing unless you stumbled into it unexpectedly.

The `Seed` in this library does not implement `Copy` or `Clone` so it's more difficult to get into a situation where the same seed is passed around and reused mindlessly.

```rust
use seeded_random::{Random,Seed};

fn main() {
  let seed = 10;

  let seed1 = Seed::unsafe_new(seed);
  let seed2 = Seed::unsafe_new(seed);

  let rng1 = Random::from_seed(seed1);
  let rng2 = Random::from_seed(seed2);

  // As long as the same calls are made in order, the RNGs
  // will stay in sync.
  assert_eq!(rng1.u32(), rng2.u32());
  assert_eq!(rng1.uuid(), rng2.uuid());

  // When one RNG generates a single new value, it starts to deviate.
  let _ = rng1.u32();

  // Now they're generating different answers.
  assert_ne!(rng1.string(10), rng2.string(10));
}
```
