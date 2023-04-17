mod io;
mod shamir_secret;
use std::error::Error;

use clap::{arg, builder, command, Arg, Command};

use io::{decode, encode};

fn main() -> Result<(), Box<dyn Error>> {
    let matches = command!()
        .subcommand(
            Command::new("encode")
                .about("Encode into shares")
                .arg(arg!(-f --datafile <VALUE> "The input datafile to create shares of").required(true))
                .arg(
                    arg!(-m --min_shares <VALUE> "The minimum number of shares required to recover the datafile")
                        .required(true)
                        .value_parser(builder::RangedU64ValueParser::<usize>::new()),
                )
                .arg(
                    arg!(-s --shares <VALUE> "The number of shares to create")
                        .required(true)
                        .value_parser(builder::RangedU64ValueParser::<usize>::new()),
                ),
        )
        .subcommand(
            Command::new("decode")
                .about("Decode shares into datafile")
                .arg(arg!(-f --datafile <VALUE> "The output datafile").required(true))
                .arg(Arg::new("SHARES").num_args(1..)),
        )
        .subcommand_required(true)
        .get_matches();

    if let Some(sub_matches) = matches.subcommand_matches("encode") {
        let datafile = sub_matches.get_one::<String>("datafile").unwrap();
        let min_shares = sub_matches.get_one::<usize>("min_shares").unwrap();
        let num_shares = sub_matches.get_one::<usize>("shares").unwrap();
        encode(datafile.clone(), *num_shares, *min_shares)?;
    } else if let Some(sub_matches) = matches.subcommand_matches("decode") {
        let datafile = sub_matches.get_one::<String>("datafile").unwrap().clone();
        let shares: Vec<String> = sub_matches
            .get_many::<String>("SHARES")
            .unwrap()
            .map(|v| v.clone())
            .collect();
        decode(&shares, datafile)?;
    }
    Ok(())
}
