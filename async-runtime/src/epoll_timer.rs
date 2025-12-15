use std::io;
use std::mem::MaybeUninit;
use std::os::raw::c_int;
use std::os::unix::io::RawFd;
use std::time::Duration;

const MAX_EVENTS: usize = 16;

fn main() -> io::Result<()> {
    unsafe {
        // 1) create epoll instance
        let epfd = libc::epoll_create1(libc::EPOLL_CLOEXEC);
        if epfd < 0 {
            return Err(io::Error::last_os_error());
        }

        // 2) create timerfd (monotonic clock, non-blocking)
        let timerfd = libc::timerfd_create(
            libc::CLOCK_MONOTONIC,
            libc::TFD_NONBLOCK | libc::TFD_CLOEXEC,
        );
        if timerfd < 0 {
            libc::close(epfd);
            return Err(io::Error::last_os_error());
        }

        // 3) arm the timer (first fire after 1s, then every 1s)
        let its = libc::itimerspec {
            it_interval: libc::timespec {
                tv_sec: 1,
                tv_nsec: 0,
            },
            it_value: libc::timespec {
                tv_sec: 1,
                tv_nsec: 0,
            },
        };

        if libc::timerfd_settime(timerfd, 0, &its, std::ptr::null_mut()) < 0 {
            libc::close(timerfd);
            libc::close(epfd);
            return Err(io::Error::last_os_error());
        }

        // 4) register timerfd with epoll
        let mut ev = libc::epoll_event {
            events: libc::EPOLLIN as u32,
            u64: timerfd as u64,
        };

        if libc::epoll_ctl(epfd, libc::EPOLL_CTL_ADD, timerfd, &mut ev) < 0 {
            libc::close(timerfd);
            libc::close(epfd);
            return Err(io::Error::last_os_error());
        }

        println!("timer started (1s interval)");

        // 5) event loop
        let mut events: [libc::epoll_event; MAX_EVENTS] =
            [MaybeUninit::zeroed().assume_init(); MAX_EVENTS];

        loop {
            let n = libc::epoll_wait(epfd, events.as_mut_ptr(), MAX_EVENTS as c_int, -1);
            if n < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                break Err(err);
            }

            for i in 0..n as usize {
                let ev = events[i];
                let fd = ev.u64 as RawFd;

                if fd == timerfd {
                    // MUST read to clear the event
                    let mut expirations: u64 = 0;
                    let res = libc::read(
                        timerfd,
                        &mut expirations as *mut _ as *mut _,
                        std::mem::size_of::<u64>(),
                    );

                    if res < 0 {
                        let err = io::Error::last_os_error();
                        if err.kind() != io::ErrorKind::WouldBlock {
                            break Err(err);
                        }
                    } else {
                        println!("tick ({} expirations)", expirations);
                    }
                }
            }
        }
    }
}
