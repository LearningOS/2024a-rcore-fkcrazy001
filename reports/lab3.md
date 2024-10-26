- 完成的功能
1. spawn: 采用了 taskcontrolblock::new 方法来通过elf数据来产生一个任务，与initproc的生成方式一致.区别是spawn生成的task需要维护父子进程关系。
2. stride调度算法：在tcb里面加了prio与当前stride值。然后在fetch_task中选取stride最小的值。

- 问答作业
1. p2 会继续执行，因为 p2.stride += 10 之后溢出，值会变为 5， 又变成最小的了。
2. 假设调度队列中初始有n个任务为t1、t2...tn，满足t1.stride == t2.stride == tn.stride=0，那么当t1执行之后，t1 的 stride <=  BigStride / 2, 这个时候STRIDE_MAX=t1.stride – STRIDE_MIN=t2.stride <= BigStride / 2。 显然，当下一次t2执行时，max(t2.stride, t1.stride) - t3.stride < BigStride / 2仍然满足。
3.
```rust
use core::cmp::Ordering;

struct Stride(u64);

impl PartialOrd for Stride {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let diff = self.0.abs_diff(other.0);
        if self.0 > other.0 {
            if diff > BigStride / 2 {
                Some(Ordering::Less)
            } else {
                Some(Ordering::Greater)
            }
        } else {
            if diff > BigStride / 2 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Less)
            }
        }
    }
}

impl PartialEq for Stride {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}
```