use std::io::{self, BufRead, Write};
use std::fs::metadata;
use nix::unistd::{execve, fork, ForkResult};
use nix::sys::wait::{waitpid, WaitPidFlag};
use libc::{getpwuid, geteuid};
use std::ffi::CString;

#[derive(Debug)]
struct Command {
    cmd: String,
    args: Vec<String>,
}

struct Environment {
    pwd: String,
    user: String,
    home: String,
    path: String,
}

fn main() {
    let env = init();
    repl(env);
}

fn init() -> Environment {

    let home_cstr: CString;

    unsafe {
        let euid = geteuid();
        let passwd = getpwuid(euid as u32);

        if passwd.is_null() {
            panic!("No passwd entry found for user ({})", euid);
        }

        home_cstr = CString::from_raw((*passwd).pw_dir);
    }

    let home = home_cstr.into_string().unwrap();

    Environment {
        pwd: home.clone(),
        user: String::from("root"),
        home: home,
        path: String::from("/bin"),
    }
}

fn repl(mut env: Environment) {

    print_prompt(&env);
    for input_result in io::stdin().lock().lines() {
        
        let input = match input_result {
            Ok(i) => i ,
            Err(err) => panic!("Unable to read stdin: {}", err),
        };

        let command = parse_input(input);
        run_command(command, &mut env);
        print_prompt(&env);

    }
}

fn print_prompt(env: &Environment) {
    print!("{}# ", env.pwd);
    io::stdout().flush().unwrap();
}

fn parse_input(input: String) -> Command {
    let args: Vec<String> = input.split(' ').map(|s| String::from(s)).collect();
    Command {
        cmd: args[0].clone(),
        args: args,
    }
}

fn run_command(command: Command, env: &mut Environment) {
    match &command.cmd[..] {
        "pwd" => run_pwd(command, env),
        "cd" => run_cd(command, env),
        "whoami" => run_whoami(command, env),
        "exit" => run_exit(command, env),
        "" => (),
        //_ => eprintln!("Command ({}) not found", command.cmd),
        _ => try_exec(command, env),
    }
}

fn try_exec(command: Command, env: &Environment) {
    let path = env.path.clone() + "/" + &command.cmd;
    match metadata(&path) {
        Ok(md) => {
            if !md.is_file() {
                eprintln!("{} isn't a file!", path);
                return;
            }

            fork_exec(path, command, env);

            
        },
        Err(err) => eprintln!("command not found: {}", path),
    }
}

fn fork_exec(path: String, command: Command, env: &Environment) {
    let path_c = CString::new(path).unwrap();
    let argv_c: Vec<CString> = command.args.iter().map(|a| CString::new(a.clone()).unwrap()).collect();
    let env_c: Vec<CString> = vec![];

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                nix::sys::wait::waitpid(child, Some(WaitPidFlag::empty()));
            }
            Ok(ForkResult::Child) => { execve(&path_c, &argv_c[..], &env_c[..]); },
            Err(e) => eprintln!("Fork failed"),
        }
    }

    
}

fn run_pwd(_command: Command, env: &Environment) {
    println!("{}", env.pwd);
}


fn run_cd(command: Command, env: &mut Environment) {
    let path = command.args.get(1).unwrap_or(&env.home).clone();
    match metadata(&path) {
        Ok(md) => {
            if md.is_dir() {
                env.pwd = path;
            }
            else {
                eprintln!("{} is not a directory!", path);
            }
        },
        Err(err) => eprintln!("{}: {}", path, err),
    }
}


fn run_whoami(_command: Command, env: &Environment) {
    println!("{}", env.user);
}

fn run_exit(_command: Command, env: &Environment) {
    println!("Thanks for using TomSH");
    println!("Your kernel is going to panic now as it doesn't like it when your init proces exits");
    std::process::exit(0);
}