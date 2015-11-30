Gunship
=======

"The game engine for people who don't like making games."

Gunship is an experimental game engine being developed as an effort to learn
more about the inner-workings of a modern game engine. It is being written in
[Rust](http://rust-lang.org/) in an attempt to break away from game
development's unhealthy codependence on C++.

Design Goals
------------

At this point Gunship is meant to be more of a learning project than a
production-ready game engine (though of course I'd learn the most from building
an engine that's actually used in a game). As such one of the design goals of
Gunship is to do everything from scratch -- no external libraries are used to
provide even basic game-related functionality. The only exceptions that I've
made so far are the Rust standard library, since it's included by default, and
bindings to platform-specific APIs, like winapi-rs. Even then I'm working on
removing standard library from Gunship, so eventually the engine should have no
dependencies outside of the project.

As for the engine itself high-level design is pretty nonexistent. I'm keeping
development directed by building a game along with the engine, so for the time
being the design motivations for Gunship are "whatever work best for making a
game".

Current Features
----------------

The engine is very early in its development and so doesn't have much to show off
as of yet. That said, it does have a few working features at this point:

- Basic (*very* basic) 3D mesh rendering support, just basic meshes and limited
  support for custom shading. No textures, materials, shadows, real features.
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

- Proper rendering system, including support for custom surface materials and
  shaders, more efficient rendering through batching, skinned meshes, and
  particles.
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
