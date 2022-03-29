use lazy_static::lazy_static;
use spin::Mutex;

use crate::{pagetable::PageTable, println, memory};

const NPROC: usize = 2;

lazy_static! {
    static ref PROCESS_LIST: [Mutex<Process>; NPROC] = init_process_list();
}

#[allow(dead_code)]
#[derive(Debug)]
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
            pagetable: memory::copy_active_pagetable(),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
enum State {
    Available,
    Ready,
    Running,
    Blocked,
    Zombie,
}

#[allow(dead_code)]
pub fn init_process() {
    println!("{:p}", &PROCESS_LIST);
}

#[allow(unreachable_code)]
fn init_process_list() -> [Mutex<Process>; NPROC] {
    [Mutex::new(Process::new()), Mutex::new(Process::new())]
}
