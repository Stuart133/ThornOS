use lazy_static::lazy_static;
use spin::Mutex;

use crate::paging::PageTable;

const NPROC: usize = 2;

lazy_static! {
    static ref PROCESS_LIST: [Mutex<Process>; NPROC] = init_process_list();
}

#[allow(dead_code)]
struct Process {
    state: State,
    exit_code: i32,
    process_id: i32,
    pagetable: PageTable,
}

impl Process {
    fn new() -> Self {
        Process {
            state: State::Available,
            exit_code: 0,
            process_id: 0,
            pagetable: PageTable::new(),
        }
    }
}

#[allow(dead_code)]
enum State {
    Available,
    Ready,
    Running,
    Blocked,
    Zombie,
}

#[allow(dead_code)]
fn init_process() {}

#[allow(unreachable_code)]
fn init_process_list() -> [Mutex<Process>; NPROC] {
    [
        Mutex::new(Process::new()),
        Mutex::new(Process::new()),
    ]
}
