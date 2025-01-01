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
        current_state: Option<S>,
        context: CTX,
        global_event_handler: Option<Box<dyn EventHandler<S, CTX, E> + Send>>,
    }

    // Implement methods for the StateMachine struct
    impl<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> StateMachine<S, CTX, E> {
        // Define a constructor for the StateMachine struct
        pub fn new(context: CTX, handler: Option<Box<dyn EventHandler<S, CTX, E> + Send>>) -> Self {
            let states = HashMap::<S, Box<dyn Stateful<S, CTX, E> + Send>>::new();
            Self {
                states,
                current_state: None,
                context,
                global_event_handler: handler,
            }
        }

        // Define a method to get the current state
        pub fn get_current_state(&self) -> Option<&S> {
            self.current_state.as_ref()
        }

        // Define a method to get a reference to the context
        pub fn get_context(&self) -> &CTX {
            &self.context
        }

        // Define a method to initialize the state machine with an initial state
        // Note how the state objects are cached in a HashMap and not recreated every time we transition to this event.
        pub fn init(&mut self, initial_state: S) -> Result<(), Error> {
            if self.current_state.is_none() {
                let mut next_state = Some(initial_state);
                // TODO: maybe CTX should implement Clone to prevent side effects (clone self.context here and set later, according to state)
                loop {
                    let current_state_ref = next_state.as_ref().unwrap();
                    let state = if let Some(existing_state) = self.states.get_mut(current_state_ref)
                    {
                        existing_state
                    } else {
                        let new_state = S::create(current_state_ref);
                        let current_state_clone = next_state.clone().unwrap();
                        self.states.entry(current_state_clone).or_insert(new_state)
                    };

                    match state.on_enter(&mut self.context) {
                        Response::Handled => break,
                        Response::Error(e) => return Err(Error::StateInvalid(e)),
                        Response::Transition(s) => next_state = Some(s),
                    }
                }
                self.current_state = next_state;
            }
            Ok(())
        }

        // Define a method to process events and transition between states
        pub fn process_event(&mut self, event: &E) -> Result<(), Error> {
            let c_state = match &self.current_state {
                Some(state) => state,
                None => return Err(Error::StateMachineNotInitialized),
            };

            if let Some(global_handler) = &mut self.global_event_handler {
                match global_handler.on_event(event, &mut self.context) {
                    Response::Handled => {}
                    Response::Error(s) => return Err(Error::InvalidEvent(s)),
                    Response::Transition(new_state) => {
                        if new_state != *c_state {
                            return self.transition_to(new_state);
                        }
                    }
                }
            }

            let current_state_ref = self.current_state.as_ref().unwrap();
            let state = if let Some(existing_state) = self.states.get_mut(current_state_ref) {
                existing_state
            } else {
                let new_state = S::create(current_state_ref);
                let current_state_clone = self.current_state.clone().unwrap();
                self.states.entry(current_state_clone).or_insert(new_state)
            };
            match state.on_event(event, &mut self.context) {
                Response::Handled => Ok(()),
                Response::Error(s) => Err(Error::InvalidEvent(s)),
                Response::Transition(new_state) => {
                    if new_state != *c_state {
                        self.transition_to(new_state)?;
                    }
                    Ok(())
                }
            }
        }

        // Define a method to handle state transitions
        fn transition_to(&mut self, new_state: S) -> Result<(), Error> {
            let c_state = self.current_state.as_ref().unwrap();
            let state = self.states.get_mut(&c_state).unwrap();
            state.on_exit(&mut self.context);

            let mut next_state = Some(new_state.clone());
            loop {
                let current_state_ref = next_state.as_ref().unwrap();
                let s = if let Some(existing_state) = self.states.get_mut(current_state_ref) {
                    existing_state
                } else {
                    let new_state = S::create(current_state_ref);
                    let current_state_clone = next_state.clone().unwrap();
                    self.states.entry(current_state_clone).or_insert(new_state)
                };
                match s.on_enter(&mut self.context) {
                    Response::Handled => {
                        break;
                    }
                    Response::Error(e) => return Err(Error::StateInvalid(e)),
                    Response::Transition(s) => {
                        if s == *next_state.as_ref().unwrap() {
                            break;
                        } else {
                            next_state = Some(s);
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
        current_state: Option<S>,
        context: CTX,
        global_event_handler: Option<Box<dyn EventHandler<S, CTX, E> + Send>>,
    }

    // Implement methods for the StateMachine struct
    impl<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> StateMachine<S, CTX, E> {
        // Define a constructor for the StateMachine struct
        pub fn new(
            context: CTX,
            global_handler: Option<Box<dyn EventHandler<S, CTX, E> + Send>>,
        ) -> Self {
            Self {
                states: HashMap::new(),
                current_state: None,
                context,
                global_event_handler: global_handler,
            }
        }

        // Define a method to get the current state
        pub fn get_current_state(&self) -> Option<&S> {
            self.current_state.as_ref()
        }

        // Define a method to get a reference to the context
        pub fn get_context(&self) -> &CTX {
            &self.context
        }

        // Define a method to initialize the state machine with an initial state
        pub async fn init(&mut self, initial_state: S) -> Result<(), Error> {
            if self.current_state.is_none() {
                let mut next_state = Some(initial_state);
                // TODO: maybe CTX should implement Clone to prevent side effects (clone self.context here and set later, according to state)
                loop {
                    let current_state_ref = next_state.as_ref().unwrap();
                    let state = if let Some(existing_state) = self.states.get_mut(current_state_ref)
                    {
                        existing_state
                    } else {
                        let new_state = S::create(current_state_ref);
                        let current_state_clone = next_state.clone().unwrap();
                        self.states.entry(current_state_clone).or_insert(new_state)
                    };

                    match state.on_enter(&mut self.context).await {
                        Response::Handled => break,
                        Response::Error(e) => return Err(Error::StateInvalid(e)),
                        Response::Transition(s) => next_state = Some(s),
                    }
                }
                self.current_state = next_state;
            }
            Ok(())
        }
        // Define a method to process events and transition between states
        pub async fn process_event(&mut self, event: &E) -> Result<(), Error> {
            let c_state = match &self.current_state {
                Some(state) => state,
                None => return Err(Error::StateMachineNotInitialized),
            };

            if let Some(ref mut handler) = self.global_event_handler {
                match handler.on_event(event, &mut self.context).await {
                    Response::Handled => {}
                    Response::Error(s) => return Err(Error::InvalidEvent(s)),
                    Response::Transition(new_state) => {
                        if new_state != *c_state {
                            self.transition_to(new_state).await;
                            return Ok(());
                        }
                    }
                }
            }

            let current_state_ref = self.current_state.as_ref().unwrap();
            let state = if let Some(existing_state) = self.states.get_mut(current_state_ref) {
                existing_state
            } else {
                let new_state = S::create(current_state_ref);
                let current_state_clone = self.current_state.clone().unwrap();
                self.states.entry(current_state_clone).or_insert(new_state)
            };

            match state.on_event(event, &mut self.context).await {
                Response::Handled => Ok(()),
                Response::Error(s) => Err(Error::InvalidEvent(s)),
                Response::Transition(new_state) => {
                    if new_state != *c_state {
                        return self.transition_to(new_state).await;
                    }
                    Ok(())
                }
            }
        }

        async fn transition_to(&mut self, new_state: S) -> Result<(), Error> {
            let c_state = self.current_state.as_ref().unwrap();
            let state = self.states.get_mut(&c_state).unwrap();
            state.on_exit(&mut self.context).await;

            let mut next_state = Some(new_state.clone());
            loop {
                let current_state_ref = next_state.as_ref().unwrap();
                let s = if let Some(existing_state) = self.states.get_mut(current_state_ref) {
                    existing_state
                } else {
                    let new_state = S::create(current_state_ref);
                    let current_state_clone = next_state.clone().unwrap();
                    self.states.entry(current_state_clone).or_insert(new_state)
                };

                match s.on_enter(&mut self.context).await {
                    Response::Handled => {
                        break;
                    }
                    Response::Error(e) => return Err(Error::StateInvalid(e)),
                    Response::Transition(s) => {
                        if s == *current_state_ref {
                            break;
                        } else {
                            next_state = Some(s);
                        }
                    }
                }
            }

            self.current_state = next_state;

            Ok(())
        }
    }
}
