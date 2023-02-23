

use nefsm;

use nefsm::FsmEnum;
use nefsm::Stateful;



#[derive(Hash, Eq, PartialEq, Clone, Debug)]
pub enum State {
    Null,
    Starting,
    Ready,        
}

impl FsmEnum<State, Context, Event> for State {
    fn create(enum_value: &State) -> Box<dyn Stateful<State, Context, Event> + Send> {
        match enum_value {
            State::Null => Box::new(Null {}),
            State::Starting => Box::new(Starting{}),
            State::Ready => Box::new(Ready {}),
        }
    }
}

pub struct Null {

}
pub struct Starting {

}

pub struct Ready {

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
        println!("Null state on enter, retries = {}", context.retries);
        nefsm::Response::Transition(State::Starting)
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::Response<State> {
        println!("Null state on event : {:?}", event);
        nefsm::Response::Transition(State::Starting)
    }

    fn on_exit(&mut self, context: &mut Context) {
        println!("Null state on exit");
    }
}

impl nefsm::Stateful<State, Context, Event> for Starting {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::Response<State> {
        println!("Starting state on enter");
        context.retries = context.retries + 1;
        nefsm::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::Response<State> {
        println!("Starting state on event : {:?}", event);
        match event {
            Event::Started => nefsm::Response::Transition(State::Ready),
            _ => nefsm::Response::Handled
        }
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
        match event{            
            Event::Disconnected => nefsm::Response::Transition(State::Null),
            _ => nefsm::Response::Handled
        }
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
        nefsm::StateMachine::<State, Context, Event>::new (Context {retries : 0});
    
    state_machine.init(State::Null);

    let events =[Event::Started, Event::Disconnected, Event::Started, Event::Disconnected];

    for e in events.into_iter() {
        if let Err(e) = state_machine.process_event(&e) {
            println!("state machine error : {:?}", e);
        }    
    }    
}