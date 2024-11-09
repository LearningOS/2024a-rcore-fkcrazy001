1. 完成的功能 -- 花费了一个晚上4小时
   主要是一个死锁检测算法的实现。其基础原理是在分配资源的时候根据现有的信息进行模拟分配，如果能够把资源给分配完成，那么就让分配继续，否则就返回deadlock。

问答题：
1. 需要回收其它线程的
   - res中的tid
   - 其它线程的kstack和userstk映射
   - 其它线程的trap——ctx资源

其它线程的 TaskControlBlock 可能在ready——queue中，是需要做回收的，所以 remove_inactive_task 就是这个作用。

2. 实现区别
   Mutex1的unlock实现会导致最新的被唤醒的task实际上是没有成功持有这个锁的。
   Mutex2的unlock实现可以保证被唤醒的task持有这个锁。