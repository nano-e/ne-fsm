

use nefsm;

use nefsm::sync::EventHandler;
use nefsm::sync::FsmEnum;
use nefsm::sync::Response;
use nefsm::sync::Stateful;
use rand::Rng;



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


pub struct GlobalStateTransitionHandler;

impl EventHandler<State, Context, Event> for GlobalStateTransitionHandler {
    fn on_event(&mut self, event: &Event, context: &mut Context) -> Response<State> {
        match event {
            Event::Started => {
                println!("Global state transition handler: Started event received");
                let mut rng = rand::thread_rng();
                let next_state = match rng.gen_range(0..3) {
                    0 => State::Null,
                    1 => State::Starting,
                    2 => State::Ready,
                    _ => panic!("Unexpected random value"),
                };
                Response::Transition(next_state)
            }
            _ => Response::Handled,
        }
    }
}


// impl FsmEnum<State, Context, Event> for State{
//     fn create<'a>(enum_value: &'a State) -> &'a mut Box<dyn Stateful<State, Context, Event>> {
//         &mut Box::new(Null{})
//     }
// }

// impl FsmEnum<State, Context, Event> for State {

//     fn create(&self) -> Option<&mut Box<dyn nefsm::sync::Stateful<S, CTX, E>>> {
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



impl nefsm::sync::Stateful<State, Context, Event> for Null {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::sync::Response<State> {
        println!("Null state on enter, retries = {}", context.retries);
        nefsm::sync::Response::Transition(State::Starting)
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::sync::Response<State> {
        println!("Null state on event : {:?}", event);
        nefsm::sync::Response::Transition(State::Starting)
    }

    fn on_exit(&mut self, context: &mut Context) {
        println!("Null state on exit");
    }
}

impl nefsm::sync::Stateful<State, Context, Event> for Starting {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::sync::Response<State> {
        println!("Starting state on enter");
        context.retries = context.retries + 1;
        nefsm::sync::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::sync::Response<State> {
        println!("Starting state on event : {:?}", event);
        match event {
            Event::Started => nefsm::sync::Response::Transition(State::Ready),
            _ => nefsm::sync::Response::Handled
        }
    }

    fn on_exit(&mut self, context: &mut Context) {
        println!("Starting state on exit");
    }
}

impl nefsm::sync::Stateful<State, Context, Event> for Ready {
    fn on_enter(&mut self, context: &mut Context) -> nefsm::sync::Response<State> {
        println!("Ready state on enter");
        nefsm::sync::Response::Handled
    }

    fn on_event(&mut self, event: &Event, context: &mut Context) -> nefsm::sync::Response<State> {
        println!("Ready state on event : {:?}", event);
        match event{            
            Event::Disconnected => nefsm::sync::Response::Transition(State::Null),
            _ => nefsm::sync::Response::Handled
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
        nefsm::sync::StateMachine::<State, Context, Event>::new (Context {retries : 0}, Some(Box::new(GlobalStateTransitionHandler{})));
    
    state_machine.init(State::Null);

    let events =[Event::Started, Event::Disconnected, Event::Started, Event::Disconnected];

    for e in events.into_iter() {
        if let Err(e) = state_machine.process_event(&e) {
            println!("state machine error : {:?}", e);
        }    
    }    
}