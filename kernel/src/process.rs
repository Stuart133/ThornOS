use lazy_static::lazy_static;
use spin::Mutex;

const NPROC: usize = 2;

lazy_static! {
    static ref PROCESS_LIST: [Mutex<Process>; NPROC] = init_process_list();
}

struct Process {
    state: State,
    exit_code: i32,
    process_id: i32,
}

enum State {
    Available,
    Ready,
    Running,
    Blocked,
    Zombie,
}

fn init_process() {}

fn init_process_list() -> [Mutex<Process>; NPROC] {
    [
        Mutex::new(Process {
            state: State::Available,
            exit_code: 0,
            process_id: 0,
        }),
        Mutex::new(Process {
            state: State::Available,
            exit_code: 0,
            process_id: 0,
        }),
    ]
}
