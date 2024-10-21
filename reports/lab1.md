# 1. 完成的功能
主要工作分为三块
- syscall调用次数统计
- 程序运行时间记录
- taskInfo syscall实现

## syscall 调用次数统计
- 在syscall入口 `crate::syscall::syscall` 中获取syscallid进行统一统计 
- 在进程的tcb中做一个 u32 [max_syscall_num] 的数组，直接将 syscallid作为索引对该数组进行+1操作。

## 程序运行时间记录
- 进程初始化时将所有的进程tcb中第一次被调度的时间设置为0ms
- 在进行调度时检查下一个就绪的任务tcb的第一次调度时间是否为0，如果为0就置为当前时间。

## taskinfo syscall实现
- task_status: 能进行syscall的进程肯定得是running的，所以状态就是running
- syscall_time: 获取当前进程syscall_time数组进行拷贝
- time: 获取当前时间与进程time相减即可。

# trap.s函数解读

- A1: a0 是传递给 __restore的参数（在ch3不需要），也就是kernel sp的地址。

    场景一：进程第一次运行，从supervisor mode到 user——mode。
    
    场景二：进程 return from syscall。

- A2: 这个是从kernel stack中读取 kernel 设置的 sstatus(supervisor status), sepc(app addr), sscratch(app stack start)，对应结构如下：
    ```rust
    pub struct TrapContext {
        /// General-Purpose Register x0-31
        pub x: [usize; 32], // x[2] 就是sp
        /// Supervisor Status Register
        pub sstatus: Sstatus,
        /// Supervisor Exception Program Counter
        pub sepc: usize,
    }
    ```

- A3: x2是app的sp地址，不应在此处设置。 x4是tp，目前没用到。

- A4: 相当于把sp的值和sscratch中的值进行调换，现在sp指向app的stack，而 sscratch指向kernel stk

- A5: sret 指令。按照手册，sret执行之后有如下动作

    ```
    设置 pc 为 CSRs[spec]，权限模式为 CSRs[sstatus].SPP，
    CSRs[sstatus].SIE 为 CSRs[sstatus].SPIE，CSRs[sstatus].SPIE 为 1，CSRs[sstatus].spp 为 0
    ```
  而 sstatus 为 trap 时被保存的。

- A6: 与Q4 问题一样，把sp的值和sscratch中的值进行调换，不过trap时候 sp指向app的stack，而 sscratch指向kernel stk，所以交换之后 sp 指向kernel stk 而 sscratch指向app_stack

- A7 : app 在 U 模式执行ecall，触发中断，进入Smode

# end: 碰到的问题
初版方案: commit id: 807deceb2ffe94ee0b47daa4022fcecdf8cb78e0

在 tcb 中 加入 TaskInfo 并且实现TaskInfo::zero_init()->Self, 在lazy_static!{} 中实现对TASK_MANAGER的初始化。

这样操作后 get_time() 获取到的都是很小的tick值，应该是栈溢出导致写越界了。
目前还不能排查具体情况，等到后续再来做研究。