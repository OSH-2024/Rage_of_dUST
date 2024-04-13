## 摘要
---
本文为 Rage_of_dUST 小组的关于使用 Rust 改写 LiteOS 内存管理单元的可行性报告，文章先介绍了LiteOS 在Linux和Windows下的编译方法，并讨论了LiteOS 的代码树结构，分析了各个代码模块的组成和功能，详细阐述了 LiteOS 的内核代码组成，并对每个可能改写的模块改写时的优点和挑战进行了分析和评价。然后对 Rust 和 C 的相互调用的实现方式进行了阐述和说明，并讨论了实现正常编译的可行性，而且挑选了其中几个小的代码块进行了试改写，验证这样改写的可行性，最后对该项目的创新点进行的简要概述，并且对今后的工作开展和进度控制进行了展望。
## LiteOS 在 Linux 及 Windows下的编译方法
---
### LiteOS 在 Linux 下的编译方法
### LiteOS 在 Windows下的编译方法
## 改写模块选择
---
### LiteOS 代码树结构
LiteOS的代码树结构如下：
```
.
├─arch
├─build
├─compat
├─components
│  ├─ai
│  ├─bootloader
│  ├─connectivity
│  ├─fs
│  ├─gui
│  ├─language
│  ├─lib
│  ├─media
│  ├─net
│  ├─ota
│  ├─security
│  ├─sensorhub
│  └─utility
├─demos
├─doc
├─drivers
├─include
├─kernel
├─lib
├─osdepends
├─shell
├─targets
├─test
├─tests
├─tools
├─Makefile
└─.config
```
下面对每个目录文件夹的作用进行简要说明：
+ arch:实现对 arm , riscv ,  cskyv2 等架构的支持
+ build:LiteOS编译系统需要的配置及脚本
+ compat:liteos提供的CMSIS-RTOS 1.0和2.0接口
+ components:文件系统，媒体，日志，语言等相关组件
+ demos:各个模块 demo 汇总
+ doc:存放 LiteOS 的使用文档和API说明等文档
+ drivers:驱动框架，并包含了串口，定时器，中断接口
+ include:components各个模块所依赖的头文件
+ kernel:LiteOS 基础内核代码，包括任务、中断、软件定时器、队列、事件、信号量、互斥锁、tick等功能，及一些扩展功能代码，如cpu占用率统计，trace 跟踪系统轨迹
+ lib:LiteOS适配的lib库
+ osdepends:LiteOS 提供的部分OS适配接口
+ shell:实现 shell 命令的代码，支持基本调试功能
+ targets:各种开发板的开发工程源包
+ test and tests:单独对各个模块(如内核中任务管理，内存管理模块)的测试文件及对整个LiteOS的测试文件
+ tools:LiteOS支持的开发板编译配置文件及LiteOS编译所需的menuconfig脚本
+ Makefile:顶层的Makefile编译脚本
+ .config:开发板的配置文件

由于 LiteOS 工程整体太过庞大，对整个系统进行全部改写显然无法做到，鉴于小组的编程背景和改写难度，我们确定改写的总代码量约为4000-6000,在这样的考虑下，我们对每个模块进行了初步筛选，分析如下：
#### 改写components模块的考虑：
components 为 LiteOS 的各种组件，其中含有 fs 及 net 等组分的相关代码，整个 components 代码量非常庞大，远远超过了 kernel 部分，最初我们将 fs 列为 components 中改写的目标，但 LiteOS 带有的可以编译的 fs 一共有7个，每个都大概在400行的代码量，若只改写一个则太过于轻松，达不到通过改写体会 Rust 语言的目的，但若全部改写，则显得过于累赘，因为在menuconfig配置时不需要如此多的 fs ，只需要勾选我们想使用的即可。
所以，我们放弃对components模块的改写
#### 改写drivers模块的考虑：
drivers 为
#### 改写kernel模块的考虑：
#### 改写shell命令的考虑：
综上讨论，我们选择 LiteOS 的kernel进行改写，下面对kernel模块内部进行分析和考虑：
### LiteOS 内核代码树结构
```
.
├─kernel
│  ├─base
│  │  ├─debug
│  │  ├─include
│  │  ├─mem
│  │  │  ├─bestfit
│  │  │  ├─bestfit_little
│  │  │  ├─common
│  │  │  │  ├─memstat
│  │  │  │  └─multipool
│  │  │  ├─membox
│  │  │  └─slab
│  │  ├─sched
│  │  │  ├─sched_mq
│  │  │  └─sched_sq
│  │  └─shellcmd
│  ├─extended
│  ├─include
│  └─init
```
下面对kernel内各模块进行简单的功能介绍



