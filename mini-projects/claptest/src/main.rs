use clap::{command, Arg, Command, ArgMatches, ArgAction};

fn main() {
    let match_result: ArgMatches = command!()
    .subcommand(
        Command::new("register-person")
            .arg(
                Arg::new("firstname")
                    .short('f')
                    .long("first-name")
                    .aliases(["fname","firstname"])
                    .required(true)
                    .help("The person's first name")
                    // .conflicts_with("lastname")
            )
            .arg(
                Arg::new("lastname")
                    .short('l')
                    .long("last-name")
                    .aliases(["lname","lastname"])
                    .required(true)
                    .help("The person's last name")
            )
    )
    .subcommand(
        Command::new("register-pet")
            .arg(
                Arg::new("pet-name")
                    .long("pet-name")
                    .short('n')
                    .required(true)
            )
    )
    .about("This app registers people with doctor's office.")
    .arg(
        Arg::new("fluffy")
            .long("fluffy")
            .help("Is person waring fluffy coat or not")
            .action(ArgAction::SetTrue),
    )
    .get_matches();

    let fluffy = *match_result.get_one::<bool>("fluffy").unwrap_or(&false);
    println!("Fluffy: {}", fluffy);

    let person_args: &ArgMatches = match_result.subcommand_matches("register-person").unwrap();

    let fname: &String = person_args.get_one::<String>("firstname").unwrap();
    let lname: &String = person_args.get_one::<String>("lastname").unwrap();

    println!("First name: {} last name: {}", fname, lname);
}