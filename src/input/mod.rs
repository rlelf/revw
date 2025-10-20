mod command_mode;
mod event_loop;
mod insert_mode;
mod mouse;
mod normal_mode;
mod overlay_mode;
mod search_mode;

pub use event_loop::run_app;
