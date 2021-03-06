#[cfg(windows)]
extern crate kernel32;
#[cfg(not(windows))]
extern crate libc;

use std::fmt::Write;
#[cfg(not(windows))]
use std::mem;
use std::time::Duration;

#[cfg(target_os = "linux")]
fn get() -> Result<Duration, String> {
    let mut info: libc::sysinfo = unsafe { mem::zeroed() };
    let ret = unsafe { libc::sysinfo(&mut info) };
    if ret == 0 {
        Ok(Duration::from_secs(info.uptime as u64))
    } else {
        Err("sysinfo failed".to_string())
    }
}

#[cfg(any(
    target_os = "macos",
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd"
))]
fn get() -> Result<Duration, String> {
    let mut request = [libc::CTL_KERN, libc::KERN_BOOTTIME];
    let mut boottime: libc::timeval = unsafe { mem::zeroed() };
    let mut size: libc::size_t = mem::size_of_val(&boottime) as libc::size_t;
    let ret = unsafe {
        libc::sysctl(
            &mut request[0],
            2,
            &mut boottime as *mut libc::timeval as *mut libc::c_void,
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret == 0 {
        Ok((time::now().to_timespec()
            - time::Timespec::new(boottime.tv_sec, boottime.tv_usec * 1000)))
    } else {
        Err("sysctl failed".to_string())
    }
}

#[cfg(target_os = "windows")]
fn get() -> Result<Duration, String> {
    let ret: u64 = unsafe { kernel32::GetTickCount64() };
    Ok(Duration::milliseconds(ret as i64))
}

pub fn uptime(short: bool) -> String {
    match get() {
        Ok(uptime) => {
            let uptime = format_duration(uptime);
            if short {
                parse_for_shorthand_time(uptime)
            } else {
                uptime
            }
        }
        Err(err) => {
            eprintln!("Uptime: {}", err);
            std::process::exit(1);
        }
    }
}

/// Takes Time in milliseconds
/// Returns time formated (e.g. 3 days, 4 hours, 16 mins, 17 seconds)
/// Or (e.g. 1 day, 1 hour, 1 min, 1 second)
fn format_duration(duration: Duration) -> String {
    let sec = duration.as_secs();

    let years = sec / 31557600; // 365.25d
    let sec = sec % 31557600;

    let months = sec / 2630016;
    let sec = sec % 2630016;

    let days = sec / 86400;
    let sec = sec % 86400;

    let hours = sec / 3600;
    let sec = sec % 3600;

    let minutes = sec / 60;
    let seconds = sec % 60;

    let mut s = String::new();

    if years > 0 {
        s.write_fmt(format_args!("{} year", years)).unwrap();

        if years > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if months > 0 {
        s.write_fmt(format_args!("{} month", months)).unwrap();

        if months > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if days > 0 {
        s.write_fmt(format_args!("{} day", days)).unwrap();

        if days > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if hours > 0 || (days > 0) && (minutes > 0 || seconds > 0) {
        s.write_fmt(format_args!("{} hour", hours)).unwrap();

        if hours > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if minutes > 0 || (hours > 0 && seconds > 0) {
        s.write_fmt(format_args!("{} minute", minutes)).unwrap();

        if minutes > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    if seconds > 0 {
        s.write_fmt(format_args!("{} second", seconds)).unwrap();

        if seconds > 1 {
            s.push('s');
        }

        s.push_str(", ");
    }

    debug_assert!(s.len() >= 2);

    if let Some(index) = s.as_str()[..(s.len() - 2)].rfind(", ") {
        s.insert_str(index + 2, "and ");
    }

    let len = s.len();

    let mut v = s.into_bytes();

    unsafe {
        v.set_len(len - 2);

        String::from_utf8_unchecked(v)
    }
}

fn parse_for_shorthand_time(uptime: String) -> String {
    let newtime = str::replace(&uptime, "years", "y");
    let newtime = str::replace(&newtime, "year", "y");
    let newtime = str::replace(&newtime, "days", "d");
    let newtime = str::replace(&newtime, "day", "d");
    let newtime = str::replace(&newtime, "hours", "h");
    let newtime = str::replace(&newtime, "hour", "h");
    let newtime = str::replace(&newtime, "minutes", "m");
    let newtime = str::replace(&newtime, "minute", "m");
    let newtime = str::replace(&newtime, "seconds", "s");
    str::replace(&newtime, "second", "s")
}

#[test]
fn test_uptime_get() {
    assert_eq!(get().is_ok(), true);
}

#[test]
fn test_format_duration() {
    assert_eq!(
        format_duration(Duration::new(864000000, 0)).len() == 65,
        true
    );
    assert_eq!(
        format_duration(Duration::new(864000000, 0))
            == String::from("27 years, 4 months, 16 days, 11 hours, 45 minutes, and 36 seconds"),
        true
    );
    println!("{}", format_duration(Duration::new(864000000, 0)));
    println!(
        "len = {}",
        format_duration(Duration::new(864000000, 0)).len()
    );
}

#[test]
fn test_uptime() {
    let d = get();
    println!("{:#?}", d);
    println!("len = {}", format_duration(d.unwrap()));
}
