## main concept
### Swap List & Primary List
When the main thread spawns a task,the main thread will put it into the `swap list`,the swap list is a special list for the main thread,with this the main thread will not wait when it wants to spawn a task.

When threads in the thread pool want to take on a task,the thread pool will focus on the primary list.However,when the primary list is empty,a representative from the thread pool will perform a swap on the swap list to fill the primary list.After the swap is performed,the swap list will be empty.This swap process will not disrupt the main thread

### Representative Thread
Threads in the thread pool have no central governing body.They are all parallel.To reduce latency due to conflicting threads when attempting to retrieve tasks from the primary list or perform swaps,one thread is selected from the pool as the Representative Thread,based on first-come-first-served. Some representative thread conditions:
1.If the queue in a thread is empty,it will nominate itself to become the representative thread
2.If selected,then he can take the task on the primary list and perform a swap
3.The maximum task it can take is based on the size of its queue thread
4.After that,this thread will release the representative thread to another thread
5.If it is not selected as the representative thread,this thread will steal tasks from another thread's queue

### Work Stealing
When a thread fails to become a representative,the thread will steal a task from another queue,the thread will choose randomly using Xorshift,when the designated target thread is empty,the thread returns to the mode of submitting itself to become a representative.If the target thread has a task,the thread will steal half of the total tasks in the target thread queue,here it will use compare_and_swap to avoid race conditions when many threads try to steal from 1 thread

### Owner And Thieves
Owners and Thieves have different approaches to their activities.Owners execute tasks from the bottom, while Thieves steal tasks from the top
