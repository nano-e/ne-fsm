
use std::{time::Duration};
use strum::{EnumCount, IntoEnumIterator};
use strum_macros::{EnumCount as EnumCountMacro, EnumIter};


use rand::{prelude::Distribution, distributions::Standard, Rng};
use tokio::{sync::{oneshot, mpsc}, time, task};
use nefsm::{FsmEnum, Stateful, StateMachine};

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum State {
    StateA,
    StateB,
    StateC,        
}

#[derive(Debug)]
pub struct Payload {
    f1: u16
}

#[derive(Debug)]
pub enum Event<'a> {
    E1,
    E2(&'a Payload),
    E3,
    E4,
}

struct StateA {}
struct StateB {}

struct StateC{}

impl <'a> FsmEnum<State, Context, Event<'a>> for State {
    fn create(enum_value: &State) -> Box<dyn Stateful<State, Context, Event<'a>> + Send> {
        match enum_value {
            State::StateA => Box::new(StateA {}),
            State::StateB => Box::new(StateB {}),
            State::StateC => Box::new(StateC {}),
        }
    }
}

impl <'a>Stateful<State, Context, Event<'a>> for StateA {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::Response<State> {        
        context.retries = context.retries + 1;
        nefsm::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::Response<State> {
        match event{
            Event::E1 => nefsm::Response::Transition(State::StateB),
            _ => nefsm::Response::Transition(State::StateC),
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        
    }
}
impl <'a>Stateful<State, Context, Event<'a>> for StateB {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::Response<State> {
        context.retries = context.retries - 1;
        nefsm::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::Response<State> {
        match event{
            Event::E1 => nefsm::Response::Transition(State::StateC),
            _ => nefsm::Response::Transition(State::StateA),
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        
    }
}
impl <'a>Stateful<State, Context, Event<'a>> for StateC {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::Response<State> {
        context.retries = context.retries + 2;
        nefsm::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::Response<State> {
        match event{
            Event::E1 => nefsm::Response::Transition(State::StateA),
            _ => nefsm::Response::Transition(State::StateB),
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        
    }
}

#[derive(Debug)]
pub struct Context {
    retries: u32
}
// impl Event {
//     pub fn random() -> Event {

        
//         Event::E1
//     }
// }

// impl <'a>FsmEnum<State, Context, Event<'a>> for State {
//     fn create(enum_value: &State) -> Box<dyn Stateful<State, Context, Event<'a>> + Send> {
//         match enum_value {
//             State::StateA => Box::new(StateA{}),
//             State::StateB => Box::new(StateB{}),
//             State::StateC => Box::new(StateC{})
//         }
//     }
// }

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<Event>(10);
    let producer = task::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(500));
        let tx2 = tx.clone();
        loop {
            interval.tick().await;
            let event:Event = Event::E1;
            match tx2.send(event).await {
                Ok(_) => {},
                Err(e) => {
                    println!("error sending event {}", e);
                }
            }
        }
    });

    // while let Some(message) = rx.recv().await {
    //     println!("GOT = {}", message.to_string());
    // }
    let mut state_machine = 
    nefsm::StateMachine::<State, Context, Event>::new (Context {retries : 0});

    let consumer = task::spawn(async move{
       
        state_machine.init(State::StateA);

        while let Some(message) = rx.recv().await {            
             println!("current state {:?} - event {:?}", state_machine.get_current_state(), message);
             state_machine.process_event(&message);
        }
    });



    producer.await;
    consumer.await;

}