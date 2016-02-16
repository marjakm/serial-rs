extern crate libc;
extern crate termios;
extern crate ioctl_rs as ioctl;

use self::termios::speed_t;
use ::SerialPortSettings;

/// Serial port settings for TTY devices.
#[derive(Copy,Clone,Debug)]
pub struct TTYSettings {
    pub termios: termios::Termios
}

impl TTYSettings {
    pub fn new(termios: termios::Termios) -> Self {
        TTYSettings {
            termios: termios
        }
    }

    #[cfg(target_os = "linux")]
    fn set_custom_baud_rate(&mut self, baud: usize) -> ::Result<()> {
        use self::termios::os::target::{BOTHER, CBAUD};
        self.termios.c_cflag &= !CBAUD;
        self.termios.c_cflag |= BOTHER;
        self.termios.c_ispeed = baud as speed_t;
        self.termios.c_ospeed = baud as speed_t;
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn set_custom_baud_rate(&mut self, _baud: usize) -> ::Result<speed_t> {
        Err(super::error::from_raw_os_error(self::libc::EINVAL))
    }

    #[cfg(target_os = "linux")]
    fn get_custom_baud_rate(&self) -> Result<Option<::BaudRate>, ()> {
        use self::termios::os::target::{BOTHER, CBAUD};
        if (self.termios.c_cflag & CBAUD) == BOTHER {
            if self.termios.c_ispeed == self.termios.c_ospeed {
                return Ok(Some(::BaudOther(self.termios.c_ispeed as usize)))
            } else {
                return Err(())
            }
        }
        Ok(None)
    }

    #[cfg(not(target_os = "linux"))]
    fn get_custom_baud_rate(&self) -> Result<Option<::BaudRate>> {
        Ok(None)
    }
}

impl SerialPortSettings for TTYSettings {
    fn baud_rate(&self) -> Option<::BaudRate> {
        use self::termios::{cfgetospeed,cfgetispeed};
        use self::termios::{B50,B75,B110,B134,B150,B200,B300,B600,B1200,B1800,B2400,B4800,B9600,B19200,B38400};
        use self::termios::os::target::{B57600,B115200,B230400};

        match self.get_custom_baud_rate() {
            res @ Ok(Some(_)) => res.unwrap(),
            Err(_) => None,
            Ok(None) => {
                let ospeed = cfgetospeed(&self.termios);
                let ispeed = cfgetispeed(&self.termios);

                if ospeed != ispeed {
                    return None
                }

                match ospeed {
                    B50     => Some(::BaudOther(50)),
                    B75     => Some(::BaudOther(75)),
                    B110    => Some(::Baud110),
                    B134    => Some(::BaudOther(134)),
                    B150    => Some(::BaudOther(150)),
                    B200    => Some(::BaudOther(200)),
                    B300    => Some(::Baud300),
                    B600    => Some(::Baud600),
                    B1200   => Some(::Baud1200),
                    B1800   => Some(::BaudOther(1800)),
                    B2400   => Some(::Baud2400),
                    B4800   => Some(::Baud4800),
                    B9600   => Some(::Baud9600),
                    B19200  => Some(::Baud19200),
                    B38400  => Some(::Baud38400),
                    B57600  => Some(::Baud57600),
                    B115200 => Some(::Baud115200),
                    B230400 => Some(::BaudOther(230400)),
                    _       => None
                }
            }
        }
    }

    fn char_size(&self) -> Option<::CharSize> {
        use self::termios::{CSIZE,CS5,CS6,CS7,CS8};

        match self.termios.c_cflag & CSIZE {
            CS8 => Some(::Bits8),
            CS7 => Some(::Bits7),
            CS6 => Some(::Bits6),
            CS5 => Some(::Bits5),

            _ => None
        }
    }

    fn parity(&self) -> Option<::Parity> {
        use self::termios::{PARENB,PARODD};

        if self.termios.c_cflag & PARENB != 0 {
            if self.termios.c_cflag & PARODD != 0 {
                Some(::ParityOdd)
            }
            else {
                Some(::ParityEven)
            }
        }
        else {
            Some(::ParityNone)
        }
    }

    fn stop_bits(&self) -> Option<::StopBits> {
        use self::termios::{CSTOPB};

        if self.termios.c_cflag & CSTOPB != 0 {
            Some(::Stop2)
        }
        else {
            Some(::Stop1)
        }
    }

    fn flow_control(&self) -> Option<::FlowControl> {
        use self::termios::{IXON,IXOFF};
        use self::termios::os::target::{CRTSCTS};

        if self.termios.c_cflag & CRTSCTS != 0 {
            Some(::FlowHardware)
        }
        else if self.termios.c_iflag & (IXON | IXOFF) != 0 {
            Some(::FlowSoftware)
        }
        else {
            Some(::FlowNone)
        }
    }

    fn set_baud_rate(&mut self, baud_rate: ::BaudRate) -> ::Result<()> {
        use self::termios::cfsetspeed;
        use self::termios::{B50,B75,B110,B134,B150,B200,B300,B600,B1200,B1800,B2400,B4800,B9600,B19200,B38400};
        use self::termios::os::target::{B57600,B115200,B230400};

        let baud = match baud_rate {
            ::BaudOther(50)     => B50,
            ::BaudOther(75)     => B75,
            ::Baud110           => B110,
            ::BaudOther(134)    => B134,
            ::BaudOther(150)    => B150,
            ::BaudOther(200)    => B200,
            ::Baud300           => B300,
            ::Baud600           => B600,
            ::Baud1200          => B1200,
            ::BaudOther(1800)   => B1800,
            ::Baud2400          => B2400,
            ::Baud4800          => B4800,
            ::Baud9600          => B9600,
            ::Baud19200         => B19200,
            ::Baud38400         => B38400,
            ::Baud57600         => B57600,
            ::Baud115200        => B115200,
            ::BaudOther(230400) => B230400,
            ::BaudOther(b)      => return self.set_custom_baud_rate(b)
        };

        match cfsetspeed(&mut self.termios, baud) {
            Ok(()) => Ok(()),
            Err(err) => Err(::posix::error::from_io_error(err))
        }
    }

    fn set_char_size(&mut self, char_size: ::CharSize) {
        use self::termios::{CSIZE,CS5,CS6,CS7,CS8};

        let size = match char_size {
            ::Bits5 => CS5,
            ::Bits6 => CS6,
            ::Bits7 => CS7,
            ::Bits8 => CS8
        };

        self.termios.c_cflag &= !CSIZE;
        self.termios.c_cflag |= size;
    }

    fn set_parity(&mut self, parity: ::Parity) {
        use self::termios::{PARENB,PARODD,INPCK,IGNPAR};

        match parity {
            ::ParityNone => {
                self.termios.c_cflag &= !(PARENB | PARODD);
                self.termios.c_iflag &= !INPCK;
                self.termios.c_iflag |= IGNPAR;
            },
            ::ParityOdd => {
                self.termios.c_cflag |= PARENB | PARODD;
                self.termios.c_iflag |= INPCK;
                self.termios.c_iflag &= !IGNPAR;
            },
            ::ParityEven => {
                self.termios.c_cflag &= !PARODD;
                self.termios.c_cflag |= PARENB;
                self.termios.c_iflag |= INPCK;
                self.termios.c_iflag &= !IGNPAR;
            }
        };
    }

    fn set_stop_bits(&mut self, stop_bits: ::StopBits) {
        use self::termios::{CSTOPB};

        match stop_bits {
            ::Stop1 => self.termios.c_cflag &= !CSTOPB,
            ::Stop2 => self.termios.c_cflag |= CSTOPB
        };
    }

    fn set_flow_control(&mut self, flow_control: ::FlowControl) {
        use self::termios::{IXON,IXOFF};
        use self::termios::os::target::{CRTSCTS};

        match flow_control {
            ::FlowNone => {
                self.termios.c_iflag &= !(IXON | IXOFF);
                self.termios.c_cflag &= !CRTSCTS;
            },
            ::FlowSoftware => {
                self.termios.c_iflag |= IXON | IXOFF;
                self.termios.c_cflag &= !CRTSCTS;
            },
            ::FlowHardware => {
                self.termios.c_iflag &= !(IXON | IXOFF);
                self.termios.c_cflag |= CRTSCTS;
            }
        };
    }
}
