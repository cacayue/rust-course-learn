## async生命周期

* ​`async fn`​函数如果有引用类型的参数，那么返回的`Future`​的生命周期会被这些参数生命周期限制
* ```rust

  #![allow(unused)]
  fn main() {
  	async fn foo(x: &u8) -> u8 { *x }

  	// 上面的函数跟下面的函数是等价的:
  	fn foo_expanded<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
  	    async move { *x }
  	}
  }

  ```

  * x有效时，该`Future`​就必须继续等待（.await)，就是说x必须比Future更久
  * ```rust
    use std::future::Future;
    fn bad() -> impl Future<Output = u8> {
        let x = 5;
        borrow_x(&x) // ERROR: `x` does not live long enough
    }

    async fn borrow_x(x: &u8) -> u8 { *x }

    // 输出
    error[E0597]: `x` does not live long enough
     --> src/main.rs:4:14
      |
    4 |     borrow_x(&x) // ERROR: `x` does not live long enough
      |     ---------^^-
      |     |        |
      |     |        borrowed value does not live long enough
      |     argument requires that `x` is borrowed for `'static`
    5 | }
      | - `x` dropped here while still borrowed

    ```

* 常用的解决方法就是将具有引用参数的 `async fn`​ 函数转变成一个具有 `'static`​ 生命周期的 `Future`​

  * ```rust
    use std::future::Future;

    async fn borrow_x(x: &u8) -> u8 { *x }

    fn good() -> impl Future<Output = u8> {
        async {
            let x = 5;
            borrow_x(&x).await
        }
    }
    ```

## async move

使用 `move`​ 关键字来将环境中变量的所有权转移到语句块内

* 优势：不再发愁该如何解决借用生命周期的问题
* 劣势：无法跟其它代码实现对变量的共享

## 当.await 遇见多线程执行器

当使用多线程 `Future`​ 执行器( `executor`​ )时， `Future`​ 可能会在线程间被移动

* 原因是：它内部的任何`.await`​都可能导致它被切换到一个新线程上去执行
* ​`async`​ 语句块中的变量必须要能在线程间传递
* 需要在多线程环境使用，意味着 `Rc`​、 `RefCell`​ 、没有实现 `Send`​ 的所有权类型、没有实现 `Sync`​ 的引用类型，它们都是不安全的，因此无法被使用
* 在 `.await`​ 时使用普通的锁也不安全，例如 `Mutex`​

  * 原因是，它可能会导致线程池被锁：当一个任务获取锁 `A`​ 后，若它将线程的控制权还给执行器，然后执行器又调度运行另一个任务，该任务也去尝试获取了锁 `A`​ ，结果当前线程会直接卡死，最终陷入死锁中
  * 需要使用 `futures`​ 包下的锁 `futures::lock`​ 来替代 `Mutex`​ 完成任务。

## Stream流处理

```rust
trait Stream {
    // Stream生成的值的类型
    type Item;

    // 尝试去解析Stream中的下一个值,
    // 若无数据，返回`Poll::Pending`, 若有数据，返回 `Poll::Ready(Some(x))`, `Stream`完成则返回 `Poll::Ready(None)`
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>)
        -> Poll<Option<Self::Item>>;
}
```

常用例子：消息通道（ `futures`​ 包中的）的消费者 `Receiver`​

```rust
async fn send_recv() {
    const BUFFER_SIZE: usize = 10;
    let (mut tx, mut rx) = mpsc::channel::<i32>(BUFFER_SIZE);

    tx.send(1).await.unwrap();
    tx.send(2).await.unwrap();
    drop(tx);

    // `StreamExt::next` 类似于 `Iterator::next`, 但是前者返回的不是值，而是一个 `Future<Output = Option<T>>`，
    // 因此还需要使用`.await`来获取具体的值
    assert_eq!(Some(1), rx.next().await);
    assert_eq!(Some(2), rx.next().await);
    assert_eq!(None, rx.next().await);
}
```

### 迭代和并发

* 可以使用`map`​，`filter`​，`fold`​方法，以及`try_map`​，`try_filter`​，`try_fold`​
* ​`for`​ 循环无法在这里使用，但是命令式风格的循环["while let"](siyuan://blocks/20221127161046-hto5nva)​是可以用的，同时还可以使用`next`​ 和 `try_next`​ 方法

```rust

async fn sum_with_next(mut stream: Pin<&mut dyn Stream<Item = i32>>) -> i32 {
    use futures::stream::StreamExt; // 引入 next
    let mut sum = 0;
    while let Some(item) = stream.next().await {
        sum += item;
    }
    sum
}

async fn sum_with_try_next(
    mut stream: Pin<&mut dyn Stream<Item = Result<i32, io::Error>>>,
) -> Result<i32, io::Error> {
    use futures::stream::TryStreamExt; // 引入 try_next
    let mut sum = 0;
    //一次处理一个值的模式，可能会造成无法并发
    while let Some(item) = stream.try_next().await? {
        sum += item;
    }
    Ok(sum)
}
```

选择从一个 `Stream`​ 并发处理多个值的方式，通过 `for_each_concurrent`​ 或 `try_for_each_concurrent`​ 方法来实现

```rust
async fn jump_around(
    mut stream: Pin<&mut dyn Stream<Item = Result<u8, io::Error>>>,
) -> Result<(), io::Error> {
    use futures::stream::TryStreamExt; // 引入 `try_for_each_concurrent`
    const MAX_CONCURRENT_JUMPERS: usize = 100;

    stream.try_for_each_concurrent(MAX_CONCURRENT_JUMPERS, |num| async move {
        jump_n_times(num).await?;
        report_n_jumps(num).await?;
        Ok(())
    }).await?;

    Ok(())
}
```

‍
