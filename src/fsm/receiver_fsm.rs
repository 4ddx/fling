#[derive(Debug)]
pub enum ReceiverState {
    Listening,
    Connected,
    Terminating
}

pub fn start_receiver_fsm() -> ReceiverState{
    use ReceiverState::*;
    println!("Activating receiver FSM");
    let mut state = Listening;
    loop {
        state = match state {
            Listening =>{
                println!("Listening for bluetooth offers");
                ReceiverState::Connected
            }
            Connected => {
                println!("Ready to Receive Data");
                ReceiverState::Terminating
            }
            Terminating => {
                println!("Terminating FSM instance");
                break ReceiverState::Terminating;
            }
        }
    }
}