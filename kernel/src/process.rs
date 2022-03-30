use lazy_static::lazy_static;
use spin::Mutex;

use crate::{pagetable::PageTable, println};

const NPROC: usize = 2;

lazy_static! {
    static ref PROCESS_LIST: [Mutex<Process>; NPROC] = init_process_list_internal();
}
static NEXT_PID: Mutex<u64> = Mutex::new(0);

#[allow(dead_code)]
#[derive(Debug)]
struct Process {
    state: State,
    exit_code: i32,
    process_id: u64,
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
#[derive(Debug)]
enum State {
    Available,
    Ready,
    Running,
    Blocked,
    Zombie,
}

pub fn init_process_list() {
    println!("{:p}", &PROCESS_LIST);
}

fn init_process_list_internal() -> [Mutex<Process>; NPROC] {
    [Mutex::new(Process::new()), Mutex::new(Process::new())]
}

pub fn allocate_process() {
    for proc in PROCESS_LIST.iter() {
        let mut p = proc.lock();
        match p.state {
            State::Available => {
                let mut next_pid = NEXT_PID.lock();

                p.state = State::Ready;
                p.process_id = *next_pid;
                p.pagetable = PageTable::new();

                *next_pid += 1;
            }
            _ => continue,
        }
    }
}
