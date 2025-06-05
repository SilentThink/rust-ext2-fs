use std::{collections::BTreeMap, sync::Arc};

use super::Shell;

mod cat;
mod cd;
mod chmod;
mod chown;
mod clear;
mod cp;
mod exit;
mod format;
mod help;
mod ln;
mod ls;
mod mkdir;
mod mv;
mod passwd;
mod pwd;
mod rm;
mod rmdir;
mod touch;
mod useradd;
mod userdel;
mod users;
mod whoami;
mod write;
mod zip;
mod unzip;

pub mod login;

use {
    cat::Cat, cd::Cd, chmod::Chmod, chown::Chown, clear::Clear, cp::Cp, exit::Exit, format::Format,
    help::Help, ln::Ln, login::Login, ls::Ls, mkdir::Mkdir, mv::Mv, passwd::Passwd, pwd::Pwd, rm::Rm, rmdir::RmDir,
    touch::Touch, useradd::Useradd, userdel::UserDel, users::Users, whoami::Whoami, write::Write,
    zip::Zip, unzip::Unzip,
};

pub trait Cmd: Send + Sync {
    fn description(&self) -> String;
    fn run(&self, shell: &mut Shell, argv: &[&str]);
    fn help(&self) -> String {
        self.description()
    }
}

pub type Cmds = Arc<BTreeMap<&'static str, Box<dyn Cmd + Send + Sync>>>;

pub fn cmds() -> Cmds {
    Arc::new(BTreeMap::from([
        ("ls", Box::new(Ls) as Box<dyn Cmd + Send + Sync>),
        ("cd", Box::new(Cd) as Box<dyn Cmd + Send + Sync>),
        ("exit", Box::new(Exit) as Box<dyn Cmd + Send + Sync>),
        ("help", Box::new(Help) as Box<dyn Cmd + Send + Sync>),
        ("?", Box::new(Help) as Box<dyn Cmd + Send + Sync>),
        ("mkdir", Box::new(Mkdir) as Box<dyn Cmd + Send + Sync>),
        ("pwd", Box::new(Pwd) as Box<dyn Cmd + Send + Sync>),
        ("rm", Box::new(Rm) as Box<dyn Cmd + Send + Sync>),
        ("rmdir", Box::new(RmDir) as Box<dyn Cmd + Send + Sync>),
        ("format", Box::new(Format) as Box<dyn Cmd + Send + Sync>),
        ("touch", Box::new(Touch) as Box<dyn Cmd + Send + Sync>),
        ("write", Box::new(Write) as Box<dyn Cmd + Send + Sync>),
        ("cat", Box::new(Cat) as Box<dyn Cmd + Send + Sync>),
        ("cp", Box::new(Cp) as Box<dyn Cmd + Send + Sync>),
        ("mv", Box::new(Mv) as Box<dyn Cmd + Send + Sync>),
        ("ln", Box::new(Ln) as Box<dyn Cmd + Send + Sync>),
        ("login", Box::new(Login) as Box<dyn Cmd + Send + Sync>),
        ("whoami", Box::new(Whoami) as Box<dyn Cmd + Send + Sync>),
        ("passwd", Box::new(Passwd) as Box<dyn Cmd + Send + Sync>),
        ("useradd", Box::new(Useradd) as Box<dyn Cmd + Send + Sync>),
        ("userdel", Box::new(UserDel) as Box<dyn Cmd + Send + Sync>),
        ("chmod", Box::new(Chmod) as Box<dyn Cmd + Send + Sync>),
        ("chown", Box::new(Chown) as Box<dyn Cmd + Send + Sync>),
        ("users", Box::new(Users) as Box<dyn Cmd + Send + Sync>),
        ("clear", Box::new(Clear) as Box<dyn Cmd + Send + Sync>),
        ("zip", Box::new(Zip) as Box<dyn Cmd + Send + Sync>),
        ("unzip", Box::new(Unzip) as Box<dyn Cmd + Send + Sync>),
    ]))
}
