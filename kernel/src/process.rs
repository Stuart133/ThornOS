use lazy_static::lazy_static;

lazy_static! {
    static ref PROCESS_LIST: [Process; 2] = init_process_list();
}

struct Process {
    state: State,
}

enum State {
    Available,
    Ready,
    Running,
    Blocked,
    Zombie,
}

fn init_process_list() -> [Process; 2] {
    [
        Process {
            state: State::Available,
        },
        Process {
            state: State::Available,
        },
    ]
}
