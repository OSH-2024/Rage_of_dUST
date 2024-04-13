## 摘要
---
本文为 Rage_of_dUST 小组的关于使用 Rust 改写 LiteOS 内存管理单元的可行性报告，文章先介绍了LiteOS 在Linux和Windows下的编译方法，并讨论了LiteOS 的代码树结构，分析了各个代码模块的组成和功能，详细阐述了 LiteOS 的内核代码组成，并对每个可能改写的模块改写时的优点和挑战进行了分析和评价。然后对 Rust 和 C 的相互调用的实现方式进行了阐述和说明，并讨论了实现正常编译的可行性，而且挑选了其中几个小的代码块进行了试改写，验证这样改写的可行性，最后对该项目的创新点进行的简要概述，并且对今后的工作开展和进度控制进行了展望。
## 理论依据
---
### LiteOS 在 Linux 及 Windows下的编译方法
---
#### 编译框架简介
Huawei LiteOS使用Kconfig文件配置系统，基于GCC/Makefile实现组件化编译。 
<center> 

![complie_arch](figs/Makefile_inclusion_relationship.png)
图1：LiteOS编译框架
</center>
不论是 Linux 下使用make menuconfig命令配置系统，还是 Windows 下使用 Huawei LiteOS Studio 进行图形化配置，Huawei LiteOS都会同时解析、展示根目录下的.config文件和tools/menuconfig/config.in文件(该文件包含了各个模块的Kconfig文件)同时在开发板的include文件夹下生成menuconfig.h,config.in文件由Kconfig语言（一种菜单配置语言）编写而成。config.in文件决定了要展示的配置项，.config文件决定了各个配置项的默认值。Huawei LiteOS通过在根目录下执行make命令完成自动化编译整个工程。对于根目录下的Makefile文件，它包含了config.mk,config.mk又包含了los_config.mk，而los_config.mk则包含了各个模块的Makefile和.config文件，从而定义了对整个工程的编译链接规则。

>+ 对于Windows平台，提供了Huawei LiteOS Studio图形化IDE，用户可直接在 Studio上完成配置和编译。
>+ 对于Linux平台，通过menuconfig进行组件化配置及裁剪后，执行make命令完成编译。
#### LiteOS 在 Linux 下的编译方法
##### Linux 下配置工具的使用
在Huawei_LiteOS根目录下执行make menuconfig命令会展示图形化配置界面，用户可以根据需要自行裁剪模块或者修改设置。执行完保存菜单退出，该命令会更新根目录下.config文件。
具体使用见参考文档。
##### Linux 下的编译
###### 编译环境要求
> + Ubuntu Version 14.04及以上版本，作为编译Huawei LiteOS的服务器。
> + GNU Arm Embedded Toolchain编译器，用于代码编译。
> + GNU Make构建器，用于文件组织与链接。
> + python 2.7/3.2+，pip包管理工具，kconfiglib库，用于编译前通过图形化界面完成配置。
具体安装配置方法见参考文档
###### Linux 下编译流程
1. 下载Huawei LiteOS代码。
2. 请下载完整的Huawei LiteOS代码，代码仓在gitee上，请选择master分支进行下载。
3. 拷贝开发板配置文件为根目录.config文件。
4. 根据实际使用的开发板，拷贝tools/build/config/目录下的默认配置文件${platform}.config到根目录，并重命名为.config。
5. 如果不希望使用系统的默认配置，可以在Huawei_LiteOS根目录下执行make menuconfig命令，在图形化配置界面中自行裁剪模块或修改配置。修改完保存菜单退出，其修改默认会保存到根目录下.config文件中。
6. 在编译前，先在Huawei_LiteOS根目录下执行make clean命令删除以前编译出的二进制文件。
7. 在Huawei_LiteOS根目录下执行make命令即可完成工程编译，编译结果会在屏幕上输出。
> 以Cloud_STM32F429IGTx_FIRE为例，生成的系统镜像文件、反汇编等文件保存在out/Cloud_STM32F429IGTx_FIRE目录中，库文件在out/Cloud_STM32F429IGTx_FIRE/lib目录中，中间文件在out/Cloud_STM32F429IGTx_FIRE/obj目录中。
#### LiteOS 在 Windows下的编译方法
Windows 编译需要安装 LiteOS Studio ，由于我们选择使用Linux虚拟机进行编译，所以这里不做赘述。
### 改写模块选择
---
#### LiteOS 代码树结构
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
>+ arch:实现对 arm , riscv ,  cskyv2 等架构的支持
>+ build:LiteOS编译系统需要的配置及脚本
>+ compat:liteos提供的CMSIS-RTOS 1.0和2.0接口
>+ components:文件系统，媒体，日志，语言等相关组件
>+ demos:各个模块 demo 汇总
>+ doc:存放 LiteOS 的使用文档和API说明等文档
>+ drivers:驱动框架，并包含了串口，定时器，中断接口
>+ include:components各个模块所依赖的头文件
>+ kernel:LiteOS 基础内核代码，包括任务、中断、软件定时器、队列、事件、信号量、互斥锁、tick等功能，及一些扩展功能代码，如cpu占用率统计，trace 跟踪系统轨迹
>+ lib:LiteOS适配的lib库
>+ osdepends:LiteOS 提供的部分OS适配接口
>+ shell:实现 shell 命令的代码，支持基本调试功能
>+ targets:各种开发板的开发工程源包
>+ test and tests:单独对各个模块(如内核中任务管理，内存管理模块)的测试文件及对整个LiteOS的测试文件
>+ tools:LiteOS支持的开发板编译配置文件及LiteOS编译所需的menuconfig脚本
>+ Makefile:顶层的Makefile编译脚本
>+ .config:开发板的配置文件

由于 LiteOS 工程整体太过庞大，对整个系统进行全部改写显然无法做到，鉴于小组的编程背景和改写难度，我们确定改写的总代码量约为4000-6000,在这样的考虑下，我们对每个模块进行了初步筛选，分析如下：
##### 改写components模块的考虑：
components 模块为 LiteOS 的各种组件，其中含有 fs 及 net 等组分的相关代码，整个 components 代码量非常庞大，远远超过了 kernel 部分，最初我们将 fs 列为 components 中改写的目标，但 LiteOS 带有的可以编译的 fs 一共有7个，每个都大概在400行的代码量，若只改写一个则太过于轻松，达不到通过改写体会 Rust 语言的目的，但若全部改写，则显得过于累赘，因为在menuconfig配置时不需要如此多的 fs ，只需要勾选我们想使用的即可。更进一步考虑，部分改写应选择较为独立的一部分进行改写较为妥当，且 components 中大部分组件为操作系统外围的功能性应用，改写意义不大。
所以，我们放弃对components模块的改写。
##### 改写shell命令的考虑：
shell 模块为 LiteOS 实现shell命令的代码，代码量约为3000行左右。
+ 改写优势：shell命令模块较为独立，可以很好的改写全部shell模块后再进行静态链接，且代码量适中。
+ 改写缺点：shell命令作为人机交互的接口，是OS外部的一层封装，用于与用户交互，在这种意义下，其对操作系统安全性的影响不大，与内核等操作系统核心模块比较，重要性略显不足。
##### 改写kernel模块的考虑：
kernel 模块为 LiteOS 的核心，包含了 LiteOS 基础内核代码，包括任务、中断、软件定时器、队列、事件、信号量、互斥锁、tick等功能，这些功能是操作系统的核心，也是操作系统在计算机中发挥最大作用的部分。
+ 改写优势：
   + 内核代码区分度高：基础内核代码分为mem , sched , 
  
综上讨论，我们选择 LiteOS 的kernel进行改写，下面对kernel模块内部进行分析和考虑：
#### LiteOS 内核代码树结构
```
.
│  Kconfig
│  Makefile
│
├─base
│  │  los_bitmap.c
│  │  los_err.c
│  │  los_event.c
│  │  los_exc.c
│  │  los_hwi.c
│  │  los_lockdep.c
│  │  los_misc.c
│  │  los_mp.c
│  │  los_mux.c
│  │  los_percpu.c
│  │  los_printf.c
│  │  los_queue.c
│  │  los_ringbuf.c
│  │  los_sem.c
│  │  los_sortlink.c
│  │  los_stackinfo.c
│  │  los_swtmr.c
│  │  los_task.c
│  │  los_tick.c
│  │  Makefile
│  │
│  ├─debug
│  │      los_mux_deadlock.c
│  │      los_mux_debug.c
│  │      los_queue_debug.c
│  │      los_sched_debug.c
│  │      los_sem_debug.c
│  │
│  ├─include//28 .h
│  ├─mem
│  │  │  Kconfig
│  │  │
│  │  ├─bestfit
│  │  │      los_memory.c
│  │  │      los_memory_internal.h
│  │  │      los_multipledlinkhead.c
│  │  │
│  │  ├─bestfit_little
│  │  │      los_heap.c
│  │  │      los_memory.c
│  │  │      los_memory_internal.h
│  │  │
│  │  ├─common
│  │  │  ├─memstat
│  │  │  │      los_memstat.c
│  │  │  │
│  │  │  └─multipool
│  │  │          los_multipool.c
│  │  │
│  │  ├─membox
│  │  │      los_membox.c
│  │  │      los_membox_dyn.c
│  │  │
│  │  └─slab
│  │          los_slab.c
│  │          los_slabmem.c
│  │
│  ├─sched
│  │  ├─sched_mq
│  │  │      los_sched.c
│  │  │
│  │  └─sched_sq
│  │          los_priqueue.c
│  │          los_sched.c
│  │
│  └─shellcmd
├─extended
│  │  Makefile
│  ├─cppsupport
│  ├─cpup
│  ├─include
│  ├─lms
│  ├─lowpower
│  ├─perf
│  └─trace
├─include//42 .h
└─init
        los_init.c
        Makefile
```
下面对kernel内各模块进行简单的功能介绍:
>+ Kconfig:内核配置文件
>+ Makefile:内核编译脚本文件
>+ base:
>   + mem(内存管理)：分为动态内存(bestfit slab)和静态内存(membox)。
>   + sched:任务调度，进程管理的相关代码
>   + shellcmd:LiteOS 中与基础内核相关的shell命令，包括memcheck、task、systeminfo、swtmr等
>   + include:LiteOS 基础内核内部使用的头文件
>   + debug:LiteOS 内核调测代码，包括队列、信号量、互斥锁及任务调度的调测
>   + 任务、中断、软件定时器、队列、事件、信号量、互斥锁、tick 等相关代码。
>+ extended:内核扩展功能代码，如计算cpu占用率，实现低功耗，trace 跟踪等。
>+ include:LiteOS 内核头文件
>+ init:LiteOS 内核初始化相关代码

由于基础内核部分为LiteOS kernel的最核心部分，内核扩展功能代码并不是OS所必须的，改写意义不大，且真正影响操作系统安全性的应该是基础内核部分，所以我们把改写方向定在base模块上。
##### 改写任务调度模块的考虑和分析
+ 改写优势：任务调度模块较为独立，适合进行部分改写
+ 改写缺点：任务调度模块偏重于性能和效率而并非是安全性，用 Rust 改写意义不大，且整个sched模块仅有约700行代码，工作量过于微小，不符合我们的预期。
##### 改写内存管理模块的考虑和分析
+ 改写优势：内存管理单元对于操作系统的安全性十分重要，使用Rust进行改写的意义非常大，并且mem模块的独立性也非常好，并且总代码量约为5000行左右，符合我们的改写预期。
+ 改写缺点：由于内存与安全性相关很大，如何合理使用 Rust 将 MMU 改写得更安全是很使得考虑得问题。
##### 改写其他小源码(如事件，队列，信号量，tick等)的分析和考虑
这些部分与 mem 和 sched 模块相比，更加的零散，每个源文件基本上都是300-400的代码量，但是文件没有相互依赖性，导致后期调试与测试时需要很多单独的测试样例，比较麻烦和繁琐，故不将其作为改写对象。
#### 小结：经过上述讨论，我们将改写的模块定位到 LiteOS 的内存管理单元

## 技术依据
---
## 创新点
---
## 工作展望和规划
---
## 参考文献及相关资料



