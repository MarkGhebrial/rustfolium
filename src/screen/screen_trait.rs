use crate::screen::flushable_screen::FlushableDrawTarget;

/// The state of a single button
pub struct ButtonState {
    /// Is the button pressed?
    currently_pressed: bool,
    /// The time since the last button edge
    time_since_last_edge: embassy_time::Duration,
}

pub struct InputState {
    rev_trigger: bool,
    main_trigger: bool,
    menu_button: bool,
    
}

pub enum UpdateResult {
    Noop,
    GoBack,
}

pub trait Screen {
    fn initialize<T>(&self, screen: &mut T) -> Result<(), T::Error>
    where
        T: FlushableDrawTarget;

    fn update<T>(&self, screen: &mut T, buttons: &ButtonState) -> Result<UpdateResult, T::Error>
    where
        T: FlushableDrawTarget;
}
