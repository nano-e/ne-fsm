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

    // Define the Response enum, which is used to handle state transitions
    pub enum Response<S> {
        Handled,
        Transition(S),
    }
    // Define the Error enum, which is used to handle errors
    #[derive(Debug)]
    pub enum Error {
        StateNotFound(String),
        StateMachineNotInitialized,
    }

    // Define the StateMachine struct, which represents the finite state machine
    pub struct StateMachine<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> {
        states: HashMap<S, Box<dyn Stateful<S, CTX, E> + Send>>,
        current_state: Option<S>,
        context: CTX,
    }

    // Implement methods for the StateMachine struct
    impl<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> StateMachine<S, CTX, E>
    where
        S: Debug + Send,
        CTX: Sized,
        E: Sized,
    {
        // Define a constructor for the StateMachine struct
        pub fn new(context: CTX) -> Self {
            let states = HashMap::<S, Box<dyn Stateful<S, CTX, E> + Send>>::new();
            Self {
                states: states,
                current_state: None,
                context: context,
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

        // Define a method to process events and transition between states
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
}
pub mod Async {
    use std::fmt::Debug;
    use std::{collections::HashMap, hash::Hash};

    use async_trait::async_trait;
    use tracing::{instrument, event, Level, trace};

    // Define the FsmEnum trait, which is used to create new state objects
    pub trait FsmEnum<S, CTX, E> {
        fn create(enum_value: &S) -> Box<dyn Stateful<S, CTX, E> + Send>;
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
        Transition(S),
    }
    // Define the Error enum, which is used to handle errors
    #[derive(Debug)]
    pub enum Error {
        StateNotFound(String),
        StateMachineNotInitialized,
    }

    // Define the StateMachine struct, which represents the finite state machine
    pub struct StateMachine<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> {
        states: HashMap<S, Box<dyn Stateful<S, CTX, E> + Send>>,
        current_state: Option<S>,
        context: CTX,
    }

    // Implement methods for the StateMachine struct
    impl<S: Hash + PartialEq + Eq + Clone + FsmEnum<S, CTX, E>, CTX, E: Debug> StateMachine<S, CTX, E>
    where
        S: Debug + Send,
        CTX: Sized,
        E: Sized,
    {
        // Define a constructor for the StateMachine struct
        pub fn new(context: CTX) -> Self {
            let states = HashMap::<S, Box<dyn Stateful<S, CTX, E> + Send>>::new();
            Self {
                states: states,
                current_state: None,
                context: context,
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
        #[instrument]
        pub async fn init(&mut self, initial_state: S) -> Result<(), Error> {
            if self.current_state.is_none() {
                self.current_state = Some(initial_state.clone());
                loop {
                    let state = self
                        .states
                        .entry(self.current_state.clone().unwrap())
                        .or_insert_with(|| S::create(&self.current_state.clone().unwrap()));                    
                    match state.on_enter(&mut self.context).await {
                        Response::Handled => break,
                        Response::Transition(s) => self.current_state = Some(s),
                    }
                }
            }
            Ok(())
        }

        // Define a method to process events and transition between states
        #[instrument]
        pub async fn process_event(&mut self, event: &E) -> Result<(), Error> {
            let c_state = match self.current_state.clone() {
                Some(state) => state,
                None => return Err(Error::StateMachineNotInitialized),
            };
            let state = self
                .states
                .entry(c_state.clone())
                .or_insert_with(|| S::create(&c_state));            
            match state.on_event(event, &mut self.context).await {
                Response::Handled => {}
                Response::Transition(new_state) => {
                    if new_state != c_state {
                        state.on_exit(&mut &mut self.context).await;
                        self.current_state = Some(new_state.clone());
                        loop {
                            let s = self
                                .states
                                .entry(self.current_state.clone().unwrap())
                                .or_insert_with(|| S::create(&self.current_state.clone().unwrap()));
                            match s.on_enter(&mut self.context).await {
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
    impl<S, CTX, E> Debug for StateMachine<S, CTX, E>
    where
        S: Debug + Send + Hash + Clone + Eq + FsmEnum<S, CTX, E>,
        CTX: Sized,
        E: Sized + Debug,
    {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("StateMachine")
                .field("current_state", &self.current_state)
                .finish()
        }
    }
}
