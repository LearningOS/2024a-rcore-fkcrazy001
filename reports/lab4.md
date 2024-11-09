ch6:
1. 完成的功能：
    linkat: 是linux中的hardlink，意思就是把一个磁盘中的文件hardlink到一个entry上去。由此，hardlink只需要在 root_entry 下创建一个新的entry，然后把entry的 inode 号与旧的保持一致，然后将这个disk_inode的硬引用计数+1；
    
    unlinkat： 通过 name 找到 root_node 下面的 entry, 首先将这个entry的内容清空，并判断-1之后是否为0，如果为0，则将这个disk_inode的数据进行清除。
    
    create：首先去root_node下面检查是否有空的entry，如果有的话直接使用这个空的entry。 不然才执行原有的resize流程。
2. 问答
    root inode就是根目录对应的inode。
    root inode损坏的发生在 entry——name区域，那么导致文件的名称发生变化；
    root inode损坏发生在entry-inode-n区域，那么将导致文件内容发生变化。

ch7:
1. 使用 pipe 的实例: 数 a.txt 的行数。

    cat a.txt | wc -l
2. 多进程通信机制： 网络通信/共享内存。