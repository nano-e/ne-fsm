#[cfg(test)]
mod tests {
    use nefsm::sync::*;

    #[derive(Debug, Hash, PartialEq, Eq, Clone)]
    enum TestState {
        State1,
        State2,
    }

    #[derive(Debug)]
    enum TestEvent {
        InvalidEvent,
        TransitionToState2,
    }

    struct TestContext {
        counter: i8,
    }

    impl FsmEnum<TestState, TestContext, TestEvent> for TestState {
        fn create(
            enum_value: &TestState,
        ) -> Box<dyn Stateful<TestState, TestContext, TestEvent> + Send> {
            match enum_value {
                TestState::State1 => Box::new(TestState1 {}),
                TestState::State2 => Box::new(TestState2 {}),
            }
        }
    }

    struct TestState1 {}

    impl Stateful<TestState, TestContext, TestEvent> for TestState1 {
        fn on_enter(&mut self, _context: &mut TestContext) -> Response<TestState> {
            Response::Handled
        }

        fn on_event(
            &mut self,
            event: &TestEvent,
            _context: &mut TestContext,
        ) -> Response<TestState> {
            match event {
                TestEvent::InvalidEvent => Response::Error("cannot handle event1".to_string()),
                TestEvent::TransitionToState2 => {
                    _context.counter += 1;
                    Response::Transition(TestState::State2)
                }
            }
        }

        fn on_exit(&mut self, _context: &mut TestContext) {}
    }

    struct TestState2 {}

    impl Stateful<TestState, TestContext, TestEvent> for TestState2 {
        fn on_enter(&mut self, _context: &mut TestContext) -> Response<TestState> {
            match _context.counter {
                2 => Response::Handled,
                _ => Response::Error("counter needs to be 2 to enter TestState2".to_string()),
            }
        }

        fn on_event(
            &mut self,
            event: &TestEvent,
            _context: &mut TestContext,
        ) -> Response<TestState> {
            match event {
                TestEvent::InvalidEvent => Response::Error("cannot handle event1".to_string()),
                TestEvent::TransitionToState2 => Response::Error("already in state2".to_string()),
            }
        }

        fn on_exit(&mut self, _context: &mut TestContext) {}
    }

    #[test]
    fn test_state_machine() {
        let mut sm = StateMachine::new(TestContext { counter: 0 }, None);
        sm.init(TestState::State1).unwrap();

        assert_eq!(*sm.get_current_state().unwrap(), TestState::State1);
        match sm.process_event(&TestEvent::InvalidEvent) {
            Ok(_) => panic!("event1 should raise an error"),
            Err(Error::InvalidEvent(e)) => assert_eq!("cannot handle event1".to_string(), e),
            Err(e) => panic!("unexpected error {:?}", e),
        }

        assert_eq!(*sm.get_current_state().unwrap(), TestState::State1);

        match sm.process_event(&TestEvent::TransitionToState2) {
            Ok(_) => panic!("first transition should fail"),
            Err(Error::StateInvalid(e)) => {
                assert_eq!("counter needs to be 2 to enter TestState2".to_string(), e)
            }
            Err(e) => panic!("unexpected error {:?}", e),
        }

        assert_eq!(*sm.get_current_state().unwrap(), TestState::State1);

        match sm.process_event(&TestEvent::TransitionToState2) {
            Ok(_) => (),
            Err(e) => panic!("unexpected error {:?}", e),
        }

        assert_eq!(*sm.get_current_state().unwrap(), TestState::State2);
    }
}
