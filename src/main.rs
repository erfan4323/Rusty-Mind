#![allow(unused)]

use anyhow::Result;
use colored::{ColoredString, Colorize};
use rand::prelude::*;
use std::{
    error::Error,
    fmt::{self, Debug},
    fs::File,
    io::{self, prelude::*, BufReader, Write},
    ops::ControlFlow,
    process::{exit, Command},
    rc::Rc,
    thread::sleep,
    time::{Duration, Instant},
};
// Terminal Functions
fn cls() {
    if cfg!(target_os = "windows") {
        Command::new("cmd")
            .args(["/c", "cls"])
            .spawn()
            .expect("cls command failed to start")
            .wait()
            .expect("failed to wait");
    } else {
        Command::new("clear")
            .spawn()
            .expect("clear command failed to start")
            .wait()
            .expect("failed to wait");
    };
}

// User Account
#[derive(Debug, Clone, PartialEq)]
struct Account {
    id: String,
    full_name: String,
    password: String,
    score: u32,
}

impl Account {
    fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        pass: impl Into<String>,
        sc: u32,
    ) -> Self {
        Account {
            id: id.into(),
            full_name: name.into(),
            password: pass.into(),
            score: sc,
        }
    }

    fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut user_file = File::options().append(true).create(true).open(path)?;

        let data_to_write = format!(
            "{},{},{},{}\n",
            self.id, self.full_name, self.password, self.score
        );
        let _ = user_file.write_all(data_to_write.as_bytes());
        user_file.flush()?;

        Ok(())
    }

    fn from_line(line: &str) -> Option<Account> {
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() == 4 {
            Some(Account {
                id: parts[0].to_string(),
                full_name: parts[1].to_string(),
                password: parts[2].to_string(),
                score: parts[3].parse().unwrap(),
            })
        } else {
            None
        }
    }
}
trait Save {
    fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>>;
}

impl Save for Vec<Account> {
    fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut user_file = File::create(path)?;

        for account in self {
            let data_to_write = format!(
                "{},{},{},{}\n",
                account.id, account.full_name, account.password, account.score
            );
            user_file.write_all(data_to_write.as_bytes())?;
        }

        user_file.flush()?;

        Ok(())
    }
}

impl Save for List<Account> {
    fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let account_vec: Vec<Account> = self.next().map(|acc| (*acc).clone()).collect();
        Vec::<Account>::save(&account_vec, path) // Call the save method on Vec<Account>
    }
}

impl List<Account> {
    fn load_from_file(path: &str) -> Result<Self, Box<dyn Error>> {
        let file: File = File::open(path)?;
        let reader: BufReader<File> = BufReader::new(file);

        let mut account_list: List<Account> = List::new();

        reader.lines().for_each(|line| {
            if let Ok(account_line) = line {
                if let Some(account) = Account::from_line(&account_line) {
                    account_list = account_list.prepend(account);
                }
            }
        });

        Ok(account_list)
    }

    fn find_by_id(&self, id: &str) -> Option<&Account> {
        self.find_by_predicate(|account| account.id == id)
    }

    fn sort_by_id(&self) -> List<Account> {
        let mut sorted_accounts: Vec<Account> = Vec::new();
        let mut current = &self.head;

        while let Some(node) = current {
            sorted_accounts.push(node.value.clone());
            current = &node.next;
        }

        sorted_accounts.sort_by_key(|account| account.id.clone());
        let mut sorted_list = List::new();

        for account in sorted_accounts.iter() {
            sorted_list = sorted_list.prepend(account.clone());
        }

        sorted_list
    }

    fn reverse(&self) -> List<Account> {
        let mut reversed_list = List::new();
        let mut current = &self.head;

        while let Some(node) = current {
            reversed_list = reversed_list.prepend(node.value.clone());
            current = &node.next;
        }

        reversed_list
    }
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{{ ID: {}, Name: {}, Pass: {}, Score: {} }}",
            self.id, self.full_name, self.password, self.score
        )
    }
}

// Linked List Implementation
struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
}

impl<T: std::cmp::PartialEq> List<T> {
    fn new() -> Self {
        List { head: None }
    }

    fn prepend(&self, elem: T) -> List<T> {
        List {
            head: Some(Rc::new(Node {
                value: elem,
                next: self.head.clone(),
            })),
        }
    }

    fn tail(&self) -> List<T> {
        List {
            head: self.head.as_deref().and_then(|node| node.next.clone()),
        }
    }

    fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    fn next(&self) -> Iter<'_, T> {
        Iter {
            next: self.head.as_deref(),
        }
    }

    fn find(&self, value: &T) -> Option<&T> {
        let mut iter: Iter<'_, T> = self.next();

        iter.find(|&prop| prop == value)
    }

    fn find_by_predicate(&self, predicate: impl Fn(&T) -> bool) -> Option<&T> {
        let mut iter: Iter<'_, T> = self.next();

        iter.find(|&prop| predicate(prop))
    }
}

struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            self.next = node.next.as_deref();
            &node.value
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut head: Option<Rc<Node<T>>> = self.head.take();

        while let Some(node) = head {
            if let Ok(mut node) = Rc::try_unwrap(node) {
                head = node.next.take();
            } else {
                break;
            }
        }
    }
}

impl<T: Debug + std::fmt::Display> fmt::Display for List<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut current = &self.head;
        while let Some(node) = current {
            write!(f, "{} -> ", node.value)?;
            current = &node.next;
        }
        write!(f, "None")
    }
}

// Setings

#[derive(PartialEq)]
enum Settings {
    NumLength(u8),
    Tries(u8),
    Repetition(bool),
}

impl Settings {
    fn length(num: u8) -> Self {
        if num > 4 {
            return Self::NumLength(num);
        }

        println!("wrong number");
        Self::NumLength(4)
    }

    fn active_repetition(state: bool) -> Self {
        Self::Repetition(state)
    }

    fn set_tries(tr: u8) -> Self {
        if tr > 10 {
            return Self::Tries(tr);
        }

        println!("wrong number");
        Self::Tries(10)
    }

    fn subtract(&mut self, value: u8) {
        match self {
            Settings::Tries(ref mut tr) => {
                *tr -= value;
            }
            Settings::NumLength(ref mut tr) if (*tr - value) > 0 => {
                *tr -= value;
            }
            _ => {}
        }
    }

    fn addition(&mut self, value: u8) {
        match self {
            Settings::Tries(ref mut tr) if (*tr + value) <= 10 => {
                *tr += value;
            }
            Settings::NumLength(ref mut tr) if (*tr + value) <= 4 => {
                *tr += value;
            }
            _ => {}
        }
    }
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Settings::NumLength(len) => write!(f, "NumLength: {}", len),
            Settings::Tries(tr) => write!(f, "Tries: {}", tr),
            Settings::Repetition(rp) => write!(f, "Repetition: {}", rp),
        }
    }
}

// game

struct Game {
    settings: (Settings, Settings, Settings),
    score: u32,
}

impl Game {
    fn set_settings(len: u8, tr: u8, rp: bool) -> Game {
        let max_index = 10u128.pow(len as u32) - 1;

        let tries = if u128::from(tr) > max_index {
            max_index
        } else {
            tr.into()
        };
        Game {
            settings: (
                Settings::NumLength(len),
                Settings::Tries(tries.try_into().unwrap()),
                Settings::Repetition(rp),
            ),
            score: 0,
        }
    }

    fn start<'a>(&'a mut self, user: &'a mut Account) -> &mut Account {
        let mut game_score: u32 = self.score; // game score
        let _duration: Instant = Instant::now(); // the time took to solve game
        let mut rng: ThreadRng = rand::thread_rng(); // rand gen

        let max_index = 10u128.pow(match &self.settings.0 {
            Settings::NumLength(len) => *len as u32,
            _ => 112,
        }) - 1;

        let min_index = 10u128.pow(match &self.settings.0 {
            Settings::NumLength(len) => *len as u32,
            _ => 4,
        });

        let answear: u128 = rng.gen_range(0..=max_index); // the key player should guess
        let answear: Vec<u128> = answear
            .to_string()
            .chars()
            .map(|c: char| c.to_digit(10).unwrap() as u128)
            .collect();
        let mut guess: String = String::new(); // the player's guess

        loop {
            guess.clear();
            let nums_in_order: i32 = 0;
            let nums_exists: i32 = 0;
            println!("you have {} times to guess.", self.settings.1);

            // get the users input
            match io::stdin().read_line(&mut guess) {
                Ok(_) => {}
                Err(e) => {
                    println!("error: {}", e);
                    self.settings.1.subtract(1);
                    continue;
                }
            }
            println!("Debug : number = {:?}", answear);
            let guess: u128 = match guess.trim().parse() {
                Ok(value) => value,
                Err(e) => {
                    println!("error: {}", e);
                    self.settings.1.subtract(1);
                    continue;
                }
            };
            let guess: Vec<u128> = guess
                .to_string()
                .chars()
                .map(|c| c.to_digit(10).unwrap() as u128)
                .collect();

            // game logic
            if let ControlFlow::Break(_) =
                Game::overal_check(guess, &answear, &mut game_score, nums_in_order, nums_exists)
            {
                break;
            }

            // subtracting tries
            self.settings.1.subtract(1);
            if self.settings.1 == Settings::Tries(0) {
                break;
            }
        }

        let time_took = _duration.elapsed().as_secs();
        game_score += Game::time_score(time_took as f32);
        user.score = game_score;
        user
    }

    fn time_score(time: f32) -> u32 {
        if time >= 300.0 {
            0
        } else if time >= 120.0 {
            50
        } else {
            100
        }
    }

    fn overal_check(
        guess: Vec<u128>,
        answear: &Vec<u128>,
        game_score: &mut u32,
        mut nums_in_order: i32,
        nums_exists: i32,
    ) -> ControlFlow<()> {
        // checks if it is valid
        if let Some(value) = Game::eq_check(&guess, answear, game_score) {
            return value;
        }

        // checks if it is visible through the answear
        Game::order_check(&guess, answear, &mut nums_in_order, game_score);

        Game::exist_check(guess, answear, nums_exists, game_score, nums_in_order);

        ControlFlow::Continue(())
    }

    fn eq_check(
        guess: &Vec<u128>,
        answear: &Vec<u128>,
        game_score: &mut u32,
    ) -> Option<ControlFlow<()>> {
        if *guess == *answear {
            println!("you guessed correctly.");
            *game_score += 100;
            return Some(ControlFlow::Break(()));
        }
        None
    }

    fn order_check(
        guess: &[u128],
        answear: &[u128],
        nums_in_order: &mut i32,
        game_score: &mut u32,
    ) {
        for (g, a) in guess.iter().zip(answear.iter()) {
            if g == a {
                *nums_in_order += 1;
                *game_score += 20;
            }
        }
        println!(
            "you have {} numbers that is correct in the exact place.",
            nums_in_order
        );
    }

    fn exist_check(
        guess: Vec<u128>,
        answear: &[u128],
        mut nums_exists: i32,
        game_score: &mut u32,
        nums_in_order: i32,
    ) {
        for i in &guess {
            if answear.iter().any(|x| x == i) {
                nums_exists += 1;
                *game_score += 10;
            }
        }
        nums_exists -= nums_in_order;
        println!(
            "you have {} numbers that is correct but in another index",
            nums_exists
        );
    }
}

struct MasterMind {
    settings: Option<Settings>,
    account: Option<Account>,
}

impl MasterMind {
    fn new() -> Self {
        MasterMind {
            settings: None,
            account: None,
        }
    }

    fn generate_menu(&mut self) {
        cls();

        let greet: [&str; 3] = [
            "+--------------------------------------------------+",
            "|               Welcome to MasterMind              |",
            "+--------------------------------------------------+",
        ];
        let men: [&str; 6] = [
            "|                    1. Start                      |",
            "|                    2. Sign Up / Login            |",
            "|                    3. Settings                   |",
            "|                    4. Leader Board               |",
            "|                    5. Exit                       |",
            "+--------------------------------------------------+",
        ];

        greet.iter().for_each(|line| {
            println!("{}", line.bright_yellow().bold());
        });
        men.iter().for_each(|lene| {
            println!("{}", lene.yellow());
        });

        let mut choice = String::new();
        println!("Please enter you're choice : ");
        match io::stdin().read_line(&mut choice) {
            Ok(_) => {}
            Err(e) => {
                println!("error: {}", e);
                sleep(Duration::from_secs(2));
                self.generate_menu();
            }
        };
        let choice: u8 = match choice.trim().parse() {
            Ok(value) => value,
            Err(e) => {
                println!("Error: {}", e);
                sleep(Duration::from_secs(2));
                0
            }
        };

        match choice {
            0 => self.generate_menu(),
            1 => self.start(),
            2 => self.login(),
            3 => self.settings(),
            4 => self.leader_board(),
            5 => self.exit(),
            _ => println!("invalid choice"),
        }
    }

    fn start(&mut self) {
        if self.account.is_none() {
            println!("you are playing as no one, please sign up or login.");
            self.login();
        }

        let mut game = Game::set_settings(4, 10, false);

        let account_id = self.account.clone().unwrap().id.clone(); // Save the user's ID

        let mut users_database =
            List::load_from_file("src/resources/user.txt").expect("Cannot load user.txt");
        let mut updated_users: Vec<Account> = Vec::new();

        for user in users_database.next() {
            if user.id == account_id {
                let mut updated_user = user.clone();
                game.start(&mut updated_user);
                updated_users.push(updated_user);
            } else {
                updated_users.push(user.clone());
            }
        }

        // Rebuild the List with the updated users
        let mut new_users_database = List::new();
        for user in &updated_users {
            new_users_database = new_users_database.prepend(user.clone());
        }

        // Save the updated list of accounts to the file
        new_users_database.save("src/resources/user.txt");

        self.account.as_mut().unwrap().score = updated_users
            .iter()
            .find(|user| user.id == account_id)
            .map(|user| user.score)
            .unwrap_or(0);

        println!(
            "Score: {}",
            self.account.clone().unwrap().score.to_string().yellow()
        );

        println!("{}", new_users_database);

        sleep(Duration::from_secs(5));

        self.generate_menu();
    }

    fn login(&mut self) {
        let users_database =
            List::load_from_file("src/resources/user.txt").expect("cannot load user.txt");
        let mut id = String::new();

        println!("Enter your id : ");
        match io::stdin().read_line(&mut id) {
            Ok(_) => {}
            Err(e) => {
                println!("error: {}", e);
                sleep(Duration::from_secs(2));
                self.generate_menu();
            }
        }

        let id: &str = id.trim();

        if let Some(user) = users_database.find_by_id(id) {
            println!("Welcome {}! we will direct you to the menu.", user.id.red());
            sleep(Duration::from_secs(2));
            self.account = Some(user.clone());
            self.generate_menu();
        } else {
            println!("We cannot find the account, so we will direct you to the register dialogue.");
            sleep(Duration::from_secs(2));
            self.account = None;
            self.register();
        }
    }

    fn register(&mut self) {
        let (id, mut name, mut pass) = (String::new(), String::new(), String::new());

        println!("please enter your desiered id : ");
        self.input(id.clone());
        println!("now add your name : ");
        self.input(name.clone());
        println!("then set your password : ");
        self.input(pass.clone());

        let (id, name, pass) = (id.trim(), name.trim(), pass.trim());
        println!("we set your account! :)");

        self.account = Some(Account::new(id, name, pass, 0));
        self.generate_menu();
    }

    fn input(&mut self, mut buf: String) {
        match io::stdin().read_line(&mut buf) {
            Ok(_) => {}
            Err(e) => {
                println!("error: {}", e);
                sleep(Duration::from_secs(2));
                self.generate_menu();
            }
        }
    }

    fn settings(&mut self) {
        let setting: [&str; 8] = [
            "+--------------------------------------------------+",
            "|                     Settings                     |",
            "+--------------------------------------------------+",
            "|                   1. Number Length               |",
            "|                   2. Tries                       |",
            "|                   3. Repetition                  |",
            "|                   4. Menu                        |",
            "+--------------------------------------------------+",
        ];
        setting.iter().for_each(|line| {
            println!("{}", line.bright_green());
        });

        let mut id = String::new();

        println!("Enter your choice : ");
        match io::stdin().read_line(&mut id) {
            Ok(_) => {}
            Err(e) => {
                println!("error: {}", e);
                sleep(Duration::from_secs(2));
                self.generate_menu();
            }
        }
        let id: u8 = match id.trim().parse() {
            Ok(value) => value,
            Err(e) => {
                println!("Error: {}", e);
                sleep(Duration::from_secs(2));
                0
            }
        };

        match id {
            0 => self.generate_menu(),
            1 => {
                let mut ch = String::new();
                print!("Enter the lenth of the number up to : ");
                match io::stdin().read_line(&mut ch) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error: {}", e);
                        sleep(Duration::from_secs(2));
                        self.generate_menu();
                    }
                }
                let ch: u8 = match ch.trim().parse() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("Error: {}", e);
                        sleep(Duration::from_secs(2));
                        4
                    }
                };
                self.settings = Some(Settings::length(ch));
            }
            2 => {
                let mut ch = String::new();
                print!("Enter the lenth of the number up to : ");
                match io::stdin().read_line(&mut ch) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error: {}", e);
                        sleep(Duration::from_secs(2));
                        self.generate_menu();
                    }
                }
                let ch: u8 = match ch.trim().parse() {
                    Ok(value) => value,
                    Err(e) => {
                        println!("Error: {}", e);
                        sleep(Duration::from_secs(2));
                        4
                    }
                };
                self.settings = Some(Settings::set_tries(ch));
            }
            3 => {
                let mut ch = String::new();
                print!("Enter the lenth of the number up to : ");
                match io::stdin().read_line(&mut ch) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("error: {}", e);
                        sleep(Duration::from_secs(2));
                        self.generate_menu();
                    }
                }
                let ch = ch.trim().to_lowercase();
                if ch == "yes" || ch == "y" {
                    self.settings = Some(Settings::active_repetition(true));
                } else if ch == "no" || ch == "n" {
                    self.settings = Some(Settings::active_repetition(false));
                } else {
                    println!("no valid input");
                    self.generate_menu();
                }
            }
            4 => self.generate_menu(),
            _ => println!("invalid id: {}", id),
        }
    }

    fn leader_board(&mut self) {
        let users_database =
            List::load_from_file("src/resources/user.txt").expect("cannot load user.txt");
        let users_database = users_database.sort_by_id().reverse();

        let lb: [&str; 3] = [
            "+--------------------------------------------------+",
            "|                   Leader Bord                    |",
            "+--------------------------------------------------+",
        ];

        lb.iter().for_each(|line| {
            println!("{}", line.cyan());
        });

        for user in users_database.next() {
            println!(
                "{} - {} - {}",
                user.id.magenta().bold(),
                user.score.to_string().yellow().underline(),
                user.full_name
            );
        }
        let mut getch = String::new();
        io::stdin().read_line(&mut getch);
        self.generate_menu();
    }

    fn exit(&mut self) {
        let ex: [&str; 3] = [
            "+--------------------------------------------------+",
            "|                      Exit                        |",
            "+--------------------------------------------------+",
        ];

        ex.iter().for_each(|line| {
            println!("{}", line.bright_red().bold());
        });

        exit(1);
    }
}

// unit testing
#[cfg(test)]
mod test;

fn main() {
    let mut mastermind: MasterMind = MasterMind::new();
    mastermind.generate_menu();
}
