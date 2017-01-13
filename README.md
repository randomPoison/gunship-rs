Gunship
=======

"The game engine for people who don't like making games."

Gunship is an experimental game engine being developed as an effort to learn
more about the inner-workings of a modern game engine. It is being written in
[Rust](http://rust-lang.org/) in an attempt to break away from the game
industry's unhealthy codependence on C++.

Design Goals
------------

At this point Gunship is meant to be more of a learning project than a
production-ready game engine. The idea is to build as much as possible from
scratch in order to develop myself as a programmer and learn all the parts
of a modern game engine. So far I've allowed myself to use the standard library
and libraries for binding to platform-specific native libraries, but the
intent is to keep dependencies to a minimum, even if there are relevant crates
in the Rust ecosystem.

As for the engine itself high-level design is pretty nonexistent. I'm keeping
development directed by building a game along with the engine, so for the time
being the design motivations for Gunship are "whatever work best for making a
game". At present the design is roughly as follows:

- The renderer is a separate crate from the engine core. It's potentially going
  to be usable on its own, though right now that's not a primary goal. Mostly
  the renderer's goal is to provide a high level, backend agnostic system for
  rendering that can be tested idependently but easily be plugged into the
  engine core.
- The engine core provides the scheduler, which makes it easy to write highly
  parallel, asynchronous gameplay and engine code.
- The engine core only provides the primitives for gameplay development (e.g.
  transforms, lights, meshes, etc.), but provides little-to-no framework for
  structuring gameplay code. Such frameworks will likely be provided
  as a layer on top of the engine core.

Current Features
----------------

The engine is very early in its development and so doesn't have much to show off
as of yet. That said, it does have a few working features at this point:

- Multi-threaded, async engine and gameplay code makes it easy to write games
  while taking full advantage of multiple cores in modern CPUs.
- Basic (*very* basic) 3D mesh rendering support, just basic meshes and limited
  support for custom shading. No shadows, highly inefficient.

Broken Features
---------------

The following features have been implemented in the past but were broken at some
point:

- 3D collision processing with sphere and box colliders. Actually pretty fast
  compared to the other collision systems, though very much lacking in
  functionality.
- Code hotloading, which allows you to recompile code without stopping and
  starting the game. Currently very hacky in its implementation, but very much
  functional.
- Super basic audio -- only one audio source at a time and pretty much
  hard-coded to work on my machine.

Planned Features
----------------

Beyond the obvious (making the existing systems more robust and actually
*usable*) there are a few features that I plan to work on in the near future:

- Proper rendering system, including more efficient rendering through batching,
  skinned meshes, and particles.
- [Cross-platform support](https://github.com/excaliburHisSheath/gunship-rs/milestones/Basic%20Cross-Platform%20Support)
  for Linux and OSX at least, eventually I'd like to get to mobile support and
  consoles, though who knows how long that could take.
- [Proper logging system](https://github.com/excaliburHisSheath/gunship-rs/issues/21).

Contributing
------------

At this point the engine's not stable enough for anyone but me to attempt to use
it to make a game -- if you want to use Rust to make a game then check out
[Piston](http://www.piston.rs/). On the other hand if what you want is to build
some engine subsystem from scratch but don't want to have to put together all
of the other pieces, feel free to use Gunship as your baseline. Even then I
don't recommend it, but I'm happy to help however I can (just send an email or
open an issue to ask questions).
