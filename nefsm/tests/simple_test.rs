#[cfg(test)]
mod tests {
    use nefsm::{*};

    #[derive(Debug, Hash, PartialEq, Eq, Clone)]
    enum TestState {
        State1,
        State2,
    }

    #[derive(Debug)]
    enum TestEvent {
        Event1,
        Event2,
    }

    impl FsmEnum<TestState, (), TestEvent> for TestState {
        fn create(enum_value: &TestState) -> Box<dyn Stateful<TestState, (), TestEvent> + Send> {
            match enum_value {
                TestState::State1 => Box::new(TestState1 {}),
                TestState::State2 => Box::new(TestState2 {}),
            }
        }
    }

    struct TestState1 {}

    impl Stateful<TestState, (), TestEvent> for TestState1 {
        fn on_enter(&mut self, _context: &mut ()) -> Response<TestState> {
            Response::Handled
        }

        fn on_event(&mut self, event: &TestEvent, _context: &mut ()) -> Response<TestState> {
            match event {
                TestEvent::Event1 => Response::Handled,
                TestEvent::Event2 => Response::Transition(TestState::State2),
            }
        }

        fn on_exit(&mut self, _context: &mut ()) {}
    }

    struct TestState2 {}

    impl Stateful<TestState, (), TestEvent> for TestState2 {
        fn on_enter(&mut self, _context: &mut ()) -> Response<TestState> {
            Response::Handled
        }

        fn on_event(&mut self, event: &TestEvent, _context: &mut ()) -> Response<TestState> {
            match event {
                TestEvent::Event1 => Response::Handled,
                TestEvent::Event2 => Response::Handled,
            }
        }

        fn on_exit(&mut self, _context: &mut ()) {}
    }

    #[test]
    fn test_state_machine() {
        let mut sm = StateMachine::new(());
        sm.init(TestState::State1).unwrap();
        assert_eq!(*sm.get_current_state().unwrap(), TestState::State1);
        sm.process_event(&TestEvent::Event1).unwrap();
        assert_eq!(*sm.get_current_state().unwrap(), TestState::State1);
        sm.process_event(&TestEvent::Event2).unwrap();
        assert_eq!(*sm.get_current_state().unwrap(), TestState::State2);
    }
}
