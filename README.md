# telescope-app
my second attempt at writing a alternative to burp suite style app in Rust. does not do much at the moment other than let you see requests at a glance. unfortunately i've been quite busy irl so there's not much done yet other than minimal core functionality.
## what it should do
* allow you to search through requests without a paid subscription lol
* easy to use modular plugins system
* epic looking gui (hard)
## bugs
* [text wrapping impossible within flow list](https://github.com/PPakalns/egui_taffy/issues/3)
## building
`telescope_core` is a dependency of `telescope_app` so you can just run `cargo build` in the root directory to build the full app. However consider reading the `README.md` of each crate since some cryptography crates have strict dependencies that are nontrivial to install.