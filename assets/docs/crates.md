A brief description of each crate in this repo and its purpose:

* `ootr` provides compile-time-static information about the randomizer.
* `ootr-derive` is a proc-macro crate generating the code for `ootr`. It uses `pyo3` and the randomizer code, but since it's a proc-macro crate, this only adds compile-time dependencies, not runtime ones.
* `oottracker` contains the main library for the tracker's logic, as well as some types that aren't directly pulled from the randomizer.
* `oottracker-bizhawk` contains the C# code for the BizHawk auto-tracking plugin. There is no Rust code here except for a build script that calls the appropriate C# build tools.
* `oottracker-csharp` contains FFI bindings to types from `oottracker` which are used by `oottracker-bizhawk`. While it is currently written in typical C FFI style, it may be modified in the future to take better advantage of C#'s safety and async features.
* `oottracker-derive` contains proc macros for the `oottracker` crate.
* `oottracker-gui` is a cross-platform graphical frontend for the tracker written using `iced`.
* `oottracker-utils` contains debugging and release scripts for the project.
* `oottracker-web` is a web frontend for the tracker with support for networked tracking, live at <https://oottracker.fenhl.net/>.
