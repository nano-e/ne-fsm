

use nefsm;
use nefsm_macro::{ToStruct, fsm_trait};
use nefsm::FsmEnum;
use nefsm::Stateful;



#[derive(Hash, Eq, PartialEq, Clone, Debug)]
#[fsm_trait(State, Context, Event)]
pub enum State {
    Null,
    Starting,
    Ready,
}

// impl FsmEnum<State, Context, Event> for State{
//     fn create<'a>(enum_value: &'a State) -> &'a mut Box<dyn Stateful<State, Context, Event>> {
//         &mut Box::new(Null{})
//     }
// }

// impl FsmEnum<State, Context, Event> for State {

//     fn create(&self) -> Option<&mut Box<dyn nefsm::Stateful<S, CTX, E>>> {
//         todo!()
//     }
// }
impl ToString for State{
    fn to_string(&self) -> String {
        stringify!(self).to_owned()
    } 
}

#[derive(Debug)]
pub enum Event {
    Started,
    Disconnected,
}

impl ToString for Event {
    fn to_string(&self) -> String {
       stringify!(self).to_owned()
    }
}



impl nefsm::Stateful<State, Context, Event> for Null {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::Response<State> {
        println!("Null state on enter");
        nefsm::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::Response<State> {
        println!("Null state on event : {:?}", event);
        nefsm::Response::Transition(State::Ready)
    }

    fn on_exit(&mut self, context: &mut Context) {
        println!("Null state on exit");
    }
}

impl nefsm::Stateful<State, Context, Event> for Starting {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::Response<State> {
        println!("Starting state on enter");
        nefsm::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::Response<State> {
        println!("Starting state on event : {:?}", event);
        nefsm::Response::Handled
    }

    fn on_exit(&mut self, context: &mut Context) {
        println!("Starting state on exit");
    }
}

impl nefsm::Stateful<State, Context, Event> for Ready {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::Response<State> {
        println!("Ready state on enter");
        nefsm::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::Response<State> {
        println!("Ready state on event : {:?}", event);
        nefsm::Response::Handled
    }

    fn on_exit(&mut self, context: &mut Context) {
        println!("Ready state on exit");
    }
}

#[derive(Debug)]
pub struct Context {
    retries: u32
}

fn main() {
    let mut state_machine = 
        nefsm::StateMachine::<State, Context, Event>::new (State::Null, Context {retries : 0}, true);
    state_machine.add_state(State::Null, Box::new(Null{}));
    state_machine.process_event(&Event::Started);
    println!("Hello world");   
}