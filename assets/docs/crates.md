A brief description of each crate in this repo and its purpose:

* `ootr` defines the `Rando` trait, whose methods provide access to the randomizer's code and data.
* `ootr-dynamic` implements the `ootr` crate's `Rando` trait by importing the Python modules from a given path using `pyo3`. It should be linked to dynamically using `libloading` if the user specifies a custom randomizer path, so that Python is only required at runtime if it's used.
* `ootr-static` implements the `ootr` crate's `Rando` trait with static data generated at compile time, without adding a runtime Python dependency.
* `ootr-static-derive` is a proc-macro crate generating the code for `ootr-static`'s `Rando` implementation. It uses `ootr-dynamic`, but since it's a proc-macro crate, this only adds a compile-time Python dependency, not a runtime one.
* `oottracker` contains the main library for the tracker's logic, as well as some types that aren't directly pulled from the randomizer.
* `oottracker-bizhawk` contains the C# code for the BizHawk auto-tracking plugin. There is no Rust code here except for a build script that calls the appropriate C# build tools.
* `oottracker-csharp` contains FFI bindings to types from `oottracker` which are used by `oottracker-bizhawk`. While it is currently written in typical C FFI style, it may be modified in the future to take better advantage of C#'s safety and async features.
* `oottracker-derive` contains proc macros for the `oottracker` crate.
* `oottracker-gui` is a cross-platform graphical frontend for the tracker written using `iced`.
* `oottracker-utils` contains debugging and release scripts for the project.
