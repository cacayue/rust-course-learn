## async简介

### async VS 其他并发模型

* **OS线程**。它最简单，也无需改变任何编程模型(业务/代码逻辑)，因此非常适合作为语言的原生并发模型；缺点，例如线程间的同步将变得更加困难，线程间的上下文切换损耗较大。使用线程池在一定程度上可以提升性能，但是对于 IO 密集的场景来说，线程池还是不够。
* **事件驱动(Event driven)**, 常常跟回调( Callback )一起使用。性能相当的好，但最大的问题就是存在回调地狱的风险：非线性的控制流和结果处理导致了数据流向和错误传播变得难以掌控，还会导致代码可维护性和可读性的大幅降低
* **协程(Coroutines)**跟线程类似，无需改变编程模型，同时，它也跟 `async`​ 类似，可以支持大量的任务并发运行。抽象层次过高，导致用户无法接触到底层的细节，这对于系统编程语言和自定义异步运行时是难以接受的
* **actor 模型，**将所有并发计算分割成一个一个单元，这些单元被称为 `actor`​ , 单元之间通过消息传递的方式进行通信和数据传递，跟分布式系统的设计理念非常相像。一旦遇到流控制、失败重试等场景时，就会变得不太好用
* **async/await**， 该模型性能高，还能支持底层编程，同时又像线程和协程那样无需过多的改变编程模型，但有得必有失，`async`​ 模型的问题就是内部实现机制过于复杂，对于用户来说，理解和使用起来也没有线程和协程简单
* Rust选择了同时提供多线程编程和async编程

  * 前者通过标准库实现，例如需要并行计算时，可以选择它
  * 后者通过语言特性 + 标准库 + 三方库的方式实现，在你需要高并发、异步 `I/O`​ 时，选择它

### async: Rust vs 其他语言

* ​**Future 在 Rust 中是惰性的**​，只有在被轮询(`poll`​)时才会运行， 因此丢弃一个 `future`​ 会阻止它未来再被运行, 你可以将`Future`​理解为一个在未来某个时间点被调度执行的任务。
* ​**Async 在 Rust 中使用开销是零**​， 意味着只有你能看到的代码(自己的代码)才有性能损耗，你看不到的(`async`​ 内部实现)都没有性能损耗，例如，你可以无需分配任何堆内存、也无需任何动态分发来使用 `async`​ ，这对于热点路径的性能有非常大的好处，正是得益于此，Rust 的异步编程性能才会这么高。
* ​**Rust 没有内置异步调用所必需的运行时**​，但是无需担心，Rust 社区生态中已经提供了非常优异的运行时实现，例如大明星 [tokio]()``​
* **运行时同时支持单线程和多线程**，这两者拥有各自的优缺点

### Rust：async vs 多线程

* OS线程适合少量任务并发，

  * 缺点：因为线程的创建和上下文切换是非常昂贵的，甚至于空闲的线程都会消耗系统资源。
  * 优点：不会破坏你的代码逻辑和编程模型，在某些操作系统中，你还可以改变线程的优先级，这对于实现驱动程序或延迟敏感的应用(例如硬实时系统)很有帮助
* CPU密集型任务

  * 这种密集任务往往会让所在的线程持续运行，任何不必要的线程切换都会带来性能损耗，因此高并发反而在此时成为了一种多余
  * 创建的线程数应该等于 CPU 核心数，充分利用 CPU 的并行能力，甚至还可以将线程绑定到 CPU 核心上，进一步减少线程上下文切换
* ​`IO`​ 密集型任务适合高并发，例如 web 服务器、数据库连接等等网络服务

  * 这些任务绝大部分时间都处于等待状态，如果使用多线程，那线程大量时间会处于无所事事的状态，再加上线程上下文切换的高昂代价，让多线程做 `IO`​ 密集任务变成了一件非常奢侈的事
  * 使用`async`​，既可以有效的降低 `CPU`​ 和内存的负担，又可以让大量的任务并发的运行，一个任务一旦处于`IO`​或者其他等待(阻塞)状态，就会被立刻切走并执行另一个任务，而这里的任务切换的性能开销要远远低于使用多线程时的线程上下文切换
  * 缺点，原因是编译器会为`async`​函数生成状态机，然后将整个运行时打包进来，这会造成我们编译出的二进制可执行文件体积显著增大
* 总结

  * 有大量 `IO`​ 任务需要并发运行时，选 `async`​ 模型
  * 有部分 `IO`​ 任务需要并发运行时，选多线程，如果想要降低线程创建和销毁的开销，可以使用线程池
  * 有大量 `CPU`​ 密集任务需要并行运行时，例如并行计算，选多线程模型，且让线程数等于或者稍大于 `CPU`​ 核心数
  * 无所谓时，统一选多线程

## Async Rust 当前进展

### 语言和库支持

要完整的使用 `async`​ 异步编程，你需要依赖以下特性和外部库:

* 所必须的特征(例如 `Future`​ )、类型和函数，由标准库提供实现
* 关键字 `async/await`​ 由 Rust 语言提供，并进行了编译器层面的支持
* 众多实用的类型、宏和函数由官方开发的 [futures]()``​ 包提供(不是标准库)，它们可以用于任何 `async`​ 应用中。
* ​`async`​ 代码的执行、`IO`​ 操作、任务创建和调度等等复杂功能由社区的 `async`​ 运行时提供，例如 [tokio]()``​ 和 [async-std]()``​

### 编译和错误

* 编译错误，由于 `async`​ 编程时需要经常使用复杂的语言特性，例如生命周期和`Pin`​，因此相关的错误可能会出现的更加频繁
* 运行时错误，编译器会为每一个`async`​函数生成状态机，这会导致在栈跟踪时会包含这些状态机的细节，同时还包含了运行时对函数的调用，因此，栈跟踪记录(例如 `panic`​ 时)将变得更加难以解读
* 一些隐蔽的错误也可能发生，例如在一个 `async`​ 上下文中去调用一个阻塞的函数，或者没有正确的实现 `Future`​ 特征都有可能导致这种错误。这种错误可能会悄无声息的通过编译检查甚至有时候会通过单元测试。好在一旦你深入学习并掌握了本章的内容和 `async`​ 原理，可以有效的降低遇到这些错误的概率

### 兼容性考虑

异步代码和同步代码并不总能和睦共处。例如，我们无法在一个同步函数中去调用一个 `async`​ 异步函数，同步和异步代码也往往使用不同的设计模式，这些都会导致两者融合上的困难。

异步代码之间也存在类似的问题，如果一个库依赖于特定的 `async`​ 运行时来运行，那么这个库非常有必要告诉它的用户，它用了这个运行时。否则一旦用户选了不同的或不兼容的运行时，就会导致不可预知的麻烦

### 性能特性

目前主流的 `async`​ 运行时几乎都使用了多线程实现，相比单线程虽然增加了并发表现，但是对于执行性能会有所损失，因为多线程实现会有同步和切换上的性能开销，若你需要极致的顺序执行性能，那么 `async`​ 目前并不是一个好的选择。

对于延迟敏感的任务来说，任务的执行次序需要能被严格掌控，而不是交由运行时去自动调度，后者会导致不可预知的延迟

## async/.await 简单入门

> 通过 `async`​ 标记的语法块会被转换成实现了`Future`​特征的状态机。 与同步调用阻塞当前线程不同，当`Future`​执行并遇到阻塞时，它会让出当前线程的控制权，这样其它的`Future`​就可以在该线程中运行，这种方式完全不会导致当前线程的阻塞。
>
> 需要先引入 `futures`​ 包，在Cargo.toml：
>
> ```rust
> [dependencies]
> futures = "0.3"
> ```

### 使用async

*  `block_on`​会阻塞当前线程直到指定的`Future`​执行完成，这种阻塞当前线程以等待任务完成的方式较为简单、粗暴，
*  其它运行时的执行器(executor)会提供更加复杂的行为，例如将多个`future`​调度到同一个线程上执行。

```rust
use futures::executor::block_on;

async fn hello_world() {
    println!("hello, world!");
}

fn main() {
    let future = hello_world(); // 返回一个Future, 因此不会打印任何输出
    block_on(future); // 执行`Future`并等待其运行完成，此时"hello, world!"会被打印输出
}
```

### 使用.await

```rust
use futures::executor::block_on;

async fn hello_world() {
    // 使用同步的代码顺序实现了异步的执行效果
    // 但是与block_on不同，.await并不会阻塞当前的线程
    hello_cat().await;
    println!("hello, world!");
}

async fn hello_cat() {
    println!("hello, kitty!");
}
fn main() {
    let future = hello_world();
    block_on(future);
}
```

‍
