use std::io::prelude::*;
use std::io;
use std::process::Command;
extern crate kernel32;
extern crate winapi;

use std::mem;
use std::os::windows::io::{AsRawHandle, RawHandle};
use std::process::Child;
use std::ptr;
use std::os::raw::c_void;

use winapi::um::{tlhelp32, handleapi};
use winapi::shared::minwindef::*;

const BASE_ADDRESS: u32 = 0x3222D0;

pub fn get_process_pid(process_name: &str) -> Result<u32, &str> {
    // let mut pid = Command::new("pidof").arg(process_name).output()?.stdout;
    // pid.pop();
    // Ok(std::str::from_utf8(&pid)?.parse()?)
    // unsafe { // blame windows, everything is unsafe
    //     let handle = kernel32::CreateToolhelp32Snapshot(tlhelp32::TH32CS_SNAPPROCESS, 0);
    //     if handle == handleapi::INVALID_HANDLE_VALUE {
    //         return Err("Lol")
    //     };
    //     let mut pe32: tlhelp32::PROCESSENTRY32 = Default::default();
    //     pe32.dwSize = mem::size_of::<tlhelp32::PROCESSENTRY32>() as u32;
    //     if kernel32::Process32First(handle, (&mut pe32) as *mut tlhelp32::PROCESSENTRY32) == 0 {
    //         return Err("Lol2")
    //     };
    //     while kernel32::Process32Next(handle, (&mut pe32) as *mut tlhelp32::PROCESSENTRY32) != 0 {
    //         if process_name == pe32.szExeFile.to_string() {
    //             Ok(pe32.th32ProcessID);
    //         };
    //     }
    //     return Err("Not found")
    // }
    Ok(3764)
}

pub struct GDMemory {
    handle: RawHandle,
    last_x_pos_address: u32,
    last_y_pos_address: u32,
    last_is_dead_address: u32,
    last_is_practice_mode_address: u32,
}

impl GDMemory {
    pub fn from_pid(pid: u32) -> io::Result<Self> {
        let handle = unsafe {
            kernel32::OpenProcess(0x0010 | 0x0020, 0, pid)
        };
        if handle == (0 as RawHandle) {
            Err(io::Error::last_os_error())
        } else {
            Ok(Self {
                handle: handle,
                last_x_pos_address: 0,
                last_y_pos_address: 0,
                last_is_dead_address: 0,
                last_is_practice_mode_address: 0,
            })
        }
    }

    pub fn get_addr(&mut self, mut base: u32, offsets: Vec<u32>) -> io::Result<u32> {
        base += offsets[0];
        for offset in offsets.iter().skip(1) {
            base = self.read_int(base)?;
            base += offset;
        }
        Ok(base)
    }

    pub fn read_buf(&self, addr: u32, buf: &mut [u8]) -> io::Result<()> {
        if unsafe {
            kernel32::ReadProcessMemory(self.handle,
                                        addr as *mut c_void,
                                        buf.as_mut_ptr() as *mut c_void,
                                        mem::size_of_val(buf) as u64,
                                        ptr::null_mut())
        } == FALSE {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn write_buf(&self, addr: u32, buf: &[u8]) -> io::Result<()> {
        if unsafe {
            kernel32::WriteProcessMemory(self.handle,
                                        addr as *mut c_void,
                                        buf.as_ptr() as *const c_void,
                                        mem::size_of_val(buf) as u64,
                                        ptr::null_mut())
        } == FALSE {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    pub fn read_int(&mut self, addr: u32) -> io::Result<u32> {
        let mut buffer = [0; 4];
        self.read_buf(addr, &mut buffer)?;
        Ok(u32::from_le_bytes(buffer))
    }

    pub fn read_float(&mut self, addr: u32) -> io::Result<f32> {
        let mut buffer = [0; 4];
        self.read_buf(addr, &mut buffer)?;
        Ok(f32::from_le_bytes(buffer))
    }

    pub fn write_float(&mut self, addr: u32, val: f32) -> io::Result<()> {
        self.write_buf(addr, &val.to_le_bytes())?;
        Ok(())
    }

    pub fn read_bool(&mut self, addr: u32) -> io::Result<bool> {
        let mut buffer = [0];
        self.read_buf(addr, &mut buffer)?;
        Ok(buffer[0] != 0)
    }

    pub fn get_x_pos(&mut self) -> io::Result<f32> {
        self.read_float(self.last_x_pos_address).or_else(|_| {
            self.last_x_pos_address = self.get_addr(BASE_ADDRESS, vec![0x164, 0x224, 0x67C])?;
            self.read_float(self.last_x_pos_address)
        })
    }

    pub fn set_x_pos(&mut self, val: f32) -> io::Result<()> {
        self.write_float(self.last_x_pos_address, val).or_else(|_| {
            self.last_x_pos_address = self.get_addr(BASE_ADDRESS, vec![0x164, 0x224, 0x67C])?;
            self.write_float(self.last_x_pos_address, val)
        })
    }

    pub fn get_y_pos(&mut self) -> io::Result<f32> {
        self.read_float(self.last_y_pos_address).or_else(|_| {
            self.last_y_pos_address = self.get_addr(BASE_ADDRESS, vec![0x164, 0x224, 0x680])?;
            self.read_float(self.last_y_pos_address)
        })
    }

    pub fn set_y_pos(&mut self, val: f32) -> io::Result<()> {
        self.write_float(self.last_y_pos_address, val).or_else(|_| {
            self.last_y_pos_address = self.get_addr(BASE_ADDRESS, vec![0x164, 0x224, 0x680])?;
            self.write_float(self.last_y_pos_address, val)
        })
    }

    pub fn is_dead(&mut self) -> io::Result<bool> {
        self.read_bool(self.last_is_dead_address).or_else(|_| {
            self.last_is_dead_address = self.get_addr(BASE_ADDRESS, vec![0x164, 0x39C])?;
            self.read_bool(self.last_is_dead_address)
        })
    }

    pub fn is_practice_mode(&mut self) -> io::Result<bool> {
        self.read_bool(self.last_is_practice_mode_address).or_else(|_| {
            self.last_is_practice_mode_address = self.get_addr(BASE_ADDRESS, vec![0x164, 0x495])?;
            self.read_bool(self.last_is_practice_mode_address)
        })
    }

    pub fn update_addresses(&mut self) -> io::Result<()> {
        self.last_x_pos_address = self.get_addr(BASE_ADDRESS, vec![0x164, 0x224, 0x67C])?;
        self.last_is_dead_address = self.get_addr(BASE_ADDRESS, vec![0x164, 0x39C])?;
        Ok(())
    }
}