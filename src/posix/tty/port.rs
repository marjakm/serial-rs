extern crate libc;
extern crate termios;
extern crate ioctl_rs as ioctl;

use std::ffi::CString;
use std::io;
use std::path::Path;
use std::time::Duration;

use std::os::unix::prelude::*;

use self::libc::{c_int,c_void,size_t};

use ::SerialDevice;
use super::TTYSettings;


#[cfg(target_os = "linux")]
const O_NOCTTY: c_int = 0x00000100;

#[cfg(target_os = "macos")]
const O_NOCTTY: c_int = 0x00020000;

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
const O_NOCTTY: c_int = 0;


/// A TTY-based serial port implementation.
pub struct TTYPort {
  fd: RawFd,
  timeout: Duration
}

impl TTYPort {
    /// Opens a TTY device as a serial port.
    ///
    /// `path` should be the path to a TTY device, e.g., `/dev/ttyS0`.
    ///
    /// ```no_run
    /// use std::path::Path;
    ///
    /// serial::posix::TTYPort::open(Path::new("/dev/ttyS0")).unwrap();
    /// ```
    ///
    /// ## Errors
    ///
    /// * `NoDevice` if the device could not be opened. This could indicate that the device is
    ///   already in use.
    /// * `InvalidInput` if `port` is not a valid device name.
    /// * `Io` for any other error while opening or initializing the device.
    pub fn open(path: &Path) -> ::Result<Self> {
        use self::libc::{O_RDWR,O_NONBLOCK,F_SETFL,EINVAL};

        let cstr = match CString::new(path.as_os_str().as_bytes()) {
            Ok(s) => s,
            Err(_) => return Err(::posix::error::from_raw_os_error(EINVAL))
        };

        let fd = unsafe { libc::open(cstr.as_ptr(), O_RDWR | O_NOCTTY | O_NONBLOCK, 0) };
        if fd < 0 {
            return Err(::posix::error::last_os_error());
        }

        let mut port = TTYPort {
            fd: fd,
            timeout: Duration::from_millis(100)
        };

        // get exclusive access to device
        if let Err(err) = ioctl::tiocexcl(port.fd) {
            return Err(::posix::error::from_io_error(err))
        }

        // clear O_NONBLOCK flag
        if unsafe { libc::fcntl(port.fd, F_SETFL, 0) } < 0 {
            return Err(::posix::error::last_os_error());
        }

        // apply initial settings
        let settings = try!(port.read_settings());
        try!(port.write_settings(&settings));

        Ok(port)
    }

    fn set_pin(&mut self, pin: c_int, level: bool) -> ::Result<()> {
        let retval = if level {
            ioctl::tiocmbis(self.fd, pin)
        }
        else {
            ioctl::tiocmbic(self.fd, pin)
        };

        match retval {
            Ok(()) => Ok(()),
            Err(err) => Err(::posix::error::from_io_error(err))
        }
    }

    fn read_pin(&mut self, pin: c_int) -> ::Result<bool> {
        match ioctl::tiocmget(self.fd) {
            Ok(pins) => Ok(pins & pin != 0),
            Err(err) => Err(::posix::error::from_io_error(err))
        }
    }
}

impl Drop for TTYPort {
    fn drop(&mut self) {
        #![allow(unused_must_use)]
        ioctl::tiocnxcl(self.fd);

        unsafe {
            libc::close(self.fd);
        }
    }
}

impl AsRawFd for TTYPort {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl io::Read for TTYPort {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        try!(::posix::poll::wait_read_fd(self.fd, self.timeout));

        let len = unsafe { libc::read(self.fd, buf.as_ptr() as *mut c_void, buf.len() as size_t) };

        if len >= 0 {
            Ok(len as usize)
        }
        else {
            Err(io::Error::last_os_error())
        }
    }
}

impl io::Write for TTYPort {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        try!(::posix::poll::wait_write_fd(self.fd, self.timeout));

        let len = unsafe { libc::write(self.fd, buf.as_ptr() as *mut c_void, buf.len() as size_t) };

        if len >= 0 {
            Ok(len as usize)
        }
        else {
            Err(io::Error::last_os_error())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        termios::tcdrain(self.fd)
    }
}

impl SerialDevice for TTYPort {
    type Settings = TTYSettings;

    fn read_settings(&self) -> ::Result<TTYSettings> {
        use self::termios::{CREAD,CLOCAL}; // cflags
        use self::termios::{ICANON,ECHO,ECHOE,ECHOK,ECHONL,ISIG,IEXTEN}; // lflags
        use self::termios::{OPOST}; // oflags
        use self::termios::{INLCR,IGNCR,ICRNL,IGNBRK}; // iflags
        use self::termios::{VMIN,VTIME}; // c_cc indexes

        let mut termios = match termios::Termios::from_fd(self.fd) {
            Ok(t) => t,
            Err(e) => return Err(::posix::error::from_io_error(e))
        };

        // setup TTY for binary serial port access
        termios.c_cflag |= CREAD | CLOCAL;
        termios.c_lflag &= !(ICANON | ECHO | ECHOE | ECHOK | ECHONL | ISIG | IEXTEN);
        termios.c_oflag &= !OPOST;
        termios.c_iflag &= !(INLCR | IGNCR | ICRNL | IGNBRK);

        termios.c_cc[VMIN] = 0;
        termios.c_cc[VTIME] = 0;

        Ok(TTYSettings::new(termios))
    }

    fn write_settings(&mut self, settings: &TTYSettings) -> ::Result<()> {
        use self::termios::{tcsetattr,tcflush};
        use self::termios::{TCSANOW,TCIOFLUSH};

        // write settings to TTY
        if let Err(err) = tcsetattr(self.fd, TCSANOW, &settings.termios) {
            return Err(::posix::error::from_io_error(err));
        }

        if let Err(err) = tcflush(self.fd, TCIOFLUSH) {
            return Err(::posix::error::from_io_error(err));
        }

        Ok(())
    }

    fn timeout(&self) -> Duration {
        self.timeout
    }

    fn set_timeout(&mut self, timeout: Duration) -> ::Result<()> {
        self.timeout = timeout;
        Ok(())
    }

    fn set_rts(&mut self, level: bool) -> ::Result<()> {
        self.set_pin(ioctl::TIOCM_RTS, level)
    }

    fn set_dtr(&mut self, level: bool) -> ::Result<()> {
        self.set_pin(ioctl::TIOCM_DTR, level)
    }

    fn read_cts(&mut self) -> ::Result<bool> {
        self.read_pin(ioctl::TIOCM_CTS)
    }

    fn read_dsr(&mut self) -> ::Result<bool> {
        self.read_pin(ioctl::TIOCM_DSR)
    }

    fn read_ri(&mut self) -> ::Result<bool> {
        self.read_pin(ioctl::TIOCM_RI)
    }

    fn read_cd(&mut self) -> ::Result<bool> {
        self.read_pin(ioctl::TIOCM_CD)
    }
}
