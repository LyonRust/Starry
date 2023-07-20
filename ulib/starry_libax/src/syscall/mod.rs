use axfs::monolithic_fs::file_io::Kstat;
use axfs::monolithic_fs::fs_stat::FsStat;
use axhal::cpu::this_cpu_id;
use axprocess::process::exit;
use axsignal::action::SigAction;
use axtask::current;
use epoll::*;
use flags::*;
use fs::*;
use log::{error, info};
use mem::*;
use poll::*;
use signal::*;
use task::*;
use utils::*;
extern crate axlog;
extern crate log;

extern crate alloc;
pub mod epoll;
pub mod flags;
pub mod fs;
pub mod futex;
pub mod mem;
pub mod poll;
#[cfg(feature = "signal")]
pub mod signal;
pub mod syscall_id;
pub mod utils;
use syscall_id::*;
use SyscallId::*;

use crate::syscall::{epoll::flags::EpollEvent, poll::syscall_ppoll};

use self::futex::check_dead_wait;
pub mod task;

#[no_mangle]
pub fn syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    let syscall_name = if let Ok(id) = SyscallId::try_from(syscall_id) {
        id
    } else {
        error!("Unsupported syscall id = {}", syscall_id);
        exit(-1);
    };
    check_dead_wait();
    let curr_id = current().id().as_u64();
    info!(
        "cpu id: {}, task id: {}, syscall: id: {} name: {:?}",
        this_cpu_id(),
        curr_id,
        syscall_id,
        syscall_name,
    );
    let ans = match syscall_name {
        OPENAT => syscall_openat(
            args[0],
            args[1] as *const u8,
            args[2] as usize,
            args[3] as u8,
        ),
        CLOSE => syscall_close(args[0]),
        READ => syscall_read(args[0], args[1] as *mut u8, args[2]),
        WRITE => syscall_write(args[0], args[1] as *const u8, args[2]),
        EXIT => syscall_exit(args[0] as i32),
        EXECVE => syscall_exec(args[0] as *const u8, args[1] as *const usize),
        CLONE => syscall_clone(args[0], args[1], args[2], args[3], args[4]),
        NANO_SLEEP => syscall_sleep(args[0] as *const TimeSecs, args[1] as *mut TimeSecs),
        SCHED_YIELD => syscall_yield(),
        TIMES => syscall_time(args[0] as *mut TMS),
        UNAME => syscall_uname(args[0] as *mut UtsName),
        GETTIMEOFDAY => syscall_get_time_of_day(args[0] as *mut TimeVal),
        GETPID => syscall_getpid(),
        GETPPID => syscall_getppid(),
        WAIT4 => syscall_wait4(
            args[0] as isize,
            args[1] as *mut i32,
            WaitFlags::from_bits(args[2] as u32).unwrap(),
        ),
        BRK => syscall_brk(args[0] as usize),
        MUNMAP => syscall_munmap(args[0], args[1]),
        MMAP => syscall_mmap(
            args[0],
            args[1],
            MMAPPROT::from_bits_truncate(args[2] as u32),
            MMAPFlags::from_bits_truncate(args[3] as u32),
            args[4] as usize,
            args[5],
        ),
        MSYNC => syscall_msync(args[0], args[1]),
        GETCWD => syscall_getcwd(args[0] as *mut u8, args[1]),
        PIPE2 => syscall_pipe2(args[0] as *mut u32),
        DUP => syscall_dup(args[0]),
        DUP3 => syscall_dup3(args[0], args[1]),
        MKDIRAT => syscall_mkdirat(args[0], args[1] as *const u8, args[2] as u32),
        CHDIR => syscall_chdir(args[0] as *const u8),
        GETDENTS64 => syscall_getdents64(args[0], args[1] as *mut u8, args[2] as usize),
        UNLINKAT => syscall_unlinkat(args[0], args[1] as *const u8, args[2] as usize),
        MOUNT => syscall_mount(
            args[0] as *const u8,
            args[1] as *const u8,
            args[2] as *const u8,
            args[3] as usize,
            args[4] as *const u8,
        ),
        UNMOUNT => syscall_umount(args[0] as *const u8, args[1] as usize),
        FSTAT => syscall_fstat(args[0], args[1] as *mut Kstat),

        SIGACTION => syscall_sigaction(
            args[0],
            args[1] as *const SigAction,
            args[2] as *mut SigAction,
        ),

        KILL => syscall_kill(args[0] as isize, args[1] as isize),
        TKILL => syscall_tkill(args[0] as isize, args[1] as isize),
        SIGPROCMASK => syscall_sigprocmask(
            SigMaskFlag::from(args[0]),
            args[1] as *const usize,
            args[2] as *mut usize,
            args[3],
        ),
        SIGRETURN => syscall_sigreturn(),
        EXIT_GROUP => syscall_exit(args[0] as i32),
        SET_TID_ADDRESS => syscall_set_tid_address(args[0] as usize),
        PRLIMIT64 => syscall_prlimit64(
            args[0] as usize,
            args[1] as i32,
            args[2] as *const RLimit,
            args[3] as *mut RLimit,
        ),
        CLOCK_GET_TIME => syscall_clock_get_time(args[0] as usize, args[1] as *mut TimeSecs),
        GETUID => syscall_getuid(),
        GETEUID => syscall_geteuid(),
        GETGID => syscall_getgid(),
        GETEGID => syscall_getegid(),
        GETTID => syscall_gettid(),
        FUTEX => syscall_futex(
            args[0] as usize,
            args[1] as i32,
            args[2] as u32,
            args[3] as usize,
            args[4] as usize,
            args[5] as u32,
        ),
        SET_ROBUST_LIST => syscall_set_robust_list(args[0] as usize, args[1] as usize),
        GET_ROBUST_LIST => {
            syscall_get_robust_list(args[0] as i32, args[1] as *mut usize, args[2] as *mut usize)
        }

        READV => syscall_readv(args[0] as usize, args[1] as *mut IoVec, args[2] as usize),
        WRITEV => syscall_writev(args[0] as usize, args[1] as *const IoVec, args[2] as usize),
        MPROTECT => syscall_mprotect(
            args[0] as usize,
            args[1] as usize,
            MMAPPROT::from_bits_truncate(args[2] as u32),
        ),

        FCNTL64 => syscall_fcntl64(args[0] as usize, args[1] as usize, args[2] as usize),
        SYSINFO => syscall_sysinfo(args[0] as *mut SysInfo),
        SETITIMER => syscall_settimer(
            args[0] as usize,
            args[1] as *const ITimerVal,
            args[1] as *mut ITimerVal,
        ),
        GETTIMER => syscall_gettimer(args[0] as usize, args[1] as *mut ITimerVal),
        GETRUSAGE => syscall_getrusage(args[0] as i32, args[1] as *mut TimeVal),
        UMASK => syscall_umask(args[0] as i32),
        PPOLL => syscall_ppoll(
            args[0] as *mut PollFd,
            args[1] as usize,
            args[2] as *const TimeSecs,
            args[3] as usize,
        ),
        EPOLL_CREATE => syscall_epoll_create1(args[0] as usize),
        EPOLL_CTL => syscall_epoll_ctl(
            args[0] as i32,
            args[1] as i32,
            args[2] as i32,
            args[3] as *const EpollEvent,
        ),
        EPOLL_WAIT => syscall_epoll_wait(
            args[0] as i32,
            args[1] as *mut EpollEvent,
            args[2] as i32,
            args[3] as i32,
        ),
        // STATFS => {
        //     error!("args: {} {} {} {}", args[0],args[1],args[2],args[3]);
        //     syscall_statfs(args[0] as *const u8, args[1] as *mut FsStat)
        // },
        FSTATAT => syscall_fstatat(
            args[0] as usize,
            args[1] as *const u8,
            args[2] as *mut Kstat,
        ),
        STATFS => syscall_statfs(args[0] as *const u8, args[1] as *mut FsStat),
        FCHMODAT => syscall_fchmodat(args[0] as usize, args[1] as *const u8, args[2] as usize),
        FACCESSAT => syscall_faccessat(args[0] as usize, args[1] as *const u8, args[2] as usize),
        LSEEK => syscall_lseek(args[0] as usize, args[1] as isize, args[2] as usize),
        PREAD64 => syscall_pread64(
            args[0] as usize,
            args[1] as *mut u8,
            args[2] as usize,
            args[3] as usize,
        ),
        SENDFILE64 => syscall_sendfile64(
            args[0] as usize,
            args[1] as usize,
            args[2] as *mut usize,
            args[3] as usize,
        ),
        FSYNC => syscall_fsync(args[0] as usize),
        UTIMENSAT => syscall_utimensat(
            args[0] as usize,
            args[1] as *const u8,
            args[2] as *const TimeSecs,
            args[3],
        ),
        IOCTL => syscall_ioctl(args[0] as usize, args[1] as usize, args[2] as *mut usize),
        // 不做处理即可
        MEMBARRIER => 0,
        SIGTIMEDWAIT => 0,
        SYSLOG => 0,
        _ => {
            error!("Invalid Syscall Id: {}!", syscall_id);
            // return -1;
            exit(-1)
        }
    };
    // let sstatus = riscv::register::sstatus::read();
    // error!("irq: {}", riscv::register::sstatus::Sstatus::sie(&sstatus));
    info!(
        "curr id: {}, Syscall {} return: {}",
        curr_id, syscall_id, ans
    );
    axhal::arch::disable_irqs();
    ans
}
