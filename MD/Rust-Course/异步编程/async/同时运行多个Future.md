## join！

```rust
use futures::join;

async fn enjoy_book_and_music() -> (Book, Music) {
    let book_fut = enjoy_book();
    let music_fut = enjoy_music();
    join!(book_fut, music_fut)
}
```

> 如果希望同时运行一个数组里的多个异步任务，可以使用 `futures::future::join_all`​ 方法

## try_join!

如果希望在某一个 `Future`​ 报错后就立即停止所有 `Future`​ 的执行，可以使用 `try_join!`​

* ​`try_join!`​ 的所有 `Future`​ 都必须拥有相同的错误类型
* 错误类型不同: 使用来自 `futures::future::TryFutureExt`​ 模块的 `map_err`​和`err_info`​方法将错误进行转换

## select!

想同时等待多个 `Future`​ ，且任何一个 `Future`​ 结束后，都可以立即被处理

```rust
use futures::{
    future::FutureExt, // for `.fuse()`
    pin_mut,
    select,
};

async fn task_one() { /* ... */ }
async fn task_two() { /* ... */ }

async fn race_tasks() {
    let t1 = task_one().fuse();
    let t2 = task_two().fuse();

    // 并发地运行 t1 和 t2
    pin_mut!(t1, t2);

    select! {
        () = t1 => println!("任务1率先完成"),
        () = t2 => println!("任务2率先完成"),
    }
}
```

### default 和 complete

​`select!`​还支持 `default`​ 和 `complete`​ 分支:

* ​`complete`​ 分支当所有的 `Future`​ 和 `Stream`​ 完成后才会被执行，它往往配合`loop`​使用，`loop`​用于循环完成所有的 `Future`​
* ​`default`​分支，若没有任何 `Future`​ 或 `Stream`​ 处于 `Ready`​ 状态， 则该分支会被立即执行

### 跟 Unpin 和 FusedFuture 进行交互

​`select`​需求特征：

* ​`.fuse()`​方法可以让 `Future`​ 实现 `FusedFuture`​ 特征

  * ​`FusedFuture`​的原因，当 `Future`​ 一旦完成后，那 `select`​ 就不能再对其进行轮询使用。`Fuse`​意味着熔断，相当于 `Future`​ 一旦完成，再次调用`poll`​会直接返回`Poll::Pending`​。
* ​`pin_mut!`​ 宏会为 `Future`​ 实现 `Unpin`​特征

  * ​`Unpin`​，由于 `select`​ 不会通过拿走所有权的方式使用`Future`​，而是通过可变引用的方式去使用，这样当 `select`​ 结束后，该 `Future`​ 若没有被完成，它的所有权还可以继续被其它代码使用。

## 在select循环中并发

​`Fuse::terminated()`​ ，可以使用它构建一个空的 `Future`​，能在后面再被填充。

案例：当你要在`select`​循环中运行一个任务，但是该任务却是在`select`​循环内部创建时

```rust
#![allow(unused)]
fn main() {
	use futures::{
	    future::{Fuse, FusedFuture, FutureExt},
	    stream::{FusedStream, Stream, StreamExt},
	    pin_mut,
	    select,
	};
	async fn get_new_num() -> u8 { /* ... */ 5 }
	async fn run_on_new_num(_: u8) { /* ... */ }
	async fn run_loop(
	    mut interval_timer: impl Stream<Item = ()> + FusedStream + Unpin,
	    starting_num: u8,
	) {
	    let run_on_new_num_fut = run_on_new_num(starting_num).fuse();
	    let get_new_num_fut = Fuse::terminated();
	    pin_mut!(run_on_new_num_fut, get_new_num_fut);
	    loop {
	        select! {
	            () = interval_timer.select_next_some() => {
	                // 定时器已结束，若`get_new_num_fut`没有在运行，就创建一个新的
	                if get_new_num_fut.is_terminated() {
	                    get_new_num_fut.set(get_new_num().fuse());
	                }
	            },
	            new_num = get_new_num_fut => {
	                // 收到新的数字 -- 创建一个新的`run_on_new_num_fut`并丢弃掉旧的
	                run_on_new_num_fut.set(run_on_new_num(new_num).fuse());
	            },
	            // 运行 `run_on_new_num_fut`
	            () = run_on_new_num_fut => {},
	            // 若所有任务都完成，直接 `panic`， 原因是 `interval_timer` 应该连续不断的产生值，而不是结束
	            //后，执行到 `complete` 分支
	            complete => panic!("`interval_timer` completed unexpectedly"),
	        }
	    }
	}
}

```

‍
