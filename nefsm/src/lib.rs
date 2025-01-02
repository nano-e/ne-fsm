//! A simple state machine library for Rust, using enums for states and events.
//!
//! This library provides a simple, flexible way to define state machines in Rust. The states
//! and events are defined using Rust enums, and the state machine itself is a generic struct
//! that can be instantiated with any specific set of enums.
//!
//! The core traits used in this library are `FsmEnum`, and `Stateful`.
//!
//! * `FsmEnum` is a trait that defines how to create a new state machine state based on a given
//!   enum value. This is used to instantiate new state machine states when a state transition occurs.
//!
//! * `Stateful` is a trait that defines how a state should handle state transition events.
//!
//! * `StateMachine` is the main struct that represents a state machine instance. It tracks the
//!   current state, and provides methods to initialize the state machine, process events, and get
//!   the current state.
//!
//! This library is designed to be easy to use and flexible enough to handle a wide variety of
//! state machine designs.
//!
//!

pub mod sync {
    use std::fmt::Debug;
    use std::{collections::HashMap, hash::Hash};

    // Define the FsmEnum trait, which is used to create new state objects
    pub trait FsmEnum<S, CTX, E> {
        fn create(enum_value: &S) -> Box<dyn Stateful<S, CTX, E> + Send>;
    }

    // Define the Stateful trait, which contains the event handling methods for each state
    pub trait Stateful<S: Hash + PartialEq + Eq + Clone, CTX, E: Debug> {
        fn on_enter(&mut self, context: &mut CTX) -> Response<S>;
        fn on_event(&mut self, event: &E, context: &mut CTX) -> Response<S>;
        fn on_exit(&mut self, context: &mut CTX);
    }

    // Define the EventHandler trait, which is used to handle global events
    pub trait EventHandler<S: Hash + PartialEq + Eq + Clone, CTX, E: Debug> {
        fn on_event(&mut self, event: &E, context: &mut CTX) -> Response<S>;
    }

    // Define the Response enum, which is used to handle state transitions
    pub enum Response<S> {
        Handled,
        Error(String),
        Transition(S),
    }

    // Define the Error enum, which is used to handle errors
    #[derive(Debug)]
    pub enum Error {
        StateNotFound(String),
        StateInvalid(String),
        InvalidEvent(String),
        StateMachineNotInitialized,
        InternalError(String),
    }

    // Define the StateMachine struct, which represents the finite state machine
    pub struct StateMachine<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> {
        states: HashMap<S, Box<dyn Stateful<S, CTX, E> + Send>>,
        current_state: S,
        context: CTX,
        global_event_handler: Option<Box<dyn EventHandler<S, CTX, E> + Send>>,
    }

    // Implement methods for the StateMachine struct
    impl<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> StateMachine<S, CTX, E> {
        // Define a constructor for the StateMachine struct
        pub fn new(
            mut initial_state: S,
            mut context: CTX,
            handler: Option<Box<dyn EventHandler<S, CTX, E> + Send>>,
        ) -> Result<Self, Error> {
            let mut states = HashMap::<S, Box<dyn Stateful<S, CTX, E> + Send>>::new();

            loop {
                let state = if let Some(existing_state) = states.get_mut(&initial_state) {
                    existing_state
                } else {
                    let new_state = S::create(&initial_state);
                    let current_state_clone = initial_state.clone();
                    states.entry(current_state_clone).or_insert(new_state)
                };

                // TODO: maybe CTX should implement Clone to prevent side effects
                // (clone context here and set later, if no Error is returned)
                match state.on_enter(&mut context) {
                    Response::Handled => break,
                    Response::Error(e) => return Err(Error::StateInvalid(e)),
                    Response::Transition(s) => initial_state = s,
                }
            }

            Ok(Self {
                states,
                current_state: initial_state,
                context,
                global_event_handler: handler,
            })
        }

        // Define a method to get the current state
        pub fn get_current_state(&self) -> &S {
            &self.current_state
        }

        // Define a method to get a reference to the context
        pub fn get_context(&self) -> &CTX {
            &self.context
        }

        // Define a method to process events and transition between states
        pub fn process_event(&mut self, event: &E) -> Result<(), Error> {
            if let Some(global_handler) = &mut self.global_event_handler {
                match global_handler.on_event(event, &mut self.context) {
                    Response::Handled => {}
                    Response::Error(s) => return Err(Error::InvalidEvent(s)),
                    Response::Transition(new_state) => {
                        if new_state != self.current_state {
                            return self.transition_to(new_state);
                        }
                    }
                }
            }

            let state = if let Some(existing_state) = self.states.get_mut(&self.current_state) {
                existing_state
            } else {
                let new_state = S::create(&self.current_state);
                let current_state_clone = self.current_state.clone();
                self.states.entry(current_state_clone).or_insert(new_state)
            };
            match state.on_event(event, &mut self.context) {
                Response::Handled => Ok(()),
                Response::Error(s) => Err(Error::InvalidEvent(s)),
                Response::Transition(new_state) => {
                    if new_state != self.current_state {
                        self.transition_to(new_state)?;
                    }
                    Ok(())
                }
            }
        }

        // Define a method to handle state transitions
        fn transition_to(&mut self, new_state: S) -> Result<(), Error> {
            let state = self.states.get_mut(&self.current_state).unwrap();
            state.on_exit(&mut self.context);

            let mut next_state = new_state.clone();
            loop {
                let s = if let Some(existing_state) = self.states.get_mut(&next_state) {
                    existing_state
                } else {
                    let new_state = S::create(&next_state);
                    let current_state_clone = next_state.clone();
                    self.states.entry(current_state_clone).or_insert(new_state)
                };
                match s.on_enter(&mut self.context) {
                    Response::Handled => {
                        break;
                    }
                    Response::Error(e) => return Err(Error::StateInvalid(e)),
                    Response::Transition(s) => {
                        if s == next_state {
                            break;
                        } else {
                            next_state = s;
                        }
                    }
                }
            }

            self.current_state = next_state;

            Ok(())
        }
    }
}
pub mod Async {
    use std::fmt::Debug;
    use std::{collections::HashMap, hash::Hash};

    use async_trait::async_trait;

    // Define the FsmEnum trait, which is used to create new state objects
    pub trait FsmEnum<S, CTX, E> {
        fn create(enum_value: &S) -> Box<dyn Stateful<S, CTX, E> + Send>;
    }

    // Define the EventHandler trait for handling global events
    #[async_trait]
    pub trait EventHandler<S: Hash + PartialEq + Eq + Clone, CTX, E: Debug> {
        async fn on_event(&mut self, event: &E, context: &mut CTX) -> Response<S>;
    }

    // Define the Stateful trait, which contains the event handling methods for each state
    #[async_trait]
    pub trait Stateful<S: Hash + PartialEq + Eq + Clone, CTX, E: Debug> {
        async fn on_enter(&mut self, context: &mut CTX) -> Response<S>;
        async fn on_event(&mut self, event: &E, context: &mut CTX) -> Response<S>;
        async fn on_exit(&mut self, context: &mut CTX);
    }

    // Define the Response enum, which is used to handle state transitions
    pub enum Response<S> {
        Handled,
        Error(String),
        Transition(S),
    }

    // Define the Error enum, which is used to handle errors
    #[derive(Debug)]
    pub enum Error {
        StateNotFound(String),
        StateInvalid(String),
        InvalidEvent(String),
        StateMachineNotInitialized,
        InternalError(String),
    }

    // Define the StateMachine struct, which represents the finite state machine
    pub struct StateMachine<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> {
        states: HashMap<S, Box<dyn Stateful<S, CTX, E> + Send>>,
        current_state: S,
        context: CTX,
        global_event_handler: Option<Box<dyn EventHandler<S, CTX, E> + Send>>,
    }

    // Implement methods for the StateMachine struct
    impl<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> StateMachine<S, CTX, E> {
        // Define a constructor for the StateMachine struct
        pub async fn new(
            mut initial_state: S,
            mut context: CTX,
            handler: Option<Box<dyn EventHandler<S, CTX, E> + Send>>,
        ) -> Result<Self, Error> {
            let mut states = HashMap::<S, Box<dyn Stateful<S, CTX, E> + Send>>::new();

            loop {
                let state = if let Some(existing_state) = states.get_mut(&initial_state) {
                    existing_state
                } else {
                    let new_state = S::create(&initial_state);
                    let current_state_clone = initial_state.clone();
                    states.entry(current_state_clone).or_insert(new_state)
                };

                // TODO: maybe CTX should implement Clone to prevent side effects
                // (clone context here and set later, if no Error is returned)
                match state.on_enter(&mut context).await {
                    Response::Handled => break,
                    Response::Error(e) => return Err(Error::StateInvalid(e)),
                    Response::Transition(s) => initial_state = s,
                }
            }

            Ok(Self {
                states,
                current_state: initial_state,
                context,
                global_event_handler: handler,
            })
        }

        // Define a method to get the current state
        pub fn get_current_state(&self) -> &S {
            &self.current_state
        }

        // Define a method to get a reference to the context
        pub fn get_context(&self) -> &CTX {
            &self.context
        }

        // Define a method to process events and transition between states
        pub async fn process_event(&mut self, event: &E) -> Result<(), Error> {
            if let Some(ref mut global_handler) = self.global_event_handler {
                match global_handler.on_event(event, &mut self.context).await {
                    Response::Handled => {}
                    Response::Error(s) => return Err(Error::InvalidEvent(s)),
                    Response::Transition(new_state) => {
                        if new_state != self.current_state {
                            self.transition_to(new_state).await;
                            return Ok(());
                        }
                    }
                }
            }

            let state = if let Some(existing_state) = self.states.get_mut(&self.current_state) {
                existing_state
            } else {
                let new_state = S::create(&self.current_state);
                let current_state_clone = self.current_state.clone();
                self.states.entry(current_state_clone).or_insert(new_state)
            };

            match state.on_event(event, &mut self.context).await {
                Response::Handled => Ok(()),
                Response::Error(s) => Err(Error::InvalidEvent(s)),
                Response::Transition(new_state) => {
                    if new_state != self.current_state {
                        return self.transition_to(new_state).await;
                    }
                    Ok(())
                }
            }
        }

        async fn transition_to(&mut self, new_state: S) -> Result<(), Error> {
            let state = self.states.get_mut(&self.current_state).unwrap();
            state.on_exit(&mut self.context).await;

            let mut next_state = new_state.clone();
            loop {
                let s = if let Some(existing_state) = self.states.get_mut(&next_state) {
                    existing_state
                } else {
                    let new_state = S::create(&next_state);
                    let current_state_clone = next_state.clone();
                    self.states.entry(current_state_clone).or_insert(new_state)
                };

                match s.on_enter(&mut self.context).await {
                    Response::Handled => {
                        break;
                    }
                    Response::Error(e) => return Err(Error::StateInvalid(e)),
                    Response::Transition(s) => {
                        if s == next_state {
                            break;
                        } else {
                            next_state = s;
                        }
                    }
                }
            }

            self.current_state = next_state;

            Ok(())
        }
    }
}
