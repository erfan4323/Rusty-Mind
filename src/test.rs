use super::*;

#[test]
fn cls_test() {
    cls();
    println!("Hi! this totally works!!");
}

#[test]
fn list_adding_account_test() {
    cls();
    let document1: Account = Account {
        id: "bandicoot".to_owned(),
        full_name: "dan".to_owned(),
        password: "4323".to_owned(),
        score: 0,
    };

    let document2: Account = Account {
        id: "scarface".to_owned(),
        full_name: "lian".to_owned(),
        password: "7988".to_owned(),
        score: 10,
    };

    let list: List<Account> = List::new().prepend(document1).prepend(document2);

    println!("{}", list);
}

#[test]
fn iter_through_list_and_find() {
    let dan: Account = Account::new("bandicoot", "Dan", "4323", 52);
    let jorji: Account = Account::new("GG", "Jorji", "7656", 125);
    let lian: Account = Account::new("scarface", "Lian", "2653", 85);

    let accounts: List<Account> = List::new()
        .prepend(dan.clone())
        .prepend(jorji.clone())
        .prepend(lian.clone());

    let result: Option<&Account> = accounts.find(&lian);

    if result.is_some() {
        println!("{}", result.unwrap())
    } else {
        println!("value not found");
    }
}

#[test]
fn save_user_data() {
    let dan: Account = Account::new("bandicoot", "Dan", "4323", 100);
    let jorji: Account = Account::new("GG", "Jorji", "7656", 865);
    let lian: Account = Account::new("scarface", "Lian", "2653", 85);

    let accounts = List::new()
        .prepend(dan.clone())
        .prepend(jorji.clone())
        .prepend(lian.clone());

    let mut iter: Iter<'_, Account> = accounts.next();
    accounts.save("src/resources/user.txt");
}

#[test]
fn load_user_data() {
    let account_list: Result<List<Account>, Box<dyn Error>> =
        List::load_from_file("src/resources/user.txt");

    let id_to_find: &str = "GG";
    if let Some(account) = account_list.expect("msg").find_by_id(id_to_find) {
        println!("{}", account);
    } else {
        println!("Account not found.");
    }
}

#[test]
fn game() {
    let mut game: Game = Game::set_settings(4, 10, true);
    let mut dan: Account = Account::new("bandicoot", "Dan", "4323", 52);
    let _ = game.start(&mut dan);
}

#[test]
fn menu() {
    let mut menu = MasterMind::new();
    menu.generate_menu();
}
