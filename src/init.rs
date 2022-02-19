use nix::sys::stat::*;
use nix::unistd::{execve, chown, Uid, Gid};
use std::path::PathBuf;
use std::ffi::CString;
use std::fs;

fn main() {
    println!(":::: Setting up devices ::::");
    mknod(&PathBuf::from("/dev/random"), SFlag::S_IFCHR, Mode::all(), (1<<8) + 8).unwrap();
    mknod(&PathBuf::from("/dev/urandom"), SFlag::S_IFCHR, Mode::all(), (1<<8) + 9).unwrap();
    
    chown(&PathBuf::from("/dev/random"), Some(Uid::current()), Some(Gid::current())).unwrap();
    chown(&PathBuf::from("/dev/urandom"), Some(Uid::current()), Some(Gid::current())).unwrap();
    print_dev();
    println!(":::: Starting Shell ::::");
    run_sh();
}

fn run_sh() {
    let path_c = CString::new("/bin/sh").unwrap();
    let argv_c: Vec<CString> = vec![path_c.clone()];
    let env_c: Vec<CString> = vec![];

    execve(&path_c, &argv_c[..], &env_c[..]).expect("evecve failed");
}

fn print_dev() {

    let files: Vec<String> = 
            fs::read_dir("/dev")
                .unwrap()
                .map(|f| format!("---- {}", f.unwrap().path().display()))
                .collect();

    println!("{}", files.join("\n"));
}
