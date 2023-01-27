use std::borrow::Borrow;
use std::fmt::Debug;
use std::{collections::HashMap, hash::Hash};

pub trait FsmEnum<S, CTX, E> {
    fn create(enum_value: &S) -> Box<dyn Stateful<S, CTX, E>>;
}

pub trait Stateful<S: Hash + PartialEq + Eq + Clone, CTX, E: Debug> {
    fn on_enter(&mut self, context: &mut CTX) -> Response<S>;
    fn on_event(&mut self, event: &E, context: &mut CTX) -> Response<S>;
    fn on_exit(&mut self, context: &mut CTX);
}

pub enum Response<S> {
    Handled,
    Transition(S),
}
pub enum Error {
    StateNotFound(String),
}
pub struct StateMachine<
    S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E> + ToString,
    CTX,
    E: Debug,
> {
    states: HashMap<S, Box<dyn Stateful<S, CTX, E>>>,
    current_state: S,
    context: CTX,
}
impl<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E> + ToString, CTX, E: Debug>
    StateMachine<S, CTX, E>
where
    S: Debug,
    CTX: Sized,
    E: Sized,
{
    pub fn new(initial_state: S, context: CTX, cache_state_objects: bool) -> Self {
        let mut states = HashMap::<S, Box<dyn Stateful<S, CTX, E>>>::new();
        Self {
            states: states,
            current_state: initial_state,
            context: context,
        }
    }
    pub fn add_state(&mut self, s: S, state: Box<dyn Stateful<S, CTX, E>>) {
        self.states.insert(s, state);
    }

    pub fn process_event(&mut self, event: &E) -> Result<(), Error> {
        let state = self
            .states
            .entry(self.current_state.clone())
            .or_insert_with(|| S::create(&self.current_state));
        match state.on_event(event, &mut self.context) {
            Response::Handled => {}
            Response::Transition(s) => {
                if s != self.current_state {
                    state.on_exit(&mut &mut self.context);
                    self.current_state = s;
                    loop {
                        let s = self
                            .states
                            .entry(self.current_state.clone())
                            .or_insert_with(|| S::create(&self.current_state));
                        match s.on_enter(&mut self.context) {
                            Response::Handled => {
                                break;
                            }
                            Response::Transition(s) => {
                                if s == self.current_state {
                                    break;
                                } else {
                                    self.current_state = s;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
