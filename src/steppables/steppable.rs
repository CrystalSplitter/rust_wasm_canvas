use crate::js_bindings;
use crate::world_state::WorldState;

pub enum StepError<T> {
    /// Do nothing with the error.
    Ignore,
    /// Exit immediately and stop stepping.
    Fatal(T),
    /// Exit after stepping through every
    FatalPostStep(T),
    /// Disable this object's future steps, but keep stepping.
    SelfDisable(T),
    /// Continue stepping through, printing out an error.
    Recover(T),
}

impl<T: std::fmt::Display> std::fmt::Display for StepError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use StepError::*;
        match self {
            Ignore => write!(f, "Ignore"),
            Fatal(s) => write!(f, "Fatal({})", s),
            FatalPostStep(s) => write!(f, "FatalPostStep({})", s),
            SelfDisable(s) => write!(f, "SelfDisable({})", s),
            Recover(s) => write!(f, "Recover({})", s),
        }
    }
}

impl<T: AsRef<str> + std::fmt::Display> StepError<T> {
    pub fn translate(self) -> Result<(), String> {
        use StepError::*;
        match self {
            Ignore => Ok(()),
            Recover(s) => {
                js_bindings::warn(s.as_ref());
                Ok(())
            },
            e => Err(format!("{}", e)),
        }
    }
}

impl<T: AsRef<str> + Clone> StepError<T> {
    pub fn store_nonfatal(self, storage: &mut Self) -> Result<(), StepError<T>> {
        use StepError::*;
        match &self {
            Fatal(s) => Err(Fatal(s.clone())),
            Ignore => {
                *storage = StepError::Ignore;
                Ok(())
            }
            FatalPostStep(s) => {
                *storage = FatalPostStep(s.clone());
                Ok(())
            }
            SelfDisable(s) => {
                *storage = SelfDisable(s.clone());
                Ok(())
            }
            Recover(s) => {
                *storage = Recover(s.clone());
                Ok(())
            }
        }
    }
}

/// An object which can be stepped on the main thread.
pub trait Steppable<S>: dyn_clone::DynClone {
    fn start(&mut self, _state: &mut S) -> Result<(), StepError<String>> {
        Ok(())
    }
    // Step on the main thread at an arbitrary rate.
    fn step(&mut self, _state: &mut S) -> Result<(), StepError<String>> {
        Ok(())
    }
    // Step at a fixed rate.
    fn fixed_step(&mut self, _state: &mut S) -> Result<(), StepError<String>> {
        Ok(())
    }
    // Step only after all `step`s are complete.
    fn late_step(&mut self, _state: &mut S) -> Result<(), StepError<String>> {
        Ok(())
    }
}
dyn_clone::clone_trait_object!(Steppable<WorldState>);
