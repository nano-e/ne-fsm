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
#[derive(Debug)]
pub enum Error {
    StateNotFound(String),
    StateMachineNotInitialized,
}
pub struct StateMachine<
    S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E> + ToString,
    CTX,
    E: Debug,
> {
    states: HashMap<S, Box<dyn Stateful<S, CTX, E>>>,
    current_state: Option<S>,
    context: CTX,
}
impl<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E> + ToString, CTX, E: Debug>
    StateMachine<S, CTX, E>
where
    S: Debug,
    CTX: Sized,
    E: Sized,
{
    pub fn new(context: CTX) -> Self {
        let mut states = HashMap::<S, Box<dyn Stateful<S, CTX, E>>>::new();
        Self {
            states: states,
            current_state: None,
            context: context,
        }
    }

    pub fn init(&mut self, initial_state: S) -> Result<(), Error> {
        if self.current_state.is_none() {
            self.current_state = Some(initial_state.clone());
            loop {                
                let state = self
                .states
                .entry(self.current_state.clone().unwrap())
                .or_insert_with(|| S::create(&self.current_state.clone().unwrap()));

                match state.on_enter(&mut self.context) {
                    Response::Handled => break,
                    Response::Transition(s) => self.current_state = Some(s),
                }
            }
        }
        Ok(())
    }
    pub fn process_event(&mut self, event: &E) -> Result<(), Error> {
        let c_state = match self.current_state.clone() {
            Some(state) => state,
            None => return Err(Error::StateMachineNotInitialized),
        };

        let state = self
            .states
            .entry(c_state.clone())
            .or_insert_with(|| S::create(&c_state));
        match state.on_event(event, &mut self.context) {
            Response::Handled => {}
            Response::Transition(new_state) => {
                if new_state != c_state {
                    state.on_exit(&mut &mut self.context);
                    self.current_state = Some(new_state.clone());
                    loop {
                        let s = self
                            .states
                            .entry(self.current_state.clone().unwrap())
                            .or_insert_with(|| S::create(&self.current_state.clone().unwrap()));
                        match s.on_enter(&mut self.context) {
                            Response::Handled => {
                                break;
                            }
                            Response::Transition(s) => {
                                if s == self.current_state.clone().unwrap() {
                                    break;
                                } else {                                
                                    self.current_state = Some(s);
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
