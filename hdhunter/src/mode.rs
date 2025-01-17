use std::fmt::Display;

use clap::ValueEnum;

#[repr(i8)]
#[derive(Debug, Clone, Copy, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum Mode {
    Request = 1,
    Response = 2,
    SCGI = 4,
    FastCGI = 8,
    AJP = 16,
    UWSGI = 32,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Request => write!(f, "request"),
            Mode::Response => write!(f, "response"),
            Mode::SCGI => write!(f, "scgi"),
            Mode::FastCGI => write!(f, "fastcgi"),
            Mode::AJP => write!(f, "ajp"),
            Mode::UWSGI => write!(f, "uwsgi"),
        }
    }
}

impl Mode {
    pub fn from_string(mode: &str) -> Self {
        match mode {
            "request" => Mode::Request,
            "response" => Mode::Response,
            "scgi" => Mode::SCGI,
            "fastcgi" => Mode::FastCGI,
            "ajp" => Mode::AJP,
            "uwsgi" => Mode::UWSGI,
            _ => panic!("Invalid mode"),
        }
    }
}