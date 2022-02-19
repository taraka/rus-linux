use nix::sys::wait::WaitPidFlag;
use nix::unistd::{execve, fork, ForkResult, Uid, User};
use std::env;
use std::ffi::CString;
use std::fs::metadata;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

#[derive(Debug)]
struct Command {
    cmd: String,
    args: Vec<String>,
}

struct Environment {
    pwd: PathBuf,
    user: String,
    home: PathBuf,
    path: String,
}

fn main() {
    let env = init();
    repl(env);
}

fn init() -> Environment {
    let user = User::from_uid(Uid::current()).unwrap().unwrap();

    update_pwd(&user.dir);

    Environment {
        pwd: user.dir.clone(),
        user: user.name.clone(),
        home: user.dir,
        path: String::from("/bin:/usr/bin"),
    }
}

fn update_pwd(path: &PathBuf) {
    env::set_current_dir(path).unwrap();
}

fn repl(mut env: Environment) {
    print_prompt(&env);
    for input_result in io::stdin().lock().lines() {
        let input = match input_result {
            Ok(i) => i,
            Err(err) => panic!("Unable to read stdin: {}", err),
        };

        let command = parse_input(input);
        run_command(command, &mut env);
        print_prompt(&env);
    }
}

fn print_prompt(env: &Environment) {
    print!("{}$ ", env.pwd.to_str().unwrap());
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
    if let Some(path) = env.path.split(":").find_map(|p| {
        let filepath = p.to_owned() + "/" + &command.cmd;
        metadata(&filepath).ok().map(|md| {
            if !md.is_file() {
                return None;
            } else {
                Some(String::from(filepath))
            }
        })
    }) {
        fork_exec(path.unwrap(), command, env);
    } else {
        eprintln!("command not found: {}", command.cmd);
    }
}

fn get_environment_for_exec(env: &Environment) -> Vec<CString> {
    vec![
        "PWD=".to_owned() + &env.pwd.to_str().unwrap(),
        "HOME=".to_owned() + &env.home.to_str().unwrap(),
        "USER=".to_owned() + &env.user,
        "PATH=".to_owned() + &env.path,
    ]
    .iter()
    .map(|a| CString::new(a.clone()).unwrap())
    .collect()
}

fn fork_exec(path: String, command: Command, env: &Environment) {
    let path_c = CString::new(path).unwrap();
    let argv_c: Vec<CString> = command
        .args
        .iter()
        .filter(|s| !s.is_empty())
        .map(|a| CString::new(a.clone()).unwrap())
        .collect();
    let env_c = get_environment_for_exec(env);

    unsafe {
        match fork() {
            Ok(ForkResult::Parent { child, .. }) => {
                nix::sys::wait::waitpid(child, Some(WaitPidFlag::empty()))
                    .expect("Error executing waitpid");
            }
            Ok(ForkResult::Child) => {
                println!("{:?} {:?} {:?}", &path_c, &argv_c[..], &env_c[..]);

                execve(&path_c, &argv_c[..], &env_c[..]).expect("evecve failed");
            }
            Err(e) => eprintln!("Fork failed: {}", e),
        }
    }
}

fn run_pwd(_command: Command, env: &Environment) {
    println!("{}", env.pwd.to_str().unwrap());
}

fn run_cd(command: Command, env: &mut Environment) {
    let path = if let Some(p) = command.args.get(1) {
        p
    } else {
        env.home.to_str().unwrap()
    };

    match metadata(&path) {
        Ok(md) => {
            if md.is_dir() {
                env.pwd = PathBuf::from(path);
                update_pwd(&env.pwd);
            } else {
                eprintln!("{} is not a directory!", path);
            }
        }
        Err(err) => eprintln!("{}: {}", path, err),
    }
}

fn run_whoami(_command: Command, env: &Environment) {
    println!("{}", env.user);
}

fn run_exit(_command: Command, _env: &Environment) {
    println!("Thanks for using TomSH");
    println!("Your kernel is going to panic now as it doesn't like it when your init proces exits");
    std::process::exit(0);
}
