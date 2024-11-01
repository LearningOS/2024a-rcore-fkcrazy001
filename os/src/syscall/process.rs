//! Process management syscalls
//!
use alloc::sync::Arc;

use crate::{
    config::MAX_SYSCALL_NUM,
    fs::{open_file, OpenFlags},
    mm::{translated_refmut, translated_str, MapPermission, VirtAddr},
    task::{
        add_task, copy_km_to_va, current_task, current_user_token, exit_current_and_run_next, get_task_info, suspend_current_and_run_next, try_map_va_range, try_unmap_va_range, TaskStatus
    },
    timer::get_time_us,
};
use core::mem::size_of;

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
/// exit
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("kernel:pid[{}] sys_exit", current_task().unwrap().pid.0);
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
}

/// yield
pub fn sys_yield() -> isize {
    //trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get pid
pub fn sys_getpid() -> isize {
    trace!("kernel: sys_getpid pid:{}", current_task().unwrap().pid.0);
    current_task().unwrap().pid.0 as isize
}

/// fork
pub fn sys_fork() -> isize {
    trace!("kernel:pid[{}] sys_fork", current_task().unwrap().pid.0);
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.inner_exclusive_access().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

/// exec
pub fn sys_exec(path: *const u8) -> isize {
    trace!("kernel:pid[{}] sys_exec", current_task().unwrap().pid.0);
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let task = current_task().unwrap();
        task.exec(all_data.as_slice());
        0
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    //trace!("kernel: sys_waitpid");
    let task = current_task().unwrap();
    // find a child process

    // ---- access current PCB exclusively
    let mut inner = task.inner_exclusive_access();
    if !inner
        .children
        .iter()
        .any(|p| pid == -1 || pid as usize == p.getpid())
    {
        return -1;
        // ---- release current PCB
    }
    let pair = inner.children.iter().enumerate().find(|(_, p)| {
        // ++++ temporarily access child PCB exclusively
        p.inner_exclusive_access().is_zombie() && (pid == -1 || pid as usize == p.getpid())
        // ++++ release child PCB
    });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after being removed from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily access child PCB exclusively
        let exit_code = child.inner_exclusive_access().exit_code;
        // ++++ release child PCB
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB automatically
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
    trace!("kernel:pid[{}] sys_sbrk", current_task().unwrap().pid.0);
    if let Some(old_brk) = current_task().unwrap().change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

/// YOUR JOB: Implement spawn.
/// HINT: fork + exec =/= spawn
pub fn sys_spawn(path: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_spawn",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(app_inode) = open_file(path.as_str(), OpenFlags::RDONLY) {
        let all_data = app_inode.read_all();
        let current = current_task().unwrap();
        let new = current.spawn(all_data.as_slice());
        let pid = new.getpid() as isize;
        add_task(new);
        pid
    } else {
        -1
    }
}

/// YOUR JOB: Set task priority.
pub fn sys_set_priority(prio: isize) -> isize {
    trace!(
        "kernel:pid[{}] sys_set_priority",
        current_task().unwrap().pid.0
    );
    match prio {
        x if x >=2 => {
            let current = current_task().unwrap();
            current.inner_exclusive_access().priority = x as usize;
            x
        }
        _ => -1
    }
}
