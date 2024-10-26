//! Process management syscalls
use core::mem::size_of;

use crate::{
    config::MAX_SYSCALL_NUM, mm::{MapPermission, VirtAddr}, task::{
        change_program_brk, copy_km_to_va, exit_current_and_run_next, get_task_info, suspend_current_and_run_next, try_map_va_range, try_unmap_va_range, TaskStatus
    }, timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
/// time val
pub struct TimeVal {
    /// sec
    pub sec: usize,
    /// usec
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub time: usize,
}

impl TaskInfo {
    fn new()->Self {
        Self{
            status:TaskStatus::Running,
            syscall_times:[0;MAX_SYSCALL_NUM],
            time:0,
        }
    }
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let utime = get_time_us();
    let time = TimeVal {
        usec: utime % 1_000_000,
        sec:utime/1_000_000,
    };
    copy_km_to_va(&time, unsafe{&mut *_ts as &mut TimeVal}, size_of::<TimeVal>());
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    // user_va -> pa
    let mut kti = TaskInfo::new();
    get_task_info(&mut kti);
    copy_km_to_va(&kti, unsafe {
        &mut *_ti as &mut TaskInfo
    }, size_of::<TaskInfo>());
    0
}

/// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    if (_port & (!0x7)) != 0 || _port & 0x7 == 0 {
        return -1;
    }
    let start_va:VirtAddr = _start.into();
    if start_va.page_offset() != 0 {
        return -1;
    }
    let end_va:VirtAddr = (_start + _len).into();
    let end_va = end_va.ceil().into();
    // println!("map {} to {}", _start, _start+_len);
    let mut mpflag = MapPermission::U;
    if _port & 0x1 != 0 {
        mpflag |= MapPermission::R;
    }
    if _port & 0x2 != 0 {
        mpflag |= MapPermission::W;
    }
    if _port &0x4 != 0 {
        mpflag |= MapPermission::X;
    }
    match try_map_va_range(start_va, end_va, mpflag) {
        Ok(_) => {
            // println!("map {:?} -> {:?} OK", start_va, end_va);
            0
        }
        Err(_) => {
            // println!("map {:?} -> {:?} FAILED, failed addr: {:?}", start_va, end_va, e);
            -1
        }
    }
}

/// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    let start_va:VirtAddr = _start.into();
    if start_va.page_offset() != 0 {
        return -1;
    }
    let end_va:VirtAddr = (_start + _len).into();
    let end_va = end_va.ceil().into();
    match try_unmap_va_range(start_va, end_va) {
        Ok(_) => {
            // println!("unmap {:?} -> {:?} OK", start_va, end_va);
            0
        }
        Err(_) => {
            // println!("unmap {:?} -> {:?} FAILED, failed addr: {:?}", start_va, end_va, e);
            -1
        }
    }
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
