/*
  This file is part of QAOS

  This file is licensed under the GNU General Public License version 3 (GPL3).

  You should have received a copy of the GNU General Public License
  along with QAOS. If not, see <https://www.gnu.org/licenses/>.

  Copyright (c) 2025-2026 by Kadir Aydın.
*/


use std::{panic::Location, time::Instant};


pub enum LogType {Panic = 5, Error = 4, Warning = 3, Display = 2, Log = 1, Verbose = 0 }


use std::sync::OnceLock;

static BOOT: OnceLock<Instant> = OnceLock::new();

pub fn start() {
  BOOT.get_or_init(Instant::now);
}


fn log_str(typ: LogType, loc: &Location, str: &str) -> String {
  let help = match typ {
    LogType::Verbose => ('V', "\033[1;30m"),
    LogType::Log     => ('L', "\033[1;30m"),
    LogType::Display => ('D', "\033[1;37m"),
    LogType::Warning => ('W', "\033[1;33m"),
    LogType::Error   => ('E', "\033[1;31m"),
    LogType::Panic   => ('P', "\033[1;31m"),
  };
  
  let boot = BOOT.get_or_init(Instant::now);
  let now = Instant::now() - *boot;
  let ms = now.subsec_millis();

  if typ as u8 >= LogType::Warning as u8 {
    format!(
      "{}{} [{}.{:03}] {}\n├─> \033[1;30m{}╰─> \033[1;30mfile: {}:{}:{}\033[0m",
      help.1, help.0,
      ms/1000, ms%1000,
      str,
      
      help.1,
      
      loc.file(),
      loc.line(), loc.column()
    )
  } else {
    format!("{}{} [{}.{:03}] {}\033[0m",
      help.1, help.0,
      ms/1000, ms%1000,
      str
    )
  }
}


#[track_caller]
pub fn log(typ: LogType, str: &str) {
  let loc = Location::caller();

  println!("{}", log_str(typ, loc, str));
}

#[track_caller]
pub fn log_nr(str: &str) -> ! {
  let loc = Location::caller();

  let err = log_str(LogType::Panic, loc, str);

  panic!("{}", err);
}



#[macro_export]
macro_rules! ME_LOG_PANIC { ($str:expr, $($e:expr)*) => { ::mochi_except::log_nr(&format!($str, $(e)*)); }; }

#[macro_export]
macro_rules! ME_LOG { ($ty:expr, $str:expr, $($e:expr)*) => { ::mochi_except::log($ty, &format!($str, $(e)*)); }; }


#[macro_export]
macro_rules! ME_LOG_ERROR { ($str:expr, $($e:expr)*) => { ::mochi_except::ME_LOG(::mochi_except::LogType::Error, $str, $(e)*); }; }
#[macro_export]
macro_rules! ME_LOG_WARN  { ($str:expr, $($e:expr)*) => { ::mochi_except::ME_LOG(::mochi_except::LogType::Warn, $str, $(e)*); }; }
#[macro_export]
macro_rules! ME_LOG_DISP  { ($str:expr, $($e:expr)*) => { ::mochi_except::ME_LOG(::mochi_except::LogType::Disp, $str, $(e)*); }; }
#[macro_export]
macro_rules! ME_LOG_LOG   { ($str:expr, $($e:expr)*) => { ::mochi_except::ME_LOG(::mochi_except::LogType::Log, $str, $(e)*); }; }
#[macro_export]
macro_rules! ME_LOG_VERB  { ($str:expr, $($e:expr)*) => { ::mochi_except::ME_LOG(::mochi_except::LogType::Verb, $str, $(e)*); }; }
