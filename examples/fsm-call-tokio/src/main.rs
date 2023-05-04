// Import the async state machine module and dependencies
use nefsm::Async::{self, FsmEnum, Stateful, Response};
use tokio::sync::mpsc::{Receiver, channel, Sender};
use std::fmt::Debug;
use async_trait::async_trait;

// Define the states for the telecom call
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CallState {
    Idle,
    Dialing,
    Ringing,
    Connected,
    Disconnected,
}

// Define the events for the telecom call
#[derive(Debug)]
pub enum CallEvent {
    Dial,
    IncomingCall,
    Answer,
    Reject,
    HangUp,
    Reset,
}

// Implement the FsmEnum trait for the CallState enum
impl FsmEnum<CallState, CallContext, CallEvent> for CallState {
    fn create(enum_value: &CallState) -> Box<dyn Stateful<CallState, CallContext, CallEvent> + Send> {
        match enum_value {
            CallState::Idle => Box::new(IdleState {}),
            CallState::Dialing => Box::new(DialingState {}),
            CallState::Ringing => Box::new(RingingState {}),
            CallState::Connected => Box::new(ConnectedState {}),
            CallState::Disconnected => Box::new(DisconnectedState {}),
        }
    }
}

// Define the CallContext struct to store the number of retries
pub struct CallContext {
    pub retries: u32,
}

impl CallContext {
    pub fn new() -> Self {
        Self { retries: 0 }
    }

    pub fn increment_retries(&mut self) {
        self.retries += 1;
    }

    pub fn reset_retries(&mut self) {
        self.retries = 0;
    }
}

// Implement the Idle state
pub struct IdleState;

#[async_trait]
impl Stateful<CallState, CallContext, CallEvent> for IdleState {
    async fn on_enter(&mut self, _context: &mut CallContext) -> Response<CallState> {
        println!("Entering Idle state");
        Response::Handled
    }

    async fn on_event(&mut self, event: &CallEvent, _context: &mut CallContext) -> Response<CallState> {
        match event {
            CallEvent::Dial => Response::Transition(CallState::Dialing),
            CallEvent::IncomingCall => Response::Transition(CallState::Ringing),
            _ => {
                println!("Invalid event for Idle state");
                Response::Handled
            }
        }
    }

    async fn on_exit(&mut self, _context: &mut CallContext) {
        println!("Exiting Idle state");
    }
}

// Implement the Dialing state
pub struct DialingState;

#[async_trait]
impl Stateful<CallState, CallContext, CallEvent> for DialingState {
    async fn on_enter(&mut self, context: &mut CallContext) -> Response<CallState> {
        println!("Entering Dialing state");
        context.increment_retries();
        if context.retries <= 3 {
            Response::Handled
        } else {
            context.reset_retries();
            Response::Transition(CallState::Disconnected)
        }
    }

    async fn on_event(&mut self, event: &CallEvent, _context: &mut CallContext) -> Response<CallState> {
        match event {
            CallEvent::Answer => Response::Transition(CallState::Connected),
            CallEvent::Reject => Response::Transition(CallState::Idle),
            _ => {
                println!("Invalid event for Dialing state");
                Response::Handled
            }
        }
    }

    async fn on_exit(&mut self, _context: &mut CallContext) {
        println!("Exiting Dialing state");
    }
}

// Implement the Ringing state
pub struct RingingState;

#[async_trait]
impl Stateful<CallState, CallContext, CallEvent> for RingingState {
    async fn on_enter(&mut self, _context: &mut CallContext) -> Response<CallState> {
        println!("Entering Ringing state");
        Response::Handled
    }

    async fn on_event(&mut self, event: &CallEvent, _context: &mut CallContext) -> Response<CallState> {
        match event {
            CallEvent::Answer => Response::Transition(CallState::Connected),
            CallEvent::Reject => Response::Transition(CallState::Idle),
            _ => {
                println!("Invalid event for Ringing state");
                Response::Handled
            }
        }
    }

    async fn on_exit(&mut self, _context: &mut CallContext) {
        println!("Exiting Ringing state");
    }
}

// Implement the Connected state
pub struct ConnectedState;

#[async_trait]
impl Stateful<CallState, CallContext, CallEvent> for ConnectedState {
    async fn on_enter(&mut self, _context: &mut CallContext) -> Response<CallState> {
        println!("Entering Connected state");
        Response::Handled
    }

    async fn on_event(&mut self, event: &CallEvent, _context: &mut CallContext) -> Response<CallState> {
        match event {
            CallEvent::HangUp => Response::Transition(CallState::Disconnected),
            _ => {
               
                println!("Invalid event for Connected state");
                Response::Handled
            }
        }
    }

    async fn on_exit(&mut self, _context: &mut CallContext) {
        println!("Exiting Connected state");
    }
}

// Implement the Disconnected state
pub struct DisconnectedState;

#[async_trait]
impl Stateful<CallState, CallContext, CallEvent> for DisconnectedState {
    async fn on_enter(&mut self, _context: &mut CallContext) -> Response<CallState> {
        println!("Entering Disconnected state");
        Response::Handled
    }

    async fn on_event(&mut self, event: &CallEvent, _context: &mut CallContext) -> Response<CallState> {
        match event {
            CallEvent::Reset => Response::Transition(CallState::Idle),
            _ => {
                println!("Invalid event for Disconnected state");
                Response::Handled
            }
        }
    }

    async fn on_exit(&mut self, _context: &mut CallContext) {
        println!("Exiting Disconnected state");
    }
}

use nefsm::Async::StateMachine;

async fn event_generator(sender: Sender<CallEvent>) {
    // Generate events and send them to the receiver
    sender.send(CallEvent::Dial).await.unwrap();
    sender.send(CallEvent::Reject).await.unwrap();
    sender.send(CallEvent::Dial).await.unwrap();
    sender.send(CallEvent::Answer).await.unwrap();
    sender.send(CallEvent::HangUp).await.unwrap();
}

async fn event_receiver(
    mut call_state_machine: StateMachine<CallState, CallContext, CallEvent>,
    mut receiver: Receiver<CallEvent>,
) {
    
    // Process events received from the event_generator
    while let Some(event) = receiver.recv().await {
        call_state_machine.process_event(&event).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    // Initialize the state machine
    let mut call_state_machine = StateMachine::new(
        CallState::Idle,
        CallContext {
            retries: 0,
        },
        None,
    );
    call_state_machine.init().await;

    // Create a Tokio channel for sending and receiving events
    let (sender, receiver) = channel(100);

    // Spawn two Tokio tasks: one for generating events and one for processing them
    let event_generator_handle = tokio::spawn(event_generator(sender));
    let event_receiver_handle = tokio::spawn(event_receiver(call_state_machine, receiver));

    // Wait for both tasks to complete
    event_generator_handle.await.unwrap();
    event_receiver_handle.await.unwrap();
}