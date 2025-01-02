use nefsm::sync::{Error, FsmEnum, Response, StateMachine, Stateful};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum TestState {
    A,
    B,
    C,
}

#[derive(Debug)]
enum TestEvent {
    TransitionToB,
    TransitionToC,
    TransitionToA,
    NoTransition,
    IncrementByTwo,
}

struct TestContext {
    transitions: u32,
}

impl FsmEnum<TestState, TestContext, TestEvent> for TestState {
    fn create(
        enum_value: &TestState,
    ) -> Box<dyn Stateful<TestState, TestContext, TestEvent> + Send> {
        match enum_value {
            TestState::A => Box::new(A {}),
            TestState::B => Box::new(B {}),
            TestState::C => Box::new(C {}),
        }
    }
}

// Implement the Stateful trait for each test state
macro_rules! impl_stateful {
    ($state_name:ident, $next_event:path, $next_state:ident) => {
        impl Stateful<TestState, TestContext, TestEvent> for $state_name {
            fn on_enter(&mut self, context: &mut TestContext) -> Response<TestState> {
                context.transitions += 1;
                Response::Handled
            }

            fn on_event(
                &mut self,
                event: &TestEvent,
                _context: &mut TestContext,
            ) -> Response<TestState> {
                match event {
                    $next_event => Response::Transition(TestState::$next_state),
                    _ => Response::Handled,
                }
            }

            fn on_exit(&mut self, _context: &mut TestContext) {}
        }
    };
}

struct A {}
impl_stateful!(A, TestEvent::TransitionToB, B);

struct B {}
impl_stateful!(B, TestEvent::TransitionToC, C);

struct C {}
impl_stateful!(C, TestEvent::TransitionToA, A);

pub struct TestGlobalHandler {
    pub handled_count: usize,
}

impl nefsm::sync::EventHandler<TestState, TestContext, TestEvent> for TestGlobalHandler {
    fn on_event(&mut self, event: &TestEvent, context: &mut TestContext) -> Response<TestState> {
        match event {
            TestEvent::IncrementByTwo => {
                context.transitions += 2;
                self.handled_count += 1;
                Response::Transition(TestState::B)
            }
            _ => Response::Handled,
        }
    }
}

// Now, let's define the tests
#[cfg(test)]
mod tests {
    use super::*;

    // Test the state machine initialization
    #[test]
    fn test_state_machine_initialization() {
        let mut fsm = StateMachine::<TestState, TestContext, TestEvent>::new(
            TestState::A,
            TestContext { transitions: 0 },
            None,
        )
        .unwrap();

        assert_eq!(fsm.get_current_state(), &TestState::A);
        assert_eq!(fsm.get_context().transitions, 1);
    }

    // Test state machine transitions
    #[test]
    fn test_state_machine_transitions() {
        let mut fsm = StateMachine::<TestState, TestContext, TestEvent>::new(
            TestState::A,
            TestContext { transitions: 0 },
            None,
        )
        .unwrap();

        fsm.process_event(&TestEvent::TransitionToB).unwrap();
        assert_eq!(fsm.get_current_state(), &TestState::B);
        assert_eq!(fsm.get_context().transitions, 2);

        fsm.process_event(&TestEvent::TransitionToC).unwrap();
        assert_eq!(fsm.get_current_state(), &TestState::C);
        assert_eq!(fsm.get_context().transitions, 3);

        fsm.process_event(&TestEvent::TransitionToA).unwrap();
        assert_eq!(fsm.get_current_state(), &TestState::A);
        assert_eq!(fsm.get_context().transitions, 4);

        fsm.process_event(&TestEvent::NoTransition).unwrap();
        assert_eq!(fsm.get_current_state(), &TestState::A);
        assert_eq!(fsm.get_context().transitions, 4);
    }

    #[test]
    fn test_global_event_handler() {
        let context = TestContext { transitions: 0 };
        let global_handler = Box::new(TestGlobalHandler { handled_count: 0 });

        let mut sm = StateMachine::<TestState, TestContext, TestEvent>::new(
            TestState::A,
            context,
            Some(global_handler),
        )
        .unwrap();

        // Trigger a global event
        sm.process_event(&TestEvent::IncrementByTwo).unwrap();
        let current_context = sm.get_context();
        let current_state = sm.get_current_state();

        assert_eq!(current_context.transitions, 4);
        assert_eq!(*current_state, TestState::B);
    }
}
