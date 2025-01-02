use std::time::Duration;

use async_trait::async_trait;
use nefsm::Async::{EventHandler, FsmEnum, Response, StateMachine, Stateful};
use rand::Rng;
use tokio::{sync::mpsc, task, time};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum State {
    StateA,
    StateB,
    StateC,
}

#[derive(Debug)]
pub struct Payload {
    f1: u16,
}

#[derive(Debug)]
pub enum Event {
    E1,
    E2(Payload),
    E3,
    E4,
}

struct StateA {}
struct StateB {}

struct StateC {}
#[async_trait]
impl Stateful<State, Context, Event> for StateA {
    async fn on_enter(&mut self, context: &mut Context) -> Response<State> {
        context.retries = context.retries + 1;
        Response::Handled
    }

    async fn on_event(&mut self, event: &Event, context: &mut Context) -> Response<State> {
        match event {
            Event::E1 => Response::Transition(State::StateB),
            _ => Response::Transition(State::StateC),
        }
    }

    async fn on_exit(&mut self, _context: &mut Context) {
        // Add any necessary code for when the state is exited
    }
}

#[async_trait]
impl Stateful<State, Context, Event> for StateB {
    async fn on_enter(&mut self, context: &mut Context) -> Response<State> {
        context.retries = context.retries - 1;
        Response::Handled
    }

    async fn on_event(&mut self, event: &Event, context: &mut Context) -> Response<State> {
        match event {
            Event::E1 => Response::Transition(State::StateC),
            _ => Response::Transition(State::StateA),
        }
    }

    async fn on_exit(&mut self, _context: &mut Context) {
        // Add any necessary code for when the state is exited
    }
}

#[async_trait]
impl Stateful<State, Context, Event> for StateC {
    async fn on_enter(&mut self, context: &mut Context) -> Response<State> {
        context.retries = context.retries + 2;
        Response::Handled
    }

    async fn on_event(&mut self, event: &Event, context: &mut Context) -> Response<State> {
        match event {
            Event::E1 => Response::Transition(State::StateA),
            _ => Response::Transition(State::StateB),
        }
    }

    async fn on_exit(&mut self, _context: &mut Context) {
        // Add any necessary code for when the state is exited
    }
}

impl Event {
    pub fn random() -> Event {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..4) {
            0 => Event::E1,
            1 => Event::E2(Payload {
                f1: rng.gen_range(0..100),
            }),
            2 => Event::E3,
            3 => Event::E4,
            _ => panic!("Invalid event"),
        }
    }
}

struct GlobalEventHandler;

#[async_trait]
impl EventHandler<State, Context, Event> for GlobalEventHandler {
    async fn on_event(&mut self, event: &Event, context: &mut Context) -> Response<State> {
        match event {
            Event::E4 => {
                println!("Global event handler: E4 received");
                Response::Handled
            }
            _ => random_transition(),
        }
    }
}

fn random_transition() -> Response<State> {
    let mut rng = rand::thread_rng();
    let random_value: f64 = rng.gen();

    if random_value < 0.5 {
        Response::Handled
    } else {
        let next_state = match rng.gen_range(0..3) {
            0 => State::StateA,
            1 => State::StateB,
            2 => State::StateC,
            _ => panic!("Invalid state"),
        };

        Response::Transition(next_state)
    }
}

#[derive(Debug)]
pub struct Context {
    retries: u32,
}

impl FsmEnum<State, Context, Event> for State {
    fn create(enum_value: &State) -> Box<dyn Stateful<State, Context, Event> + Send> {
        match enum_value {
            State::StateA => Box::new(StateA {}),
            State::StateB => Box::new(StateB {}),
            State::StateC => Box::new(StateC {}),
        }
    }
}

// Rest of the state and event handling code remains unchanged

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Event>(10);
    let producer = task::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(500));
        let tx2 = tx.clone();
        loop {
            interval.tick().await;
            let event: Event = Event::random();
            match tx2.send(event).await {
                Ok(_) => {}
                Err(e) => {
                    println!("error sending event {}", e);
                }
            }
        }
    });

    let mut state_machine = StateMachine::<State, Context, Event>::new(
        State::StateA,
        Context { retries: 0 },
        Some(Box::new(GlobalEventHandler)),
    )
    .await
    .unwrap();

    let consumer = task::spawn(async move {
        while let Some(message) = rx.recv().await {
            println!(
                "current state: {:?} - event: {:?} - context: {:?}",
                state_machine.get_current_state(),
                message,
                state_machine.get_context()
            );
            state_machine.process_event(&message).await;
        }
    });

    producer.await.unwrap();
    consumer.await.unwrap();
}
